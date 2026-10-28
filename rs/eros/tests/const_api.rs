use eros::{ErrorReport, ErrorUnion, MsgError, TypeSet};

#[test]
fn static_message_text_can_be_read_at_compile_time() {
    static MESSAGE: &str = "message \0 λ";
    // Borrow explicitly so const evaluation never needs to drop the owner.
    const ERROR: &MsgError = &MsgError::from_static_ref(&MESSAGE);
    const TEXT: &str = ERROR.as_str();
    assert_eq!(TEXT, MESSAGE);
    assert_eq!(TEXT.as_ptr(), MESSAGE.as_ptr());

    #[cfg(feature = "alloc")]
    {
        const BORROWED: &MsgError = &MsgError::from_static("borrowed");
        const BORROWED_TEXT: &str = BORROWED.as_str();
        assert_eq!(BORROWED_TEXT, "borrowed");
    }
}

#[cfg(feature = "alloc")]
#[test]
fn owned_messages_work_in_const_contexts_and_keep_their_allocation_at_runtime() {
    // An empty String can be constructed in const evaluation without allocating.
    const EMPTY: &MsgError = &MsgError::from_owned(String::new());
    const EMPTY_TEXT: &str = EMPTY.as_str();
    assert_eq!(EMPTY_TEXT, "");

    let message = String::from("owned \0 λ");
    let pointer = message.as_ptr();
    let error = MsgError::from_owned(message);
    assert_eq!(error.as_str(), "owned \0 λ");
    assert_eq!(error.as_str().as_ptr(), pointer);
}

// Diagnostics are captured at runtime. These wrappers check that every
// feature-gated accessor remains callable from a const fn.
const fn saved_report<E: TypeSet>(error: &ErrorUnion<E>) -> ErrorReport<'_> {
    let report = error.report();
    #[cfg(feature = "backtrace")]
    {
        let _ = error.backtrace();
        let _ = report.backtrace();
    }
    #[cfg(feature = "error_span_trace")]
    {
        let _ = error.span_trace();
        let _ = report.span_trace();
    }
    #[cfg(feature = "error_location")]
    {
        let _ = error.location();
        let _ = report.location();
    }
    report
}

#[cfg(feature = "alloc")]
#[test]
fn const_report_accessors_preserve_shared_interior_mutability_and_diagnostics() {
    use core::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug)]
    struct Counter(AtomicUsize);
    impl core::fmt::Display for Counter {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{}", self.0.load(Ordering::SeqCst))
        }
    }
    impl core::error::Error for Counter {}

    assert_eq!(size_of::<Counter>(), size_of::<usize>());
    let error: ErrorUnion<(Counter,)> = ErrorUnion::new(Counter(AtomicUsize::new(41)));
    let report = saved_report(&error);
    let root = (report.inner() as &dyn core::any::Any)
        .downcast_ref::<Counter>()
        .unwrap();
    assert!(core::ptr::eq(root, error.as_single()));
    error.as_single().0.fetch_add(1, Ordering::SeqCst);
    assert_eq!(root.0.load(Ordering::SeqCst), 42);
    root.0.fetch_add(1, Ordering::SeqCst);
    assert_eq!(error.as_single().0.load(Ordering::SeqCst), 43);

    #[cfg(feature = "backtrace")]
    assert!(core::ptr::eq(report.backtrace(), error.backtrace()));
    #[cfg(feature = "error_span_trace")]
    assert!(core::ptr::eq(report.span_trace(), error.span_trace()));
    #[cfg(feature = "error_location")]
    assert!(core::ptr::eq(report.location(), error.location()));
}

#[cfg(feature = "context")]
#[test]
fn const_context_accessors_borrow_static_owned_and_boxed_values() {
    const fn text(frame: &eros::ContextFrame) -> Option<&str> {
        #[cfg(feature = "context_location")]
        let _ = frame.location();
        #[cfg(feature = "context_span_trace")]
        let _ = frame.span_trace();
        #[cfg(feature = "user_context")]
        let _ = frame.is_user_context();
        frame.value().as_str()
    }

    let message = String::from("owned \0 λ");
    let pointer = message.as_ptr();
    let boxed: Box<dyn eros::SendSyncDebugDisplay> = Box::new(String::from("boxed"));
    let error: ErrorUnion<(core::fmt::Error,)> = ErrorUnion::new(core::fmt::Error);
    let error = error.context("static").context(message).context(boxed);
    let mut frames = error.contexts();
    assert_eq!(text(frames.next().unwrap()), Some("static"));
    let owned_text = text(frames.next().unwrap()).unwrap();
    assert_eq!(owned_text, "owned \0 λ");
    assert_eq!(owned_text.as_ptr(), pointer);
    assert_eq!(text(frames.next().unwrap()), None);
}

#[cfg(feature = "alloc")]
#[test]
fn batches_can_be_initialized_and_inspected_at_compile_time() {
    use eros::ErrorUnionVec;

    const EMPTY: &ErrorUnionVec<(core::fmt::Error,)> = &ErrorUnionVec::new(Vec::new());
    const LEN: usize = EMPTY.len();
    const IS_EMPTY: bool = EMPTY.is_empty();
    assert_eq!(LEN, 0);
    assert!(IS_EMPTY);

    let mut errors: ErrorUnionVec<(core::fmt::Error,)> = ErrorUnionVec::new(Vec::new());
    errors.push(ErrorUnion::new(core::fmt::Error));
    assert_eq!(errors.len(), 1);
    assert!(!errors.is_empty());
    assert_eq!(errors.into_inner().len(), 1);
}

#[cfg(not(feature = "alloc"))]
#[test]
fn const_report_accessors_borrow_zero_sized_roots() {
    let error: ErrorUnion<(core::fmt::Error,)> = ErrorUnion::new(core::fmt::Error);
    let report = saved_report(&error);
    // Trait-object vtables may be duplicated across codegen units. Compare
    // the data address and concrete type rather than fat-pointer equality.
    assert_eq!(
        core::ptr::from_ref(report.inner()).cast::<()>(),
        core::ptr::from_ref(error.inner()).cast::<()>()
    );
    assert_eq!(
        core::any::Any::type_id(report.inner()),
        core::any::Any::type_id(error.inner())
    );
}
