use core::{any::Any, error::Error, fmt};
#[cfg(not(feature = "alloc"))]
use eros::AnyError;
#[cfg(not(feature = "context"))]
use eros::Context;
use eros::{ErrorUnion, IntoAnyUnion, IntoUnion, ReshapeUnion};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, PartialEq, Eq)]
struct Timeout;
impl fmt::Display for Timeout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("timeout")
    }
}
impl Error for Timeout {}

#[eros::error_enum(Owned)]
#[eros::error_enum_ref(Borrowed)]
#[eros::error_enum_mut(Mutable)]
#[eros::error_enum_kind(Kind)]
type Failures = (Timeout, fmt::Error);

#[eros::error_enum(MixedError)]
type MixedSet = (Timeout, std::io::Error);

#[test]
fn typed_erased_and_enum_apis_work_with_zero_sized_errors() {
    fn assert_traits<T: Send + Sync>() {}
    assert_traits::<ErrorUnion<Failures>>();
    assert_traits::<ErrorUnion>();

    let mut union: ErrorUnion<Failures> = ErrorUnion::new(Timeout);
    assert!(union.is_inner::<Timeout>());
    assert!(!union.is_inner::<std::io::Error>());
    assert!(union.downcast_inner_ref::<std::io::Error>().is_none());
    assert!(union.downcast_inner_mut::<Timeout>().is_some());
    assert!((union.inner_mut() as &mut dyn Any).is::<Timeout>());
    assert!(matches!(
        Borrowed::from(&union),
        Borrowed::Timeout(&Timeout)
    ));
    assert!(matches!(Mutable::from(&mut union), Mutable::Timeout(_)));
    let union = union.downcast_inner::<std::io::Error>().unwrap_err();
    assert!(matches!(Owned::from(union), Owned::Timeout(Timeout)));

    let mut erased: ErrorUnion = ErrorUnion::new(Timeout);
    assert!(matches!(
        Borrowed::try_from(&erased),
        Ok(Borrowed::Timeout(_))
    ));
    assert!(matches!(
        Mutable::try_from(&mut erased),
        Ok(Mutable::Timeout(_))
    ));
    assert!(matches!(Kind::try_from(erased), Ok(Kind::Timeout)));
    let erased: ErrorUnion = eros::error!(Timeout);
    assert!(matches!(
        Owned::try_from(erased),
        Ok(Owned::Timeout(Timeout))
    ));

    let singleton: ErrorUnion<(Timeout,)> = Timeout.into();
    assert_eq!(*singleton, Timeout);
    assert_eq!(singleton.as_single(), &Timeout);
    let adapter = singleton.into_std_error();
    assert_eq!(adapter.to_string(), "timeout");
    assert_eq!(adapter.into_union().into_single(), Timeout);

    let result: eros::Result<(), Failures> = Err::<(), _>(Timeout).union();
    result
        .recover(|_: ErrorUnion<(Timeout,)>| ())
        .recover(|_: ErrorUnion<(fmt::Error,)>| ())
        .into_value();
    let result: eros::Result<u8, Failures> = Err(ErrorUnion::new(Timeout));
    let result: eros::Result<u8, (fmt::Error,)> =
        result.try_recover(|_: ErrorUnion<(Timeout,)>| Err(ErrorUnion::new(fmt::Error)));
    assert_eq!(
        result
            .recover(|_: ErrorUnion<(fmt::Error,)>| 7)
            .into_value(),
        7
    );
    let erased = Err::<(), _>(Timeout).any_union().unwrap_err();
    assert_eq!(erased.downcast_inner::<Timeout>().unwrap(), Timeout);

    fn validate(valid: bool) -> eros::Result<(), (Timeout,)> {
        eros::ensure!(valid, Timeout);
        Ok(())
    }
    assert!(validate(true).is_ok());
    assert_eq!(validate(false).unwrap_err().into_single(), Timeout);

    // Only the active error must be zero-sized, not every listed alternative.
    let union: ErrorUnion<MixedSet> = ErrorUnion::new(Timeout);
    assert!(matches!(
        MixedError::from(union),
        MixedError::Timeout(Timeout)
    ));
}

#[test]
fn zero_sized_roots_can_borrow_non_zero_sized_static_sources() {
    #[derive(Debug)]
    struct Source {
        message: &'static str,
        cause: Option<&'static Source>,
    }
    impl fmt::Display for Source {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.message)
        }
    }
    impl Error for Source {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            self.cause.map(|cause| cause as &dyn Error)
        }
    }
    static LEAF: Source = Source {
        message: "leaf",
        cause: None,
    };
    static SOURCE: Source = Source {
        message: "source\nmore",
        cause: Some(&LEAF),
    };
    #[derive(Debug)]
    struct Root;
    impl fmt::Display for Root {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("root")
        }
    }
    impl Error for Root {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&SOURCE)
        }
    }
    let union: ErrorUnion<(Root,)> = ErrorUnion::new(Root);
    assert_eq!(union.source().unwrap().to_string(), "source\nmore");
    assert_eq!(union.to_string(), "root <- source\nmore <- leaf");
    assert_eq!(std::format!("{union:#}"), "root");
    assert_eq!(
        std::format!("{union:#?}"),
        "root\n  caused by: source\n             more\n  caused by: leaf"
    );
}

#[cfg(not(feature = "context"))]
#[test]
fn static_context_remains_a_noop_without_allocation() {
    let union: ErrorUnion<(Timeout,)> = ErrorUnion::new(Timeout);
    let union = union
        .context("ignored")
        .with_context(|| -> &'static str { panic!("disabled lazy context must not run") });
    assert_eq!(union.into_single(), Timeout);
    let result: eros::Result<(), (Timeout,)> = Err::<(), _>(Timeout).context("ignored");
    assert_eq!(result.unwrap_err().into_single(), Timeout);
    let result = None::<()>
        .with_context(|| -> &'static str { panic!("disabled lazy Option context must not run") });
    assert!(result.unwrap_err().is_inner::<eros::AbsentValueError>());
}

#[cfg(feature = "location")]
#[test]
fn reshaping_and_mapping_preserve_the_original_location() {
    let expected = line!() + 1;
    let union: ErrorUnion<(Timeout,)> = ErrorUnion::new(Timeout);
    let location = union.location();
    assert_eq!(location.file(), file!());
    assert_eq!(location.line(), expected);
    let union = union
        .widen::<Failures, _>()
        .narrow::<(Timeout,), _>()
        .unwrap();
    let mapped = union.map_single(|_| fmt::Error);
    assert!(core::ptr::eq(location, mapped.location()));
    assert_eq!(mapped.into_single(), fmt::Error);

    let expected = line!() + 1;
    let union: ErrorUnion = eros::error!(Timeout);
    assert_eq!(union.location().line(), expected);
}

static DROPS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
#[repr(align(256))]
struct Tracked;
impl fmt::Display for Tracked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("tracked")
    }
}
impl Error for Tracked {}
impl Drop for Tracked {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

#[eros::error_enum(TrackedEnum)]
type TrackedSet = (Tracked,);

#[test]
fn aligned_zsts_preserve_ownership_through_borrowing_extraction_and_panics() {
    assert_eq!(size_of::<Tracked>(), 0);
    DROPS.store(0, Ordering::SeqCst);
    let mut first: ErrorUnion<TrackedSet> = ErrorUnion::new(Tracked);
    let mut second: ErrorUnion<TrackedSet> = ErrorUnion::new(Tracked);
    assert_eq!((first.as_single() as *const Tracked).addr() % 256, 0);
    let (a, b) = (first.as_single_mut(), second.as_single_mut());
    *a = Tracked;
    assert_eq!(size_of_val(b), 0);
    assert_eq!(DROPS.load(Ordering::SeqCst), 1);
    let first = first.downcast_inner::<std::io::Error>().unwrap_err();
    let extracted = first.into_std_error().into_union().into_single();
    assert_eq!(DROPS.load(Ordering::SeqCst), 1);
    drop(extracted);
    drop(second);
    assert_eq!(DROPS.load(Ordering::SeqCst), 3);

    for extract in [
        |union: ErrorUnion<TrackedSet>| union.into_single(),
        |union: ErrorUnion<TrackedSet>| union.narrow::<Tracked, _>().unwrap(),
        |union: ErrorUnion<TrackedSet>| union.downcast_inner::<Tracked>().unwrap(),
        |union: ErrorUnion<TrackedSet>| {
            let TrackedEnum::Tracked(value) = union.into();
            value
        },
    ] {
        let before = DROPS.load(Ordering::SeqCst);
        let value = extract(ErrorUnion::new(Tracked));
        assert_eq!(DROPS.load(Ordering::SeqCst), before);
        drop(value);
        assert_eq!(DROPS.load(Ordering::SeqCst), before + 1);
    }

    let before = DROPS.load(Ordering::SeqCst);
    let union: ErrorUnion<TrackedSet> = ErrorUnion::new(Tracked);
    let _ = union.map_single(|_value| fmt::Error).into_single();
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 1);
    let before = DROPS.load(Ordering::SeqCst);
    let panicked = std::panic::catch_unwind(|| {
        let union: ErrorUnion<TrackedSet> = ErrorUnion::new(Tracked);
        let _: ErrorUnion<(fmt::Error,)> = union.map_single(|_value| panic!("mapping failed"));
    });
    assert!(panicked.is_err());
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 1);
    let before = DROPS.load(Ordering::SeqCst);
    let union: ErrorUnion = ErrorUnion::new(Tracked);
    let selected = union.narrow::<(Tracked,), _>().unwrap();
    drop(selected);
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 1);

    let before = DROPS.load(Ordering::SeqCst);
    let union: ErrorUnion<TrackedSet> = ErrorUnion::new(Tracked);
    std::thread::spawn(move || {
        assert!(union.is_inner::<Tracked>());
        drop(union);
    })
    .join()
    .unwrap();
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 1);
}

#[cfg(not(feature = "alloc"))]
#[test]
fn storage_size_is_independent_of_the_concrete_zst_alignment() {
    let words = if cfg!(feature = "location") { 3 } else { 2 };
    assert_eq!(
        size_of::<ErrorUnion<AnyError>>(),
        words * size_of::<usize>()
    );
    assert_eq!(align_of::<ErrorUnion<(Tracked,)>>(), align_of::<usize>());
}
