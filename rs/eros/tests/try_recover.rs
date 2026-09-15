use eros::{E2, ErrorUnion, IntoUnion, MsgError, ReshapeUnion, SendSyncError};
use std::{fmt, io, num::ParseIntError};

#[test]
fn success_passes_through_without_calling_the_handler() {
    let result: eros::Result<String, (io::Error, MsgError)> = Ok(String::from("original"));
    let original = result.as_ref().unwrap().as_ptr();
    let result: eros::Result<String, (MsgError,)> =
        result.try_recover::<io::Error, _, _, _>(|_| panic!("must not handle success"));
    let value = result.unwrap();
    assert_eq!(value.as_ptr(), original);
}

#[test]
fn matching_handler_and_fallback_keep_their_own_metadata_and_identity() {
    for succeeds in [true, false] {
        let error: ErrorUnion<(io::Error, MsgError)> =
            ErrorUnion::new(io::Error::other("original"));
        let error = error.context("original operation");
        let original = error.inner() as *const dyn SendSyncError as *const ();
        let original_report = format!("{error:?}");
        let fallback: ErrorUnion<(MsgError,)> = ErrorUnion::new(MsgError::from("fallback"));
        let fallback = fallback.context("fallback operation");
        #[cfg(feature = "user_context")]
        let fallback = fallback.user_context("fallback user message");
        let fallback_ptr = fallback.inner() as *const dyn SendSyncError as *const ();
        let fallback_report = format!("{fallback:?}");
        #[cfg(feature = "diagnostic")]
        let fallback_json = fallback.to_debug_json();
        let result: eros::Result<String, (io::Error, MsgError)> = Err(error);
        let replacement = String::from("recovered");
        let mut calls = 0;
        // Both the target and the output set are inferred from the handler.
        // Moving fallback/replacement also exercises an FnOnce closure.
        let result = result.try_recover(|error: ErrorUnion<(io::Error,)>| {
            calls += 1;
            assert_eq!(
                error.inner() as *const dyn SendSyncError as *const (),
                original
            );
            assert_eq!(format!("{error:?}"), original_report);
            if succeeds {
                Ok(replacement)
            } else {
                Err(fallback)
            }
        });
        assert_eq!(calls, 1);
        if succeeds {
            assert_eq!(result.unwrap(), "recovered");
        } else {
            let error = result.unwrap_err();
            assert_eq!(
                error.inner() as *const dyn SendSyncError as *const (),
                fallback_ptr
            );
            assert_eq!(format!("{error:?}"), fallback_report);
            #[cfg(feature = "diagnostic")]
            assert_eq!(error.to_debug_json(), fallback_json);
        }
    }
}

#[test]
fn unhandled_variants_can_be_reordered_and_widened_without_losing_metadata() {
    type Input = (MsgError, io::Error, fmt::Error);
    type Output = (fmt::Error, ParseIntError, MsgError);
    let errors: [ErrorUnion<Input>; 2] = [
        ErrorUnion::new(MsgError::from("message")),
        ErrorUnion::new(fmt::Error),
    ];
    for (index, error) in errors.into_iter().enumerate() {
        let error = error.context("retained operation");
        let original = error.inner() as *const dyn SendSyncError as *const ();
        let report = format!("{error:?}");
        #[cfg(feature = "diagnostic")]
        let diagnostic = error.to_debug_json();
        let result: eros::Result<(), Input> = Err(error);
        let result: eros::Result<(), Output> =
            result.try_recover::<io::Error, _, _, _>(|_| panic!("must not handle other types"));
        let error = result.unwrap_err();
        assert_eq!(
            error.inner() as *const dyn SendSyncError as *const (),
            original
        );
        assert_eq!(format!("{error:?}"), report);
        #[cfg(feature = "diagnostic")]
        assert_eq!(error.to_debug_json(), diagnostic);
        match (index, error.into_enum()) {
            (0, eros::E3::C(error)) => assert_eq!(error.as_str(), "message"),
            (1, eros::E3::A(fmt::Error)) => {}
            _ => panic!("wrong remaining variant"),
        }
    }
}

#[test]
fn destination_inference_supports_plain_and_union_fallbacks_with_new_error_types() {
    fn plain_fallback() -> eros::Result<u16, (MsgError, ParseIntError)> {
        let result: eros::Result<u16, (io::Error, MsgError)> =
            Err(ErrorUnion::new(io::Error::other("original")));
        result.try_recover(|_: ErrorUnion<(io::Error,)>| "invalid".parse::<u16>().union())
    }

    fn union_fallback() -> eros::Result<u16, (MsgError, ParseIntError)> {
        let result: eros::Result<u16, (io::Error, MsgError)> =
            Err(ErrorUnion::new(io::Error::other("original")));
        result.try_recover(|_: ErrorUnion<(io::Error,)>| {
            let fallback: eros::Result<u16, (ParseIntError,)> = "invalid".parse::<u16>().union();
            fallback.widen()
        })
    }

    for result in [plain_fallback(), union_fallback()] {
        assert!(matches!(result.unwrap_err().into_enum(), E2::B(_)));
    }
}

#[test]
fn handler_can_reintroduce_the_handled_type_and_preserve_its_diagnostics() {
    let error: ErrorUnion<(io::Error, MsgError)> = ErrorUnion::new(io::Error::other("original"));
    let error = error.context("original operation");
    let original = error.inner() as *const dyn SendSyncError as *const ();
    let report = format!("{error:?}");
    let result: eros::Result<(), (io::Error, MsgError)> = Err(error);
    let result: eros::Result<(), (MsgError, io::Error)> =
        result.try_recover(|error: ErrorUnion<(io::Error,)>| Err(error.widen()));
    let error = result.unwrap_err();
    assert_eq!(
        error.inner() as *const dyn SendSyncError as *const (),
        original
    );
    assert_eq!(format!("{error:?}"), report);
    assert!(matches!(error.into_enum(), E2::B(_)));
}

#[test]
fn singleton_can_recover_to_an_empty_error_set() {
    let result: eros::Result<u16, (MsgError,)> = Err(ErrorUnion::new(MsgError::from("original")));
    let result: eros::Result<u16, ()> = result.try_recover(|_: ErrorUnion<(MsgError,)>| Ok(8080));
    assert_eq!(result.into_value(), 8080);
}

#[test]
fn only_the_inner_error_is_matched() {
    let error: ErrorUnion<(io::Error, MsgError)> =
        ErrorUnion::new(io::Error::other(MsgError::from("source")));
    let context: Box<dyn SendSyncError> = Box::new(MsgError::from("context"));
    let error = error.context(context);
    let report = format!("{error:?}");
    let result: eros::Result<(), (io::Error, MsgError)> = Err(error);
    let result: eros::Result<(), (io::Error,)> = result
        .try_recover::<MsgError, _, _, _>(|_| panic!("must not handle source or context errors"));
    assert_eq!(format!("{:?}", result.unwrap_err()), report);
}

#[test]
fn chained_fallbacks_can_replace_reintroduce_and_finally_recover_an_error() {
    let result: eros::Result<u8, (MsgError, fmt::Error)> = Err(MsgError::from("original")).union();
    let mut calls = Vec::new();
    let result: eros::Result<u8, (fmt::Error, io::Error)> =
        result.try_recover::<MsgError, _, _, _>(|error| {
            calls.push("replace");
            assert_eq!(error.as_str(), "original");
            Err(io::Error::other("fallback")).union()
        });
    let result: eros::Result<u8, (io::Error, fmt::Error)> =
        result.try_recover::<io::Error, _, _, _>(|error| {
            calls.push("reintroduce");
            assert_eq!(error.kind(), io::ErrorKind::Other);
            Err(error.context("retry failed").widen())
        });
    let result: eros::Result<u8, (io::Error,)> =
        result.recover::<fmt::Error, _>(|_| panic!("unrelated handler"));
    let value = result
        .recover::<io::Error, _>(|error| {
            calls.push("recover");
            assert_eq!(error.into_single().to_string(), "fallback");
            7
        })
        .into_value();
    assert_eq!(value, 7);
    assert_eq!(calls, ["replace", "reintroduce", "recover"]);
}

#[test]
fn recovery_accepts_borrowed_values_and_handlers_without_send_or_static_bounds() {
    for fallible in [false, true] {
        let mut storage = 0;
        let token = std::rc::Rc::new(());
        let borrowed_token = &token;
        let borrowed_storage = &mut storage;
        let result: eros::Result<&mut u8, (MsgError,)> =
            Err(ErrorUnion::new(MsgError::from("original")));
        let handler = move |error: ErrorUnion<(MsgError,)>| {
            assert_eq!(error.as_str(), "original");
            assert_eq!(std::rc::Rc::strong_count(borrowed_token), 1);
            borrowed_storage
        };
        let result: eros::Result<&mut u8, ()> = if fallible {
            result.try_recover::<MsgError, _, _, _>(|error| Ok(handler(error)))
        } else {
            result.recover::<MsgError, _>(handler)
        };
        *result.into_value() = 7;
        assert_eq!(storage, 7);
    }
}
