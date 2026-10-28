#[path = "no_alloc/classification.rs"]
mod classification;

#[test]
fn owned_classification_preserves_known_types_and_replaces_unknown_types() {
    classification::run_checks();
}

#[test]
fn panicking_classifier_drops_the_original_once() {
    use core::{
        error::Error,
        fmt,
        sync::atomic::{AtomicUsize, Ordering},
    };
    static DROPS: AtomicUsize = AtomicUsize::new(0);
    #[derive(Debug)]
    struct Root;
    impl fmt::Display for Root {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("root")
        }
    }
    impl Error for Root {}
    impl Drop for Root {
        fn drop(&mut self) {
            DROPS.fetch_add(1, Ordering::SeqCst);
        }
    }
    let error: eros::ErrorUnion = eros::ErrorUnion::new(Root);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        error.classify::<(fmt::Error,), _, _>(|_| -> fmt::Error { panic!("classification failed") })
    }));
    assert!(outcome.is_err());
    assert_eq!(DROPS.load(Ordering::SeqCst), 1);
}

#[cfg(feature = "alloc")]
#[test]
fn panicking_boxing_callback_drops_original_and_context_once() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[derive(Debug, thiserror::Error)]
    #[error("tracked")]
    struct Tracked(Arc<AtomicUsize>);
    impl Drop for Tracked {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    let roots = Arc::new(AtomicUsize::new(0));
    let error: eros::ErrorUnion = eros::ErrorUnion::new(Tracked(roots.clone()));
    #[cfg(feature = "context")]
    let contexts = Arc::new(AtomicUsize::new(0));
    #[cfg(feature = "context")]
    let error = error.context(Box::new(Tracked(contexts.clone())) as Box<dyn eros::SendSyncError>);

    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        error.classify::<(core::fmt::Error,), _, _>(|inner| -> core::fmt::Error {
            let boxed = inner.into_boxed();
            assert!((boxed.as_ref() as &dyn std::any::Any).is::<Tracked>());
            panic!("boxed classification failed");
        })
    }));
    assert!(outcome.is_err());
    assert_eq!(roots.load(Ordering::SeqCst), 1);
    #[cfg(feature = "context")]
    assert_eq!(contexts.load(Ordering::SeqCst), 1);
}

#[test]
fn panicking_original_destructor_drops_replacement_and_saved_context_once() {
    use core::{
        error::Error,
        fmt,
        sync::atomic::{AtomicUsize, Ordering},
    };
    static REPLACEMENTS: AtomicUsize = AtomicUsize::new(0);
    #[cfg(feature = "context")]
    static CONTEXTS: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug)]
    struct Original;
    impl fmt::Display for Original {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("original")
        }
    }
    impl Error for Original {}
    impl Drop for Original {
        fn drop(&mut self) {
            panic!("original destructor failed");
        }
    }

    #[derive(Debug)]
    struct Tracked(&'static AtomicUsize);
    impl fmt::Display for Tracked {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("tracked")
        }
    }
    impl Error for Tracked {}
    impl Drop for Tracked {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    let error: eros::ErrorUnion = eros::ErrorUnion::new(Original);
    #[cfg(feature = "context")]
    let error = error.context(Box::new(Tracked(&CONTEXTS)) as Box<dyn eros::SendSyncError>);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        error.classify::<(Tracked,), _, _>(|original| {
            let replacement = Tracked(&REPLACEMENTS);
            drop(original);
            replacement
        })
    }));
    assert!(outcome.is_err());
    assert_eq!(REPLACEMENTS.load(Ordering::SeqCst), 1);
    #[cfg(feature = "context")]
    assert_eq!(CONTEXTS.load(Ordering::SeqCst), 1);
}

#[test]
fn an_owner_can_escape_the_callback_and_transfer_to_another_thread() {
    use core::{
        error::Error,
        fmt,
        sync::atomic::{AtomicUsize, Ordering},
    };
    static FIRST: AtomicUsize = AtomicUsize::new(0);
    static SECOND: AtomicUsize = AtomicUsize::new(0);
    #[derive(Debug)]
    struct Owner(&'static AtomicUsize);
    impl fmt::Display for Owner {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("owner")
        }
    }
    impl Error for Owner {}
    impl Drop for Owner {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    fn assert_traits<T: Send + Sync + 'static>() {}
    assert_traits::<eros::InnerError>();
    let error: eros::ErrorUnion = eros::ErrorUnion::new(Owner(&FIRST));
    let mut saved = None;
    let classified: eros::ErrorUnion<(fmt::Error,)> = error.classify(|inner| {
        saved = Some(inner);
        fmt::Error
    });
    drop(classified);
    assert_eq!(FIRST.load(Ordering::SeqCst), 0);
    std::thread::spawn(move || {
        let mut inner = saved.unwrap().downcast::<usize>().unwrap_err();
        inner.downcast_mut::<Owner>().unwrap().0 = &SECOND;
        let owner = inner.downcast::<Owner>().unwrap();
        assert!(core::ptr::eq(owner.0, &SECOND));
        drop(owner);
    })
    .join()
    .unwrap();
    assert_eq!(FIRST.load(Ordering::SeqCst), 0);
    assert_eq!(SECOND.load(Ordering::SeqCst), 1);
}

#[test]
fn aligned_zsts_can_be_inspected_and_extracted_from_an_owner() {
    use core::{
        error::Error,
        fmt,
        sync::atomic::{AtomicUsize, Ordering},
    };
    static DROPS: AtomicUsize = AtomicUsize::new(0);
    #[derive(Debug)]
    #[repr(align(256))]
    struct Aligned;
    impl fmt::Display for Aligned {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("aligned")
        }
    }
    impl Error for Aligned {}
    impl Drop for Aligned {
        fn drop(&mut self) {
            DROPS.fetch_add(1, Ordering::SeqCst);
        }
    }
    let error: eros::ErrorUnion = eros::ErrorUnion::new(Aligned);
    let classified: eros::ErrorUnion<(fmt::Error,)> = error.classify(|mut inner| {
        assert_eq!(
            core::ptr::from_ref(inner.downcast_ref::<Aligned>().unwrap()).addr() % 256,
            0
        );
        assert_eq!(
            core::ptr::from_mut(inner.downcast_mut::<Aligned>().unwrap()).addr() % 256,
            0
        );
        let original = inner.downcast::<Aligned>().unwrap();
        assert_eq!(DROPS.load(Ordering::SeqCst), 0);
        drop(original);
        fmt::Error
    });
    drop(classified);
    assert_eq!(DROPS.load(Ordering::SeqCst), 1);
}
