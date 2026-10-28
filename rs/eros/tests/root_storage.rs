#[test]
fn union_handle_obeys_the_feature_size_limit() {
    use eros::{AnyError, ErrorUnion, ErrorUnionAdapter};
    let words = if cfg!(all(not(feature = "alloc"), feature = "error_location")) {
        3
    } else {
        2
    };
    type Typed = (core::fmt::Error, std::io::Error);
    assert_eq!(
        size_of::<ErrorUnion<AnyError>>(),
        words * size_of::<usize>()
    );
    assert_eq!(size_of::<ErrorUnion<Typed>>(), words * size_of::<usize>());
    assert_eq!(
        size_of::<ErrorUnionAdapter<Typed>>(),
        words * size_of::<usize>()
    );
    assert_eq!(
        size_of::<Option<ErrorUnion<Typed>>>(),
        words * size_of::<usize>()
    );
    assert_eq!(
        size_of::<eros::Result<(), Typed>>(),
        words * size_of::<usize>()
    );
}

#[path = "common/payloads.rs"]
mod payloads;

#[cfg(feature = "alloc")]
#[derive(Debug)]
struct Tracked(std::sync::Arc<std::sync::atomic::AtomicUsize>);
#[cfg(feature = "alloc")]
impl std::fmt::Display for Tracked {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("tracked root")
    }
}
#[cfg(feature = "alloc")]
impl std::error::Error for Tracked {}
#[cfg(feature = "alloc")]
impl Drop for Tracked {
    fn drop(&mut self) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}

#[cfg(feature = "alloc")]
#[test]
fn panicking_mapping_drops_the_root_once() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let drops = Arc::new(AtomicUsize::new(0));
    assert_eq!(size_of::<Tracked>(), size_of::<usize>());
    let error: eros::ErrorUnion<(Tracked,)> = eros::ErrorUnion::new(Tracked(drops.clone()));
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        error.map_single(|_root| -> std::fmt::Error { panic!("mapping failed") })
    }));
    assert_eq!(
        outcome.unwrap_err().downcast_ref::<&str>(),
        Some(&"mapping failed")
    );
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[cfg(feature = "context")]
#[test]
fn panicking_context_cleanup_drops_root_and_container_once() {
    use std::{
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
    impl std::error::Error for PanickingContext {}
    impl Drop for PanickingContext {
        fn drop(&mut self) {
            panic!("context cleanup failed");
        }
    }
    for extract in [
        |error: eros::ErrorUnion<(Tracked,)>| drop(error),
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

#[cfg(feature = "alloc")]
#[test]
fn mutable_references_remain_valid_after_moves_and_downcasts() {
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
    let mut erased: ErrorUnion = eros::ErrorUnionAdapter::from(error).union().into();
    *erased.downcast_inner_mut::<Reference>().unwrap().0 += 1;
    let error = erased.narrow::<(Reference,), _>().unwrap().into_single();
    assert_eq!(*error.0, 43);
    // Reclaim the deliberately leaked allocation after transferring its unique
    // reference through root storage. Miri checks for leaks and aliasing.
    unsafe {
        drop(Box::from_raw(error.0 as *mut usize));
    }
}

#[cfg(not(feature = "backtrace"))]
mod allocation_checks {
    use eros::Erase;
    #[cfg(feature = "alloc")]
    use eros::{ErrorUnion, IntoUnion, ReshapeUnion};
    #[cfg(feature = "alloc")]
    use std::error::Error;
    use std::{
        alloc::{GlobalAlloc, Layout, System},
        cell::Cell,
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

    #[cfg(feature = "alloc")]
    const SMALL_ALLOCS: usize = if cfg!(any(
        feature = "context",
        feature = "backtrace",
        feature = "error_location",
        feature = "error_span_trace"
    )) {
        1
    } else {
        0
    };

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
    #[cfg(feature = "alloc")]
    #[derive(Debug)]
    struct Small(usize);
    #[cfg(feature = "alloc")]
    impl fmt::Display for Small {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    #[cfg(feature = "alloc")]
    impl Error for Small {}

    #[cfg(feature = "alloc")]
    #[test]
    fn small_roots_allocate_only_with_diagnostics_and_reshaping_reuses_storage() {
        let counts = measured(|| {
            let input: eros::Result<(), (Small, fmt::Error)> = Err(Small(usize::MAX)).union();
            let erased = input.erase().unwrap_err();
            let narrowed = erased.narrow::<(Small,), _>().unwrap();
            assert_eq!(narrowed.as_single().0, usize::MAX);
            let widened: ErrorUnion<(fmt::Error, Small)> = narrowed.widen();
            let result: eros::Result<(), (fmt::Error, Small)> = Err(widened);
            result
                .recover::<Small, _>(|error| assert_eq!(error.into_single().0, usize::MAX))
                .recover::<fmt::Error, _>(|_| panic!("wrong variant"))
                .into_value();
        });
        assert_eq!(counts, (SMALL_ALLOCS, SMALL_ALLOCS));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn matching_layout_mapping_reuses_the_allocation_and_updates_operations() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static DROPS: AtomicUsize = AtomicUsize::new(0);
        #[derive(Debug)]
        struct Replacement(&'static AtomicUsize);
        impl fmt::Display for Replacement {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("replacement")
            }
        }
        impl Error for Replacement {}
        impl Drop for Replacement {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        assert_eq!(
            measured(|| {
                let error: ErrorUnion<(Small,)> = ErrorUnion::new(Small(42));
                let original = error.as_single() as *const Small as *const ();
                let mut mapped = error.map_single(|old| {
                    assert_eq!(old.0, 42);
                    Replacement(&DROPS)
                });
                if SMALL_ALLOCS != 0 {
                    assert_eq!(
                        mapped.as_single() as *const Replacement as *const (),
                        original
                    );
                }
                assert!(mapped.is_inner::<Replacement>());
                assert!(!mapped.is_inner::<Small>());
                assert!(mapped.downcast_inner_ref::<Small>().is_none());
                assert!(mapped.downcast_inner_mut::<Small>().is_none());
                assert!(mapped.downcast_inner_mut::<Replacement>().is_some());
                drop(mapped);
            }),
            (SMALL_ALLOCS, SMALL_ALLOCS)
        );
        assert_eq!(DROPS.load(Ordering::SeqCst), 1);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn zero_sized_roots_allocate_only_with_diagnostics() {
        assert_eq!(
            measured(|| {
                let error: ErrorUnion<(fmt::Error,)> = ErrorUnion::new(fmt::Error);
                assert_eq!(error.into_single(), fmt::Error);
            }),
            (SMALL_ALLOCS, SMALL_ALLOCS)
        );
        assert_eq!(
            measured(|| {
                let error: ErrorUnion<(fmt::Error,)> = ErrorUnion::new(fmt::Error);
                let boxed = error.into_inner();
                assert!((boxed.as_ref() as &dyn std::any::Any).is::<fmt::Error>());
            }),
            (SMALL_ALLOCS, SMALL_ALLOCS)
        );
    }

    #[cfg(not(feature = "alloc"))]
    #[test]
    fn literal_message_errors_and_recovery_do_not_allocate() {
        assert_eq!(
            measured(|| {
                fn operation() -> eros::Result<()> {
                    eros::bail!("message {{text}}");
                }
                let error = operation().erase().unwrap_err();
                assert_eq!(size_of_val(error.inner()), 0);
                let error = error.narrow::<(fmt::Error,), _>().unwrap_err();
                drop(eros::ErrorUnionAdapter::from(error));
                let mut error = eros::error!("another message");
                assert_eq!(size_of_val(error.inner_mut()), 0);
                assert!(!error.is_inner::<eros::MsgError>());
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
    fn padded_payload_mapping_reuses_the_complete_layout() {
        #[derive(Debug)]
        struct Byte(u8);
        impl fmt::Display for Byte {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
        impl Error for Byte {}
        assert_ne!(size_of::<Byte>(), size_of::<Small>());
        assert_eq!(
            measured(|| {
                let error: ErrorUnion<(Byte,)> = ErrorUnion::new(Byte(17));
                let address = error.as_single() as *const Byte as *const ();
                let error = error.map_single(|byte| Small(usize::from(byte.0)));
                if SMALL_ALLOCS != 0 {
                    assert_eq!(error.as_single() as *const Small as *const (), address);
                }
                assert_eq!(error.into_single().0, 17);
            }),
            (SMALL_ALLOCS, SMALL_ALLOCS)
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn mapping_between_large_errors_keeps_the_allocation() {
        #[derive(Debug)]
        struct OtherLarge([usize; 2]);
        impl fmt::Display for OtherLarge {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }
        impl Error for OtherLarge {}
        assert_eq!(
            measured(|| {
                let error: ErrorUnion<(Large,)> = ErrorUnion::new(Large([17, 29]));
                let address = error.as_single() as *const Large as *const ();
                let mapped = error.map_single(|old| OtherLarge([old.0[0] + 1, old.0[1]]));
                assert_eq!(
                    mapped.as_single() as *const OtherLarge as *const (),
                    address
                );
                assert!(!mapped.is_inner::<Large>());
                assert!(mapped.is_inner::<OtherLarge>());
                assert_eq!(mapped.into_single().0, [18, 29]);
            }),
            (1, 1)
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn boxed_extraction_moves_the_original_concrete_type() {
        let counts = measured(|| {
            let error: ErrorUnion = ErrorUnion::new(Large([17, 29]));
            let error = error.narrow::<(Large,), _>().unwrap();
            let boxed = error.into_inner();
            assert_eq!(
                (boxed.as_ref() as &dyn std::any::Any)
                    .downcast_ref::<Large>()
                    .unwrap()
                    .0,
                [17, 29]
            );
            drop(boxed);
        });
        assert_eq!(counts, (1 + SMALL_ALLOCS, 1 + SMALL_ALLOCS));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn boxed_extraction_and_mapping_have_expected_allocation_counts() {
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
            (1 + SMALL_ALLOCS, 1 + SMALL_ALLOCS)
        );
        assert_eq!(
            measured(|| {
                let error: ErrorUnion<(Small,)> = ErrorUnion::new(Small(42));
                let large = error.map_single(|small| Large([small.0, 91]));
                let small = large.map_single(|large| Small(large.0[0] + large.0[1]));
                assert_eq!(small.into_single().0, 133);
            }),
            (1 + 2 * SMALL_ALLOCS, 1 + 2 * SMALL_ALLOCS)
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
            (1 + 2 * SMALL_ALLOCS, 1 + 2 * SMALL_ALLOCS)
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

#[cfg(feature = "alloc")]
#[test]
fn matching_layout_mapping_keeps_diagnostics_in_place() {
    use eros::ErrorUnion;
    let error: ErrorUnion<(core::fmt::Error,)> = ErrorUnion::new(core::fmt::Error);
    let error = error.context("saved");
    #[cfg(feature = "backtrace")]
    let backtrace = error.backtrace() as *const _;
    #[cfg(feature = "error_span_trace")]
    let span_trace = error.span_trace() as *const _;
    #[cfg(feature = "error_location")]
    let location = error.location();
    #[cfg(feature = "context")]
    let frame = error.contexts().next().unwrap() as *const _;
    let mapped = error.map_single(|error| error);
    #[cfg(feature = "backtrace")]
    assert!(core::ptr::eq(mapped.backtrace(), backtrace));
    #[cfg(feature = "error_span_trace")]
    assert!(core::ptr::eq(mapped.span_trace(), span_trace));
    #[cfg(feature = "error_location")]
    assert!(core::ptr::eq(mapped.location(), location));
    #[cfg(feature = "context")]
    assert!(core::ptr::eq(mapped.contexts().next().unwrap(), frame));
    assert_eq!(mapped.into_single(), core::fmt::Error);
}
