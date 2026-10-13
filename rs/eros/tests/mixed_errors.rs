#![cfg(feature = "alloc")]

mod common;

use eros::{ErrorUnion, MsgError, ReshapeUnion, SendSyncError};
use std::{
    fmt, io,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[derive(Debug, thiserror::Error)]
#[error("domain failed: {0}")]
struct DomainError(#[source] Box<dyn SendSyncError>);

#[derive(Debug, thiserror::Error)]
#[error("other domain failed: {0}")]
struct OtherDomainError(#[source] Box<dyn SendSyncError>);

#[test]
fn custom_catch_all_runs_once_only_for_unmatched_errors() {
    type Domain = (fmt::Error, DomainError);
    let inputs: [eros::Result<String>; 4] = [
        Ok(String::from("success")),
        Err(ErrorUnion::new(fmt::Error)),
        Err(ErrorUnion::new(MsgError::from("failure"))),
        Err(ErrorUnion::new(DomainError(Box::new(MsgError::from(
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
        let mixed: eros::Result<String, Domain> = result.into_typed(|boxed| {
            drop(capture);
            calls += 1;
            DomainError(boxed)
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
                    .narrow::<DomainError, _>()
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
    type Combined = (fmt::Error, DomainError, OtherDomainError);
    let inputs: [ErrorUnion; 2] = [
        ErrorUnion::new(io::Error::other("first path")),
        ErrorUnion::new(io::Error::other("second path")),
    ];
    for (index, error) in inputs.into_iter().enumerate() {
        let error = error.context("operation");
        #[cfg(feature = "location")]
        let location = error.location();
        #[cfg(feature = "backtrace")]
        let backtrace = error.backtrace().to_string();
        #[cfg(feature = "context")]
        let contexts: Vec<_> = error.contexts().map(|frame| frame.to_string()).collect();
        let root = common::identity(error.inner());
        let error: ErrorUnion<Combined> = match index {
            0 => error
                .into_typed::<(fmt::Error, DomainError), _, _>(DomainError)
                .widen(),
            1 => error
                .into_typed::<(OtherDomainError, fmt::Error), _, _>(OtherDomainError)
                .widen(),
            _ => unreachable!(),
        };
        #[cfg(feature = "location")]
        assert!(std::ptr::eq(error.location(), location));
        #[cfg(feature = "backtrace")]
        assert_eq!(error.backtrace().to_string(), backtrace);
        #[cfg(feature = "context")]
        assert_eq!(
            error
                .contexts()
                .map(|frame| frame.to_string())
                .collect::<Vec<_>>(),
            contexts
        );
        let error = error.into_untyped();
        match index {
            0 => {
                let error = error.narrow::<DomainError, _>().unwrap();
                assert_eq!(common::identity(&*error.0), root);
                assert_eq!(error.0.to_string(), "first path");
            }
            1 => {
                assert!(error.is_inner::<OtherDomainError>());
                let error = error.narrow::<DomainError, _>().unwrap_err();
                let error = error.narrow::<OtherDomainError, _>().unwrap();
                assert_eq!(common::identity(&*error.0), root);
                assert_eq!(error.0.to_string(), "second path");
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn result_recovery_handles_only_the_explicit_type() {
    type Errors = (fmt::Error, DomainError);
    let inputs: [eros::Result<String>; 3] = [
        Ok(String::from("success")),
        Err(ErrorUnion::new(fmt::Error)),
        Err(ErrorUnion::new(io::Error::other("unhandled"))),
    ];
    for (index, result) in inputs.into_iter().enumerate() {
        let success = result.as_ref().ok().map(|value| value.as_ptr());
        let mixed: eros::Result<String, Errors> = result.into_typed(DomainError);
        let mut calls = 0;
        let remainder: eros::Result<String, (DomainError,)> =
            mixed.recover::<fmt::Error, _>(|_| {
                calls += 1;
                String::from("recovered")
            });
        assert_eq!(calls, usize::from(index == 1));
        let result: eros::Result<String> = remainder.into_untyped();
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
                    .narrow::<DomainError, _>()
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
fn untyped_conversion_unwraps_one_box_layer() {
    let boxed: Box<dyn SendSyncError> = Box::new(fmt::Error);
    let error: ErrorUnion = ErrorUnion::new(boxed);
    let error = error.into_untyped();
    assert_eq!(error.downcast_inner::<fmt::Error>().unwrap(), fmt::Error);

    let inner: Box<dyn SendSyncError> = Box::new(fmt::Error);
    let outer: Box<dyn SendSyncError> = Box::new(inner);
    let error: ErrorUnion<(Box<dyn SendSyncError>,)> = ErrorUnion::new(outer);
    let error = error.into_untyped();
    assert!(error.is_inner::<Box<dyn SendSyncError>>());
    assert_eq!(
        error.into_untyped().downcast_inner::<fmt::Error>().unwrap(),
        fmt::Error
    );
}

fn restored<T: SendSyncError>(value: T) -> ErrorUnion {
    let boxed: Box<dyn SendSyncError> = Box::new(value);
    let error: ErrorUnion = ErrorUnion::new(boxed);
    error.into_untyped()
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
fn dynamic_roots_support_borrows_mutation_extraction_and_drop() {
    // Exercise dynamic extraction for zero-sized, inline-sized, and aligned
    // heap roots, including unchecked extraction after a typed group selection.
    assert_eq!(
        restored(fmt::Error)
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
    let mut status = restored(Status(7));
    status.downcast_inner_mut::<Status>().unwrap().0 = 8;
    assert_eq!(status.downcast_inner::<Status>().unwrap().0, 8);
    let mut error = restored(MsgError::from("before"));
    let first = error.downcast_inner_ref::<MsgError>().unwrap();
    let second = error.downcast_inner_ref::<MsgError>().unwrap();
    assert!(std::ptr::eq(first, second));
    *error.downcast_inner_mut::<MsgError>().unwrap() = MsgError::from("after");
    assert_eq!(error.narrow::<MsgError, _>().unwrap().as_str(), "after");

    for operation in 0..4 {
        let drops = Arc::new(AtomicUsize::new(0));
        let error = restored(Tracked(drops.clone()));
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        match operation {
            0 => drop(error),
            1 => drop(error.downcast_inner::<Tracked>().unwrap()),
            2 => {
                let address = error.inner() as *const dyn SendSyncError as *const ();
                let boxed = error.into_inner();
                assert_eq!(&*boxed as *const dyn SendSyncError as *const (), address);
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
    let error = restored(SourceError(MsgError::from("source")));
    assert_eq!(error.source().unwrap().to_string(), "source");
    let error: ErrorUnion<(SourceError, DomainError)> = error.into_typed(DomainError);
    assert!(error.is_inner::<SourceError>());
    let error = error.narrow::<(SourceError,), _>().unwrap();
    assert_eq!(error.into_single().to_string(), "root");

    let error: ErrorUnion = ErrorUnion::new(SourceError(MsgError::from("source")));
    let error = error.into_typed::<(MsgError, DomainError), _, _>(DomainError);
    assert!(error.is_inner::<DomainError>());
    let context: Box<dyn SendSyncError> = Box::new(fmt::Error);
    let error: ErrorUnion = ErrorUnion::new(MsgError::from("root"));
    let error = error.context(context);
    let error = error.into_typed::<(fmt::Error, DomainError), _, _>(DomainError);
    assert!(error.is_inner::<DomainError>());
}
