use eros::{ContextSource, ErrorUnion, SendSyncError};
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
    error.context(ContextSource::Error(context))
}

#[test]
fn owned_extraction_moves_the_root_and_drops_metadata_exactly_once() {
    for extract in [
        |error: ErrorUnion<(Tracked,)>| error.into_single(),
        |error: ErrorUnion<(Tracked,)>| error.narrow::<Tracked, _>().unwrap(),
        |error: ErrorUnion<(Tracked,)>| error.downcast_inner::<Tracked>().unwrap(),
        |error: ErrorUnion<(Tracked,)>| {
            let eros::E1::A(value) = error.to_enum();
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
    let error = union(&root, &context).map(|mut error| {
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
        union(&root, &context).map(|_| -> Tracked { panic!("mapping failed") })
    }));
    assert!(outcome.is_err());
    assert_eq!(root.load(Ordering::SeqCst), 1);
    assert_eq!(context.load(Ordering::SeqCst), 1);
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
