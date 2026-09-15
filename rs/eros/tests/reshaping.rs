use eros::{E2, ErrorUnion, IntoAnyUnion, IntoUnion, MsgError, ReshapeUnion, SendSyncError};
use std::fmt;

type Pair = (MsgError, fmt::Error);
type Triple = (MsgError, fmt::Error, std::io::Error);

#[test]
fn subset_rejects_errors_outside_the_requested_types() {
    let error: ErrorUnion<Triple> = ErrorUnion::new(MsgError::from("original"));
    let error = error.context("preserved context");
    let report = format!("{error:?}");
    let result = error.subset::<(fmt::Error,), _>();
    // Check the branch before accessing the typed value: an incorrect Ok would
    // violate the invariant used by the enum conversions' unchecked downcasts.
    assert!(result.is_err(), "a MsgError is not a fmt::Error");
    let remainder: ErrorUnion<(MsgError, std::io::Error)> = result.unwrap_err();
    assert_eq!(format!("{remainder:?}"), report);
    assert_eq!(
        remainder.narrow::<MsgError, _>().unwrap().as_str(),
        "original"
    );
}

#[test]
fn subset_accepts_each_member_in_requested_order() {
    let errors: [ErrorUnion<Triple>; 2] = [
        ErrorUnion::new(MsgError::from("message")),
        ErrorUnion::new(fmt::Error),
    ];
    for (index, error) in errors.into_iter().enumerate() {
        let error = error.context("context");
        let report = format!("{error:?}");
        let subset = error.subset::<(fmt::Error, MsgError), _>().unwrap();
        assert_eq!(format!("{subset:?}"), report);
        match (index, subset.into_enum()) {
            (0, E2::B(error)) => assert_eq!(error.as_str(), "message"),
            (1, E2::A(fmt::Error)) => {}
            _ => panic!("subset changed the active variant"),
        }
    }
}

#[test]
fn empty_subset_always_returns_the_original_union() {
    let error: ErrorUnion<Pair> = ErrorUnion::new(MsgError::from("original"));
    let result = error.subset::<(), _>();
    assert!(result.is_err());
    let remainder: ErrorUnion<Pair> = result.unwrap_err();
    assert_eq!(remainder.to_string(), "original");
}

#[test]
fn full_subset_can_be_reordered_and_has_an_empty_remainder() {
    let error: ErrorUnion<Pair> = ErrorUnion::new(fmt::Error);
    let subset: Result<ErrorUnion<(fmt::Error, MsgError)>, ErrorUnion<()>> = error.subset();
    assert!(matches!(subset.unwrap().into_enum(), E2::A(fmt::Error)));
}

#[test]
fn subset_partitions_every_variant_and_preserves_metadata_and_identity() {
    use std::{io, num::ParseIntError};

    type Variants = (MsgError, fmt::Error, io::Error, ParseIntError);
    let errors: [ErrorUnion<Variants>; 4] = [
        ErrorUnion::new(MsgError::from(String::from("message"))),
        ErrorUnion::new(fmt::Error),
        ErrorUnion::new(io::Error::new(io::ErrorKind::PermissionDenied, "denied")),
        ErrorUnion::new("invalid".parse::<u16>().unwrap_err()),
    ];

    for (index, error) in errors.into_iter().enumerate() {
        let error = error.context("inner operation").context("outer operation");
        #[cfg(feature = "user_context")]
        let error = error.user_context("user message");
        let original_inner = error.inner() as *const dyn SendSyncError as *const ();
        let original_report = format!("{error:?}");
        #[cfg(feature = "diagnostic")]
        let original_json = error.to_debug_json();

        // The requested order differs from the input; the complement must retain
        // its original order. The explicit types also check the computed sets.
        let partition: Result<
            ErrorUnion<(io::Error, fmt::Error)>,
            ErrorUnion<(MsgError, ParseIntError)>,
        > = error.subset();
        let error: ErrorUnion<Variants> = match partition {
            Ok(selected) => {
                assert!(matches!(index, 1 | 2), "selected an unlisted variant");
                match selected.as_enum() {
                    E2::A(error) => {
                        assert_eq!(index, 2);
                        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
                    }
                    E2::B(_) => assert_eq!(index, 1),
                }
                selected.widen()
            }
            Err(remainder) => {
                assert!(matches!(index, 0 | 3), "rejected a requested variant");
                match remainder.as_enum() {
                    E2::A(error) => {
                        assert_eq!(index, 0);
                        assert_eq!(error.as_str(), "message");
                    }
                    E2::B(error) => {
                        assert_eq!(index, 3);
                        assert_eq!(error.kind(), &std::num::IntErrorKind::InvalidDigit);
                    }
                }
                remainder.widen()
            }
        };

        assert_eq!(
            error.inner() as *const dyn SendSyncError as *const (),
            original_inner
        );
        assert_eq!(format!("{error:?}"), original_report);
        #[cfg(feature = "diagnostic")]
        assert_eq!(error.to_debug_json(), original_json);
    }
}

#[test]
fn subset_checks_the_inner_error_type_without_matching_sources_or_contexts() {
    #[derive(Debug)]
    struct OuterError(MsgError);

    impl fmt::Display for OuterError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("outer error")
        }
    }

    impl std::error::Error for OuterError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&self.0)
        }
    }

    let error: ErrorUnion<(OuterError, MsgError)> =
        ErrorUnion::new(OuterError(MsgError::from("source error")));
    let context: Box<dyn SendSyncError> = Box::new(MsgError::from("context error"));
    let error = error.context(context);
    let report = format!("{error:?}");
    let remainder: ErrorUnion<(OuterError,)> = error.subset::<(MsgError,), _>().unwrap_err();
    assert_eq!(format!("{remainder:?}"), report);
    assert_eq!(remainder.into_single().0.as_str(), "source error");
}

#[test]
fn result_narrow_covers_success_matching_error_and_remainder() {
    let success: eros::Result<String, Pair> = Ok("value".into());
    assert_eq!(
        success.narrow::<MsgError, _>().unwrap_err().unwrap(),
        "value"
    );

    let matching: eros::Result<(), Pair> = Err(ErrorUnion::new(MsgError::from("match")));
    assert_eq!(
        matching.narrow::<MsgError, _>().unwrap().as_str(),
        "match"
    );

    let other: ErrorUnion<Pair> = ErrorUnion::new(fmt::Error);
    let other = other.context("retain me");
    let report = format!("{other:?}");
    let result: eros::Result<(), Pair> = Err(other);
    let remainder: eros::Result<(), (fmt::Error,)> =
        result.narrow::<MsgError, _>().unwrap_err();
    let error = remainder.unwrap_err();
    assert_eq!(format!("{error:?}"), report);
    assert_eq!(error.into_single(), fmt::Error);
}

#[test]
fn recover_preserves_success_and_only_calls_the_matching_handler() {
    let success: eros::Result<String, Pair> = Ok(String::from("original success"));
    let original = success.as_ref().unwrap().as_ptr();
    let success: eros::Result<String, (fmt::Error,)> =
        success.recover::<MsgError, _>(|_| panic!("must not handle success"));
    let success = success.unwrap();
    assert_eq!(success.as_ptr(), original);

    // Infers the target from the handler and accepts an FnOnce closure.
    let replacement = String::from("recovered");
    let result: eros::Result<String, Pair> = Err(ErrorUnion::new(MsgError::from("match")));
    let mut calls = 0;
    let result: eros::Result<String, (fmt::Error,)> =
        result.recover(|error: ErrorUnion<(MsgError,)>| {
            calls += 1;
            assert_eq!(error.into_single().as_str(), "match");
            replacement
        });
    assert_eq!(calls, 1);
    assert_eq!(result.unwrap(), "recovered");
}

#[test]
fn recover_preserves_error_identity_and_metadata_in_both_branches() {
    for matches in [true, false] {
        let error: ErrorUnion<Triple> = if matches {
            ErrorUnion::new(fmt::Error)
        } else {
            ErrorUnion::new(MsgError::from("unhandled"))
        };
        let error = error.context("inner").context("outer");
        #[cfg(feature = "user_context")]
        let error = error.user_context("user message");
        let original = error.inner() as *const dyn SendSyncError as *const ();
        let report = format!("{error:?}");
        #[cfg(feature = "diagnostic")]
        let diagnostic = error.to_debug_json();
        let result: eros::Result<(), Triple> = Err(error);
        let mut calls = 0;
        // fmt::Error is in the middle, and the remainder retains its order.
        let result: eros::Result<(), (MsgError, std::io::Error)> =
            result.recover::<fmt::Error, _>(|error| {
                calls += 1;
                assert_eq!(
                    error.inner() as *const dyn SendSyncError as *const (),
                    original
                );
                assert_eq!(format!("{error:?}"), report);
                #[cfg(feature = "diagnostic")]
                assert_eq!(error.to_debug_json(), diagnostic);
                assert_eq!(error.into_single(), fmt::Error);
            });
        assert_eq!(calls, usize::from(matches));
        if matches {
            result.unwrap();
        } else {
            let error = result.unwrap_err();
            assert_eq!(
                error.inner() as *const dyn SendSyncError as *const (),
                original
            );
            assert_eq!(format!("{error:?}"), report);
            #[cfg(feature = "diagnostic")]
            assert_eq!(error.to_debug_json(), diagnostic);
            assert!(matches!(error.into_enum(), E2::A(_)));
        }
    }
}

#[test]
fn recover_can_handle_each_type_until_the_error_set_is_empty() {
    for (error, expected) in [
        (ErrorUnion::new(MsgError::from("first")), 2),
        (ErrorUnion::new(fmt::Error), 1),
    ] {
        let result: eros::Result<u8, Pair> = Err(error);
        // Start with the last tuple member, then handle the remaining one.
        let result: eros::Result<u8, ()> = result
            .recover::<fmt::Error, _>(|_| 1)
            .recover::<MsgError, _>(|_| 2);
        assert_eq!(result.unwrap(), expected);
    }
}

#[test]
fn recover_does_not_match_source_or_context_errors() {
    let root = std::io::Error::other(MsgError::from("source"));
    let error: ErrorUnion<(std::io::Error, MsgError)> = ErrorUnion::new(root);
    let context: Box<dyn SendSyncError> = Box::new(MsgError::from("context"));
    let error = error.context(context);
    let report = format!("{error:?}");
    let result: eros::Result<(), (std::io::Error, MsgError)> = Err(error);
    let remainder: eros::Result<(), (std::io::Error,)> =
        result.recover::<MsgError, _>(|_| panic!("must check only the inner error"));
    assert_eq!(format!("{:?}", remainder.unwrap_err()), report);
}

#[test]
fn result_conversions_preserve_success_values_and_error_metadata() {
    let success: Result<String, MsgError> = Ok("typed".into());
    let success: eros::Result<_, Pair> = success.union();
    let success: eros::Result<_, Triple> = success.widen();
    assert_eq!(success.any_union().unwrap(), "typed");

    let success: Result<String, MsgError> = Ok("erased".into());
    assert_eq!(success.any_union().unwrap(), "erased");

    let error: eros::Result<(), Pair> = Err(MsgError::from("typed")).union();
    assert!(error.as_ref().unwrap_err().is_inner::<MsgError>());
    let error = error.unwrap_err().context("metadata");
    let report = format!("{error:?}");
    let result: eros::Result<(), Pair> = Err(error);
    let widened: eros::Result<(), Triple> = result.widen();
    let erased = widened.any_union().unwrap_err();
    assert_eq!(format!("{erased:?}"), report);
    let result: eros::Result<()> = Err(erased);
    assert_eq!(format!("{:?}", result.any_union().unwrap_err()), report);

    let erased: eros::Result<()> = Err(MsgError::from("erased")).any_union();
    assert_eq!(
        erased
            .unwrap_err()
            .downcast_inner::<MsgError>()
            .unwrap()
            .as_str(),
        "erased"
    );
}

#[test]
fn singleton_borrowing_and_mapping_preserve_owned_values_and_metadata() {
    let mut error: ErrorUnion<(MsgError,)> = MsgError::from("before").into();
    assert_eq!(error.as_str(), "before"); // Deref
    assert!(std::ptr::eq::<MsgError>(&*error, error.as_ref()));
    *error.as_mut() = MsgError::from(String::from("after"));
    let error = error.context("mapping");
    let report = format!("{error:?}");
    let mut calls = 0;
    let mapped = error.map_single(|error| {
        calls += 1;
        assert_eq!(error.as_str(), "after");
        error
    });
    assert_eq!(calls, 1);
    assert_eq!(format!("{mapped:?}"), report);
    assert_eq!(mapped.into_single().as_str(), "after");

    #[derive(Debug)]
    struct Converted(MsgError);
    impl fmt::Display for Converted {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.fmt(f)
        }
    }
    impl std::error::Error for Converted {}
    impl From<MsgError> for Converted {
        fn from(error: MsgError) -> Self {
            Self(error)
        }
    }
    let from_convertible: ErrorUnion<(Converted,)> = MsgError::from("converted").into();
    assert_eq!(from_convertible.into_single().0.as_str(), "converted");
}

#[test]
fn mutable_erased_root_and_boxed_error_trait_refer_to_the_actual_error() {
    let mut error: ErrorUnion = ErrorUnion::new(MsgError::from("before"));
    let root: &mut dyn std::any::Any = error.inner_mut();
    *root.downcast_mut::<MsgError>().unwrap() = MsgError::from("after");
    assert_eq!(
        error
            .inner()
            .as_any()
            .downcast_ref::<MsgError>()
            .unwrap()
            .as_str(),
        "after"
    );
    assert!(error.source().is_none());

    let boxed = error.into_inner();
    assert!(boxed.as_any().is::<Box<dyn eros::SendSyncError>>());
    assert!(boxed.as_ref().as_any().is::<MsgError>());
    let source = std::error::Error::source(&boxed).unwrap();
    assert_eq!(
        source.downcast_ref::<MsgError>().unwrap().as_str(),
        "after"
    );
}

#[test]
fn failed_native_adapter_downcast_retains_the_original_adapter() {
    let error: ErrorUnion<Pair> = ErrorUnion::new(MsgError::from("original"));
    let error = error.context("retained context");
    let report = format!("{error:?}");
    let adapter = error.into_dyn_error();
    let original = &*adapter as *const dyn eros::SendSyncError as *const ();
    let adapter = ErrorUnion::<(MsgError,)>::try_from_dyn_error(adapter).unwrap_err();
    assert_eq!(
        &*adapter as *const dyn eros::SendSyncError as *const (),
        original
    );
    let recovered = ErrorUnion::<Pair>::try_from_dyn_error(adapter).unwrap();
    assert_eq!(format!("{recovered:?}"), report);
}

#[test]
fn public_unions_and_adapters_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ErrorUnion>();
    assert_send_sync::<ErrorUnion<Pair>>();
    assert_send_sync::<Box<dyn eros::SendSyncError>>();
}
