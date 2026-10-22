#[path = "no_alloc/inline.rs"]
mod inline;

#[derive(Debug)]
struct Tracked(std::sync::Arc<std::sync::atomic::AtomicUsize>);
impl std::fmt::Display for Tracked {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("tracked inline root")
    }
}
impl std::error::Error for Tracked {}
impl Drop for Tracked {
    fn drop(&mut self) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

#[test]
fn panicking_inline_mapping_drops_the_root_once() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let drops = Arc::new(AtomicUsize::new(0));
    assert_eq!(size_of::<Tracked>(), size_of::<usize>());
    let error: eros::ErrorUnion<(Tracked,)> = eros::ErrorUnion::new(Tracked(drops.clone()));
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        error.map_single(|_root| -> std::fmt::Error { panic!("inline mapping failed") })
    }));
    assert_eq!(
        outcome.unwrap_err().downcast_ref::<&str>(),
        Some(&"inline mapping failed")
    );
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[cfg(feature = "context")]
#[test]
fn panicking_context_cleanup_drops_an_extracted_inline_root_once() {
    use std::{
        error::Error,
        fmt,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };
    #[derive(Debug)]
    struct PanickingContext;
    impl fmt::Display for PanickingContext {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("panic")
        }
    }
    impl Error for PanickingContext {}
    impl Drop for PanickingContext {
        fn drop(&mut self) {
            panic!("context cleanup failed");
        }
    }
    for extract in [
        |error: eros::ErrorUnion<(Tracked,)>| drop(error.into_single()),
        |error: eros::ErrorUnion<(Tracked,)>| drop(error.into_inner()),
    ] {
        let drops = Arc::new(AtomicUsize::new(0));
        let error: eros::ErrorUnion<(Tracked,)> = eros::ErrorUnion::new(Tracked(drops.clone()));
        let error = error.context(Box::new(PanickingContext) as Box<dyn eros::SendSyncError>);
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| extract(error)));
        assert_eq!(
            outcome.unwrap_err().downcast_ref::<&str>(),
            Some(&"context cleanup failed")
        );
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn inline_mutable_references_remain_valid_after_moves_and_downcasts() {
    use eros::ErrorUnion;
    use std::{error::Error, fmt};

    #[derive(Debug)]
    struct Reference(&'static mut usize);
    impl fmt::Display for Reference {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl Error for Reference {}
    let reference = Box::leak(Box::new(41));
    let mut error: ErrorUnion<(Reference,)> = ErrorUnion::new(Reference(reference));
    *error.as_single_mut().0 += 1;
    let mut erased: ErrorUnion = error.into_std_error().into_union().into();
    *erased.downcast_inner_mut::<Reference>().unwrap().0 += 1;
    let error = erased.narrow::<(Reference,), _>().unwrap().into_single();
    assert_eq!(*error.0, 43);
    // Reclaim the deliberately leaked allocation after transferring its unique
    // reference through inline storage. Miri checks for leaks and aliasing.
    unsafe {
        drop(Box::from_raw(error.0 as *mut usize));
    }
}

#[cfg(not(feature = "backtrace"))]
mod allocation_checks {
    #[cfg(feature = "alloc")]
    use eros::SendSyncError;
    use eros::{ErrorUnion, IntoAnyUnion, IntoUnion, ReshapeUnion};
    use std::{
        alloc::{GlobalAlloc, Layout, System},
        cell::Cell,
        error::Error,
        fmt,
    };

    std::thread_local! {
        static COUNTS: Cell<Option<(usize, usize)>> = const { Cell::new(None) };
    }
    struct CountingAllocator;
    unsafe impl GlobalAlloc for CountingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let result = unsafe { System.alloc(layout) };
            let _ = COUNTS.try_with(|counts| {
                if let Some((allocs, frees)) = counts.get() {
                    counts.set(Some((allocs + 1, frees)));
                }
            });
            result
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            let _ = COUNTS.try_with(|counts| {
                if let Some((allocs, frees)) = counts.get() {
                    counts.set(Some((allocs, frees + 1)));
                }
            });
            unsafe { System.dealloc(ptr, layout) };
        }
    }
    #[global_allocator]
    static ALLOCATOR: CountingAllocator = CountingAllocator;

    fn measured(f: impl FnOnce()) -> (usize, usize) {
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                COUNTS.with(|counts| counts.set(None));
            }
        }
        COUNTS.with(|counts| {
            assert!(counts.get().is_none());
            counts.set(Some((0, 0)));
        });
        let _reset = Reset;
        f();
        COUNTS.with(|counts| counts.get().unwrap())
    }
    #[derive(Debug)]
    struct Small(usize);
    impl fmt::Display for Small {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl Error for Small {}

    #[test]
    fn small_roots_and_composing_operations_do_not_allocate() {
        let counts = measured(|| {
            let input: eros::Result<(), (Small, fmt::Error)> = Err(Small(usize::MAX)).union();
            let erased = input.any_union().unwrap_err();
            let narrowed = erased.narrow::<(Small,), _>().unwrap();
            assert_eq!(narrowed.as_single().0, usize::MAX);
            let widened: ErrorUnion<(fmt::Error, Small)> = narrowed.widen();
            let result: eros::Result<(), (fmt::Error, Small)> = Err(widened);
            result
                .recover::<Small, _>(|error| assert_eq!(error.into_single().0, usize::MAX))
                .recover::<fmt::Error, _>(|_| panic!("wrong variant"))
                .into_value();
        });
        assert_eq!(counts, (0, 0));
    }

    #[cfg(not(feature = "alloc"))]
    #[test]
    fn literal_message_errors_and_recovery_do_not_allocate() {
        assert_eq!(
            measured(|| {
                fn operation() -> eros::Result<(), (eros::MsgError,)> {
                    eros::bail!("message {{text}}");
                }
                let result: eros::Result<(), (fmt::Error, eros::MsgError)> = operation().widen();
                let erased = result.any_union().unwrap_err();
                let error = erased.narrow::<(eros::MsgError,), _>().unwrap();
                assert_eq!(error.as_single().as_str(), "message {text}");
                drop(error.into_std_error());
                let error = eros::error!("another message");
                assert_eq!(
                    error.downcast_inner::<eros::MsgError>().unwrap().as_str(),
                    "another message"
                );
            }),
            (0, 0)
        );
    }

    #[cfg(feature = "alloc")]
    #[derive(Debug)]
    struct Large([usize; 2]);
    #[cfg(feature = "alloc")]
    impl fmt::Display for Large {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    #[cfg(feature = "alloc")]
    impl Error for Large {}

    #[cfg(feature = "alloc")]
    #[test]
    fn large_roots_allocate_once_and_boxed_extraction_reuses_the_heap() {
        let counts = measured(|| {
            let error: ErrorUnion = ErrorUnion::new(Large([17, 29]));
            let address = error.inner() as *const dyn SendSyncError as *const ();
            let error = error.narrow::<(Large,), _>().unwrap();
            let boxed = error.into_inner();
            assert_eq!(
                boxed.as_ref() as *const dyn SendSyncError as *const (),
                address
            );
            assert_eq!(
                (boxed.as_ref() as &dyn std::any::Any)
                    .downcast_ref::<Large>()
                    .unwrap()
                    .0,
                [17, 29]
            );
            drop(boxed);
        });
        assert_eq!(counts, (1, 1));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn inline_boxed_extraction_allocates_and_mapping_switches_both_ways() {
        assert_eq!(
            measured(|| {
                let error: ErrorUnion<(Small,)> = ErrorUnion::new(Small(42));
                let boxed = error.into_inner();
                assert_eq!(
                    (boxed.as_ref() as &dyn std::any::Any)
                        .downcast_ref::<Small>()
                        .unwrap()
                        .0,
                    42
                );
                drop(boxed);
            }),
            (1, 1)
        );
        assert_eq!(
            measured(|| {
                let error: ErrorUnion<(Small,)> = ErrorUnion::new(Small(42));
                let large = error.map_single(|small| Large([small.0, 91]));
                let small = large.map_single(|large| Small(large.0[0] + large.0[1]));
                assert_eq!(small.into_single().0, 133);
            }),
            (1, 1)
        );
        assert_eq!(
            measured(|| {
                let error: ErrorUnion<(Small,)> = ErrorUnion::new(Small(42));
                let mapped = error.map_inner(|boxed| {
                    Small(
                        (boxed.as_ref() as &dyn std::any::Any)
                            .downcast_ref::<Small>()
                            .unwrap()
                            .0
                            + 1,
                    )
                });
                assert_eq!(mapped.into_single().0, 43);
            }),
            (1, 1)
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn over_aligned_roots_fall_back_to_an_aligned_box() {
        #[derive(Debug)]
        #[repr(align(256))]
        struct Aligned(u8);
        impl fmt::Display for Aligned {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
        impl Error for Aligned {}
        assert_eq!(
            measured(|| {
                let error: ErrorUnion<(Aligned,)> = ErrorUnion::new(Aligned(19));
                assert_eq!((error.as_single() as *const Aligned).addr() % 256, 0);
                assert_eq!(error.into_single().0, 19);
            }),
            (1, 1)
        );
    }
}
