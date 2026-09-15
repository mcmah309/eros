use eros::{AnyError, ContextValue, ErrorUnion, SendSyncError};
use std::{
    fmt,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[derive(Debug)]
#[repr(align(128))]
struct Tracked {
    payload: Vec<u8>,
    drops: Arc<AtomicUsize>,
}

impl fmt::Display for Tracked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.payload)
    }
}
impl std::error::Error for Tracked {}
impl Drop for Tracked {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

fn tracked(drops: &Arc<AtomicUsize>) -> Tracked {
    Tracked {
        payload: vec![1, 2, 3],
        drops: drops.clone(),
    }
}

fn union(root: &Arc<AtomicUsize>, context: &Arc<AtomicUsize>) -> ErrorUnion<(Tracked,)> {
    let error: ErrorUnion<(Tracked,)> = ErrorUnion::new(tracked(root));
    let context: Box<dyn SendSyncError> = Box::new(tracked(context));
    error.context(ContextValue::from(context))
}

#[test]
fn owned_extraction_moves_the_root_and_drops_metadata_exactly_once() {
    for extract in [
        |error: ErrorUnion<(Tracked,)>| error.into_single(),
        |error: ErrorUnion<(Tracked,)>| error.narrow::<Tracked, _>().unwrap(),
        |error: ErrorUnion<(Tracked,)>| error.downcast_inner::<Tracked>().unwrap(),
        |error: ErrorUnion<(Tracked,)>| {
            let eros::E1::A(value) = error.into_enum();
            value
        },
    ] {
        let root = Arc::new(AtomicUsize::new(0));
        let context = Arc::new(AtomicUsize::new(0));
        let value = extract(union(&root, &context));
        assert_eq!(value.payload, [1, 2, 3]);
        assert_eq!(root.load(Ordering::SeqCst), 0);
        assert_eq!(context.load(Ordering::SeqCst), 1);
        drop(value);
        assert_eq!(root.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn mapping_preserves_metadata_until_the_new_union_is_dropped() {
    let root = Arc::new(AtomicUsize::new(0));
    let context = Arc::new(AtomicUsize::new(0));
    let error = union(&root, &context).map_single(|mut error| {
        error.payload.push(4);
        error
    });
    assert_eq!(error.payload, [1, 2, 3, 4]);
    assert_eq!(root.load(Ordering::SeqCst), 0);
    assert_eq!(
        context.load(Ordering::SeqCst),
        usize::from(!cfg!(feature = "context"))
    );
    let inner = error.into_inner();
    assert_eq!(context.load(Ordering::SeqCst), 1);
    assert_eq!(root.load(Ordering::SeqCst), 0);
    assert!(inner.as_ref().as_any().is::<Tracked>());
    drop(inner);
    assert_eq!(root.load(Ordering::SeqCst), 1);
}

#[test]
fn panicking_map_drops_both_root_and_context() {
    let root = Arc::new(AtomicUsize::new(0));
    let context = Arc::new(AtomicUsize::new(0));
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        union(&root, &context).map_single(|_| -> Tracked { panic!("mapping failed") })
    }));
    assert!(outcome.is_err());
    assert_eq!(root.load(Ordering::SeqCst), 1);
    assert_eq!(context.load(Ordering::SeqCst), 1);
}

#[test]
fn both_subset_branches_keep_the_error_and_context_alive_until_drop() {
    for partition in [
        |error: ErrorUnion<(fmt::Error, Tracked)>| error.subset::<(Tracked,), _>().unwrap(),
        |error: ErrorUnion<(fmt::Error, Tracked)>| error.subset::<(fmt::Error,), _>().unwrap_err(),
    ] {
        let root = Arc::new(AtomicUsize::new(0));
        let context = Arc::new(AtomicUsize::new(0));
        let error = union(&root, &context).widen();
        let error = partition(error);
        assert_eq!(error.payload, [1, 2, 3]);
        assert_eq!(root.load(Ordering::SeqCst), 0);
        assert_eq!(
            context.load(Ordering::SeqCst),
            usize::from(!cfg!(feature = "context"))
        );
        drop(error);
        assert_eq!(root.load(Ordering::SeqCst), 1);
        assert_eq!(context.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn rejected_subset_and_narrow_retain_ownership_until_extraction() {
    let root = Arc::new(AtomicUsize::new(0));
    let context = Arc::new(AtomicUsize::new(0));
    let error: ErrorUnion<(fmt::Error, Tracked)> = union(&root, &context).widen();
    let error = error.subset::<(fmt::Error,), _>().unwrap_err();
    let error: ErrorUnion<(fmt::Error, Tracked)> = error.widen();
    let error = error.narrow::<fmt::Error, _>().unwrap_err();
    assert_eq!(root.load(Ordering::SeqCst), 0);
    assert_eq!(
        context.load(Ordering::SeqCst),
        usize::from(!cfg!(feature = "context"))
    );
    drop(error.into_single());
    assert_eq!(root.load(Ordering::SeqCst), 1);
    assert_eq!(context.load(Ordering::SeqCst), 1);
}

#[test]
fn erased_reshaping_preserves_ownership_until_checked_extraction() {
    let root = Arc::new(AtomicUsize::new(0));
    let context = Arc::new(AtomicUsize::new(0));
    let error: ErrorUnion = union(&root, &context).into();
    let original = error.inner() as *const dyn SendSyncError as *const ();
    let report = format!("{error:?}");

    let error = error.widen::<AnyError, _>();
    // The universal subset matches, but neither branch promises a concrete type.
    let partition: Result<ErrorUnion<AnyError>, ErrorUnion<AnyError>> = error.subset();
    let error = partition.unwrap();
    // An empty subset cannot manufacture an inhabited ErrorUnion<()>.
    let partition: Result<ErrorUnion<()>, ErrorUnion<AnyError>> = error.subset();
    let mut error = partition.unwrap_err();
    assert_eq!(
        error.inner() as *const dyn SendSyncError as *const (),
        original
    );
    assert_eq!(format!("{error:?}"), report);
    assert!(error.downcast_inner_ref::<AnyError>().is_none());
    assert!(error.downcast_inner_mut::<AnyError>().is_none());
    error
        .downcast_inner_mut::<Tracked>()
        .unwrap()
        .payload
        .push(4);
    assert_eq!(root.load(Ordering::SeqCst), 0);
    assert_eq!(
        context.load(Ordering::SeqCst),
        usize::from(!cfg!(feature = "context"))
    );

    let value = error.downcast_inner::<Tracked>().unwrap();
    assert_eq!(value.payload, [1, 2, 3, 4]);
    assert_eq!(root.load(Ordering::SeqCst), 0);
    assert_eq!(context.load(Ordering::SeqCst), 1);
    drop(value);
    assert_eq!(root.load(Ordering::SeqCst), 1);
}

#[test]
fn erased_owned_downcast_cannot_create_an_anyerror_marker() {
    let root = Arc::new(AtomicUsize::new(0));
    let context = Arc::new(AtomicUsize::new(0));
    let error: ErrorUnion = union(&root, &context).into();
    assert!(error.downcast_inner::<AnyError>().is_none());
    assert_eq!(root.load(Ordering::SeqCst), 1);
    assert_eq!(context.load(Ordering::SeqCst), 1);
}

#[test]
fn erased_adapter_cannot_be_recovered_as_a_concrete_or_empty_set() {
    let root = Arc::new(AtomicUsize::new(0));
    let context = Arc::new(AtomicUsize::new(0));
    let error: ErrorUnion = union(&root, &context).into();
    let original = error.inner() as *const dyn SendSyncError as *const ();
    let report = format!("{error:?}");
    let adapter = error.into_dyn_error();
    let adapter_address = &*adapter as *const dyn SendSyncError as *const ();

    // Even a matching payload cannot change the adapter's original set parameter.
    let adapter = ErrorUnion::<(Tracked,)>::try_from_dyn_error(adapter).unwrap_err();
    let adapter = ErrorUnion::<(fmt::Error,)>::try_from_dyn_error(adapter).unwrap_err();
    let adapter = ErrorUnion::<()>::try_from_dyn_error(adapter).unwrap_err();
    assert_eq!(
        &*adapter as *const dyn SendSyncError as *const (),
        adapter_address
    );
    let error = ErrorUnion::<AnyError>::try_from_dyn_error(adapter).unwrap();
    assert_eq!(
        error.inner() as *const dyn SendSyncError as *const (),
        original
    );
    assert_eq!(format!("{error:?}"), report);
    assert_eq!(root.load(Ordering::SeqCst), 0);
    let value = error.downcast_inner::<Tracked>().unwrap();
    assert_eq!(value.payload, [1, 2, 3]);
    assert_eq!(context.load(Ordering::SeqCst), 1);
    drop(value);
    assert_eq!(root.load(Ordering::SeqCst), 1);
}

#[test]
fn typed_adapter_recovery_requires_its_original_set() {
    let root = Arc::new(AtomicUsize::new(0));
    let context = Arc::new(AtomicUsize::new(0));
    let adapter = union(&root, &context).into_dyn_error();
    let adapter = ErrorUnion::<AnyError>::try_from_dyn_error(adapter).unwrap_err();
    let error = ErrorUnion::<(Tracked,)>::try_from_dyn_error(adapter).unwrap();
    // This path uses the singleton's unchecked extraction, so the set must be exact.
    let value = error.into_single();
    assert_eq!(value.payload, [1, 2, 3]);
    assert_eq!(root.load(Ordering::SeqCst), 0);
    assert_eq!(context.load(Ordering::SeqCst), 1);
    drop(value);
    assert_eq!(root.load(Ordering::SeqCst), 1);
}
