#![cfg(feature = "alloc")]

mod common;

use eros::{Erase, ErrorUnion, MsgError, ReshapeUnion, SendSyncError};
use std::{
    fmt, io,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[derive(Debug, thiserror::Error)]
#[error("configuration failed: {0}")]
struct ConfigError(#[source] Box<dyn SendSyncError>);

#[derive(Debug, thiserror::Error)]
#[error("database failed: {0}")]
struct DatabaseError(#[source] Box<dyn SendSyncError>);

#[test]
fn boxed_classifier_runs_once_only_for_implementation_errors() {
    type Domain = (fmt::Error, ConfigError);
    let inputs: [eros::Result<String>; 4] = [
        Ok(String::from("success")),
        Err(ErrorUnion::new(fmt::Error)),
        Err(ErrorUnion::new(MsgError::from("failure"))),
        Err(ErrorUnion::new(ConfigError(Box::new(MsgError::from(
            "already wrapped",
        ))))),
    ];
    for (index, result) in inputs.into_iter().enumerate() {
        let identity = result
            .as_ref()
            .err()
            .map(|error| common::identity(error.inner()));
        let mut calls = 0;
        // Moving a captured String makes this wrapper FnOnce, not FnMut.
        let capture = String::from("captured");
        let mixed: eros::Result<String, Domain> =
            result.classify(|boxed: Box<dyn SendSyncError>| {
                drop(capture);
                calls += 1;
                ConfigError(boxed)
            });
        assert_eq!(calls, usize::from(index == 2));
        match index {
            0 => assert_eq!(mixed.unwrap(), "success"),
            1 | 3 => assert_eq!(
                common::identity(mixed.unwrap_err().inner()),
                identity.unwrap()
            ),
            2 => assert_eq!(
                mixed
                    .unwrap_err()
                    .narrow::<ConfigError, _>()
                    .unwrap()
                    .0
                    .to_string(),
                "failure"
            ),
            _ => unreachable!(),
        }
    }
}

#[test]
fn domain_wrappers_stay_distinct_when_paths_are_merged_and_erased() {
    type Combined = (fmt::Error, ConfigError, DatabaseError);
    let inputs: [ErrorUnion; 2] = [
        ErrorUnion::new(io::Error::other("first path")),
        ErrorUnion::new(io::Error::other("second path")),
    ];
    for (index, error) in inputs.into_iter().enumerate() {
        let error = error.context("operation");
        #[cfg(feature = "error_location")]
        let location = error.location();
        #[cfg(feature = "backtrace")]
        let backtrace = error.backtrace().to_string();
        #[cfg(feature = "error_span_trace")]
        let span_trace = error.span_trace().to_string();
        #[cfg(feature = "context")]
        let contexts: Vec<_> = error.contexts().map(|frame| frame.to_string()).collect();
        let root = core::any::Any::type_id(error.inner());
        let error: ErrorUnion<Combined> = match index {
            0 => error
                .classify::<(fmt::Error, ConfigError), _, _, _>(ConfigError)
                .widen(),
            1 => error
                .classify::<(DatabaseError, fmt::Error), _, _, _>(DatabaseError)
                .widen(),
            _ => unreachable!(),
        };
        #[cfg(feature = "error_location")]
        assert!(std::ptr::eq(error.location(), location));
        #[cfg(feature = "backtrace")]
        assert_eq!(error.backtrace().to_string(), backtrace);
        #[cfg(feature = "error_span_trace")]
        assert_eq!(error.span_trace().to_string(), span_trace);
        #[cfg(feature = "context")]
        assert_eq!(
            error
                .contexts()
                .map(|frame| frame.to_string())
                .collect::<Vec<_>>(),
            contexts
        );
        let error = error.erase();
        match index {
            0 => {
                let error = error.narrow::<ConfigError, _>().unwrap();
                assert_eq!(core::any::Any::type_id(&*error.0), root);
                assert_eq!(error.0.to_string(), "first path");
            }
            1 => {
                assert!(error.is_inner::<DatabaseError>());
                let error = error.narrow::<ConfigError, _>().unwrap_err();
                let error = error.narrow::<DatabaseError, _>().unwrap();
                assert_eq!(core::any::Any::type_id(&*error.0), root);
                assert_eq!(error.0.to_string(), "second path");
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn identical_io_errors_recover_separately_after_classification_and_merging() {
    type Errors = (ConfigError, DatabaseError);
    for configuration in [true, false] {
        // Both operations fail with exactly the same implementation error.
        let error: ErrorUnion = ErrorUnion::new(io::Error::from(io::ErrorKind::PermissionDenied));
        let classified: ErrorUnion<Errors> = if configuration {
            error
                .classify::<(ConfigError,), _, _, _>(ConfigError)
                .widen()
        } else {
            error
                .classify::<(DatabaseError,), _, _, _>(DatabaseError)
                .widen()
        };
        let result: eros::Result<&str, Errors> = Err(classified);
        let mut configuration_calls = 0;
        let mut database_calls = 0;
        let operation = result
            .recover::<ConfigError, _>(|error| {
                configuration_calls += 1;
                assert_eq!(
                    (error.0.as_ref() as &dyn core::any::Any)
                        .downcast_ref::<io::Error>()
                        .unwrap()
                        .kind(),
                    io::ErrorKind::PermissionDenied
                );
                "configuration"
            })
            .recover::<DatabaseError, _>(|error| {
                database_calls += 1;
                assert_eq!(
                    (error.0.as_ref() as &dyn core::any::Any)
                        .downcast_ref::<io::Error>()
                        .unwrap()
                        .kind(),
                    io::ErrorKind::PermissionDenied
                );
                "database"
            })
            .into_value();
        assert_eq!(configuration_calls, usize::from(configuration));
        assert_eq!(database_calls, usize::from(!configuration));
        assert_eq!(
            operation,
            if configuration {
                "configuration"
            } else {
                "database"
            }
        );
    }
}

#[test]
fn result_recovery_handles_only_the_explicit_type() {
    type Errors = (fmt::Error, ConfigError);
    let inputs: [eros::Result<String>; 3] = [
        Ok(String::from("success")),
        Err(ErrorUnion::new(fmt::Error)),
        Err(ErrorUnion::new(io::Error::other("unhandled"))),
    ];
    for (index, result) in inputs.into_iter().enumerate() {
        let success = result.as_ref().ok().map(|value| value.as_ptr());
        let mixed: eros::Result<String, Errors> = result.classify(ConfigError);
        let mut calls = 0;
        let remainder: eros::Result<String, (ConfigError,)> =
            mixed.recover::<fmt::Error, _>(|_| {
                calls += 1;
                String::from("recovered")
            });
        assert_eq!(calls, usize::from(index == 1));
        let result: eros::Result<String> = remainder.erase();
        match index {
            0 => {
                let value = result.unwrap();
                assert_eq!(value, "success");
                assert_eq!(Some(value.as_ptr()), success);
            }
            1 => assert_eq!(result.unwrap(), "recovered"),
            2 => assert_eq!(
                result
                    .unwrap_err()
                    .narrow::<ConfigError, _>()
                    .unwrap()
                    .0
                    .to_string(),
                "unhandled"
            ),
            _ => unreachable!(),
        }
    }
}

#[test]
fn erasure_preserves_boxes_and_diagnostics_without_unwrapping() {
    type Boxed = (Box<dyn SendSyncError>,);
    for as_result in [false, true] {
        for nested in [false, true] {
            let boxed: Box<dyn SendSyncError> = Box::new(fmt::Error);
            let boxed: Box<dyn SendSyncError> = if nested { Box::new(boxed) } else { boxed };
            let error: ErrorUnion<Boxed> = ErrorUnion::new(boxed);
            let error = error.context("operation");
            let root = common::identity(error.inner());
            let contents = common::identity(error.as_single().as_ref());
            let report = format!("{error:?}");
            #[cfg(feature = "json")]
            let diagnostic = error.to_debug_json();
            let error = if as_result {
                let result: eros::Result<(), Boxed> = Err(error);
                result.erase().erase().unwrap_err()
            } else {
                error.erase().erase()
            };
            assert_eq!(common::identity(error.inner()), root);
            assert_eq!(format!("{error:?}"), report);
            #[cfg(feature = "json")]
            assert_eq!(error.to_debug_json(), diagnostic);
            // Erasing cannot expose the contained error for direct recovery.
            let error = error.narrow::<fmt::Error, _>().unwrap_err();
            let boxed = error.downcast_inner::<Box<dyn SendSyncError>>().unwrap();
            assert_eq!(common::identity(boxed.as_ref()), contents);
            assert_eq!(
                (boxed.as_ref() as &dyn core::any::Any).is::<Box<dyn SendSyncError>>(),
                nested
            );
        }
    }
    let boxed: Box<dyn SendSyncError> = Box::new(fmt::Error);
    let error = Err::<(), _>(boxed).erase().unwrap_err();
    assert!(error.is_inner::<Box<dyn SendSyncError>>());
}

fn erased<T: SendSyncError>(value: T) -> ErrorUnion {
    let error: ErrorUnion<(T,)> = ErrorUnion::new(value);
    error.erase()
}

#[derive(Debug)]
#[repr(align(128))]
struct Tracked(Arc<AtomicUsize>);
impl fmt::Display for Tracked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("tracked")
    }
}
impl std::error::Error for Tracked {}
impl Drop for Tracked {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn erased_roots_support_borrows_mutation_extraction_and_drop() {
    // Exercise erased roots for zero-sized, inline-sized, and aligned
    // heap roots, including unchecked extraction after a typed group selection.
    assert_eq!(
        erased(fmt::Error)
            .narrow::<(fmt::Error,), _>()
            .unwrap()
            .into_single(),
        fmt::Error
    );
    #[derive(Debug)]
    struct Status(u8);
    impl fmt::Display for Status {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    impl std::error::Error for Status {}
    let mut status = erased(Status(7));
    status.downcast_inner_mut::<Status>().unwrap().0 = 8;
    assert_eq!(status.downcast_inner::<Status>().unwrap().0, 8);
    let mut error = erased(MsgError::from("before"));
    let first = error.downcast_inner_ref::<MsgError>().unwrap();
    let second = error.downcast_inner_ref::<MsgError>().unwrap();
    assert!(std::ptr::eq(first, second));
    *error.downcast_inner_mut::<MsgError>().unwrap() = MsgError::from("after");
    assert_eq!(error.narrow::<MsgError, _>().unwrap().as_str(), "after");

    for operation in 0..4 {
        let drops = Arc::new(AtomicUsize::new(0));
        let error = erased(Tracked(drops.clone()));
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        match operation {
            0 => drop(error),
            1 => drop(error.downcast_inner::<Tracked>().unwrap()),
            2 => {
                let boxed = error.into_inner();
                let tracked = (boxed.as_ref() as &dyn core::any::Any)
                    .downcast_ref::<Tracked>()
                    .unwrap();
                assert!(Arc::ptr_eq(&tracked.0, &drops));
                drop(boxed);
            }
            3 => drop(
                error
                    .narrow::<(Tracked,), _>()
                    .unwrap()
                    .map_single(|value| value),
            ),
            _ => unreachable!(),
        }
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn source_chains_and_subsequent_classification_still_work() {
    #[derive(Debug)]
    struct SourceError(MsgError);
    impl fmt::Display for SourceError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("root")
        }
    }
    impl std::error::Error for SourceError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&self.0)
        }
    }
    let error = erased(SourceError(MsgError::from("source")));
    assert_eq!(error.source().unwrap().to_string(), "source");
    let error: ErrorUnion<(SourceError, ConfigError)> =
        error.classify(|inner: eros::InnerError| ConfigError(inner.into_boxed()));
    assert!(error.is_inner::<SourceError>());
    let error = error.narrow::<(SourceError,), _>().unwrap();
    assert_eq!(error.into_single().to_string(), "root");

    let error: ErrorUnion = ErrorUnion::new(SourceError(MsgError::from("source")));
    let error = error.classify::<(MsgError, ConfigError), _, _, _>(|inner: eros::InnerError| {
        ConfigError(inner.into_boxed())
    });
    assert!(error.is_inner::<ConfigError>());
    let context: Box<dyn SendSyncError> = Box::new(fmt::Error);
    let error: ErrorUnion = ErrorUnion::new(MsgError::from("root"));
    let error = error.context(context);
    let error = error.classify::<(fmt::Error, ConfigError), _, _, _>(|inner: eros::InnerError| {
        ConfigError(inner.into_boxed())
    });
    assert!(error.is_inner::<ConfigError>());
}
