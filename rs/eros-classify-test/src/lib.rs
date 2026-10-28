//! A regular consumer enabling features on its core-only dependency's Eros.
/// Calls the core-only library through this crate's allocation-enabled Eros.
pub fn classify(code: usize) -> eros::ErrorUnion<eros_no_alloc_test::classify::Errors> {
    eros_no_alloc_test::classify::classify_error(eros_no_alloc_test::classify::code_error(code))
}

#[cfg(test)]
mod tests {
    use eros::{ErrorUnion, InnerError, ReshapeUnion, SendSyncError};
    use eros_no_alloc_test::classify::{self as library, Code, DomainError};
    use std::{
        error::Error,
        fmt,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };

    #[derive(Debug)]
    struct AppError(Box<dyn SendSyncError>);
    impl fmt::Display for AppError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "application: {}", self.0)
        }
    }
    impl Error for AppError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(self.0.as_ref())
        }
    }

    #[test]
    fn core_only_library_callbacks_work_after_enabling_alloc() {
        library::run_checks();
        // The dependency's signatures are checked explicitly in this consumer.
        let _: fn(InnerError) -> DomainError = library::category;
        let error: ErrorUnion<library::Errors> = super::classify(7);
        assert_eq!(error.narrow::<DomainError, _>().unwrap().0, 8);
    }

    #[test]
    fn downstream_classification_boxes_the_original_concrete_error() {
        let error = library::classify_error(library::code_error(7));
        #[cfg(feature = "diagnostics")]
        let error = error.context("library boundary");
        #[cfg(feature = "diagnostics")]
        let location = error.location();
        #[cfg(feature = "backtrace")]
        let backtrace = error.backtrace().to_string();
        #[cfg(feature = "traces")]
        let trace = error.span_trace().to_string();

        let input: eros::Result<u8, library::Errors> = Err(error);
        let output: eros::Result<u8, (AppError, fmt::Error)> = input.classify(|mut inner| {
            assert!(inner.is::<DomainError>());
            assert!(!inner.is::<Box<DomainError>>());
            inner.downcast_mut::<DomainError>().unwrap().0 += 1;
            assert_eq!(inner.to_string(), "domain 9");
            AppError(inner.into_boxed())
        });
        let error = output.unwrap_err();
        #[cfg(feature = "diagnostics")]
        {
            assert!(core::ptr::eq(location, error.location()));
            assert_eq!(
                error.contexts().next().unwrap().to_string(),
                "library boundary"
            );
        }
        #[cfg(feature = "backtrace")]
        assert_eq!(error.backtrace().to_string(), backtrace);
        #[cfg(feature = "traces")]
        assert_eq!(error.span_trace().to_string(), trace);
        let error = error.narrow::<AppError, _>().unwrap();
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<DomainError>()
                .unwrap()
                .0,
            9
        );
    }

    #[derive(Debug)]
    struct LargeError {
        bytes: [u8; 257],
        drops: Arc<AtomicUsize>,
    }
    impl fmt::Display for LargeError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "large {}", self.bytes[0])
        }
    }
    impl Error for LargeError {}
    impl Drop for LargeError {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }
    fn large(drops: &Arc<AtomicUsize>) -> ErrorUnion {
        ErrorUnion::new(LargeError {
            bytes: [71; 257],
            drops: drops.clone(),
        })
    }

    #[test]
    fn core_library_accepts_foreign_heap_errors_and_drops_them_once() {
        let drops = Arc::new(AtomicUsize::new(0));
        let result = library::classify_result(Err(large(&drops))).unwrap_err();
        assert_eq!(result.narrow::<DomainError, _>().unwrap().0, 0);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn owned_downcasts_keep_original_type_value_and_ownership() {
        let drops = Arc::new(AtomicUsize::new(0));
        let error = large(&drops);
        let error: ErrorUnion<(DomainError,)> = error.classify(|inner| {
            assert!(inner.is::<LargeError>());
            let mut inner = inner.downcast::<Code>().unwrap_err();
            inner.downcast_mut::<LargeError>().unwrap().bytes[0] = 93;
            let original = inner.downcast::<LargeError>().unwrap();
            assert_eq!(original.bytes[0], 93);
            assert_eq!(drops.load(Ordering::SeqCst), 0);
            drop(original);
            DomainError(5)
        });
        assert_eq!(error.into_single().0, 5);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }

    #[derive(Debug)]
    struct Retained(InnerError);
    impl fmt::Display for Retained {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "retained: {}", self.0)
        }
    }
    impl Error for Retained {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(self.0.as_error())
        }
    }

    #[test]
    fn opaque_owners_survive_nested_classification_and_source_wrapping() {
        let drops = Arc::new(AtomicUsize::new(0));
        let error: ErrorUnion<(Retained,)> = large(&drops).classify(Retained);
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        assert!(error.source().unwrap().is::<LargeError>());
        let error: ErrorUnion<(AppError,)> = error.classify(|inner| {
            assert!(
                inner
                    .downcast_ref::<Retained>()
                    .unwrap()
                    .0
                    .is::<LargeError>()
            );
            AppError(inner.into_boxed())
        });
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        let retained = error.source().unwrap().downcast_ref::<Retained>().unwrap();
        assert_eq!(
            retained.0.downcast_ref::<LargeError>().unwrap().bytes[0],
            71
        );
        drop(error);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn unwinding_a_callback_drops_foreign_heap_errors_once() {
        let drops = Arc::new(AtomicUsize::new(0));
        let error = large(&drops);
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            error.classify::<(DomainError,), _, _>(|inner| {
                let inner = inner.downcast::<Code>().unwrap_err();
                assert!(inner.is::<LargeError>());
                panic!("classifier failed");
            })
        }));
        assert!(outcome.is_err());
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}
