// Exercise public APIs under Miri in both boxed and inline storage configurations.
// The payloads include padding, owned pointers, atomics, and unusual alignments.
use eros::ErrorUnion;
use std::{
    fmt,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
struct Payload<T>(T);
impl<T: fmt::Debug> fmt::Display for Payload<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl<T: fmt::Debug> std::error::Error for Payload<T> {}

#[test]
fn concurrent_atomic_access_and_cross_thread_owner_transfer() {
    let error: ErrorUnion<(Payload<AtomicUsize>,)> = ErrorUnion::new(Payload(AtomicUsize::new(0)));
    std::thread::scope(|scope| {
        for _ in 0..2 {
            scope.spawn(|| {
                for _ in 0..10 {
                    let first = error.as_single();
                    let second = error.downcast_inner_ref::<Payload<AtomicUsize>>().unwrap();
                    assert!(error.is_inner::<Payload<AtomicUsize>>());
                    first.0.fetch_add(1, Ordering::SeqCst);
                    second.0.fetch_add(1, Ordering::SeqCst);
                }
            });
        }
    });
    let owner = std::thread::spawn(move || error.into_inner())
        .join()
        .unwrap();
    let value = owner.downcast::<Payload<AtomicUsize>>().unwrap();
    assert_eq!(value.0.into_inner(), 40);
}

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
struct Padded(u8, u16);

#[test]
fn partially_initialized_word_survives_moves_and_mapping() {
    let error: ErrorUnion<(Payload<Padded>,)> = ErrorUnion::new(Payload(Padded(3, 65530)));
    let mut values = vec![error.erase()];
    values.reserve(32);
    let mut owner = values.pop().unwrap().into_inner();
    owner.downcast_mut::<Payload<Padded>>().unwrap().0.1 += 1;
    assert_eq!(
        owner.downcast::<Payload<Padded>>().unwrap().0,
        Padded(3, 65531)
    );
    let error: ErrorUnion<(Payload<bool>,)> = ErrorUnion::new(Payload(true));
    let error = error.map_single(|value| Payload(if value.0 { usize::MAX } else { 0 }));
    assert_eq!(error.into_single().0, usize::MAX);
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq)]
#[repr(align(64))]
struct Aligned(u8);

#[cfg(feature = "alloc")]
fn map_pair<T, U>(from: T, to: U)
where
    T: fmt::Debug + Clone + PartialEq + Send + Sync + 'static,
    U: fmt::Debug + Clone + PartialEq + Send + Sync + 'static,
{
    for extraction in 0..4 {
        let root: ErrorUnion<(Payload<T>,)> = ErrorUnion::new(Payload(from.clone()));
        let root = root.context("mapping audit");
        let mut mapped = root.map_single(|old| {
            assert_eq!(old.0, from);
            Payload(to.clone())
        });
        assert_eq!(&mapped.as_single().0, &to);
        mapped.as_single_mut().0 = to.clone();
        assert!(mapped.is_inner::<Payload<U>>());
        match extraction {
            0 => drop(mapped),
            1 => assert_eq!(mapped.into_single().0, to),
            2 => assert_eq!(mapped.into_inner().downcast::<Payload<U>>().unwrap().0, to),
            _ => {
                let boxed = mapped.into_inner().into_boxed();
                let any = boxed.as_ref() as &dyn std::any::Any;
                assert_eq!(&any.downcast_ref::<Payload<U>>().unwrap().0, &to);
            }
        }
    }
}

#[cfg(feature = "alloc")]
fn map_from<T>(from: T)
where
    T: fmt::Debug + Clone + PartialEq + Send + Sync + 'static,
{
    map_pair(from.clone(), ());
    map_pair(from.clone(), 241u8);
    map_pair(from.clone(), Padded(7, 513));
    map_pair(from.clone(), Box::new(89usize));
    map_pair(from.clone(), [19u8; 9]);
    map_pair(from.clone(), (Box::new(73usize), Box::new(97usize)));
    map_pair(from.clone(), [43u8; 64]);
    map_pair(from, Aligned(137));
}

#[cfg(feature = "alloc")]
#[test]
fn mapping_size_alignment_and_pointer_matrix() {
    map_from(());
    map_from(241u8);
    map_from(Padded(7, 513));
    map_from(Box::new(89usize));
    map_from([19u8; 9]);
    map_from((Box::new(73usize), Box::new(97usize)));
    map_from([43u8; 64]);
    map_from(Aligned(137));
}
