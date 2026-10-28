#![cfg(feature = "alloc")]

use eros::{Context, ContextValue, ErrorUnion, MsgError, SendSyncDebugDisplay, SendSyncError};
use std::{borrow::Cow, cell::Cell};

#[test]
#[cfg(feature = "context")]
fn contexts_borrow_frames_in_attachment_order() {
    let error = eros::error!("root");
    assert_eq!(error.contexts().len(), 0);
    let context: Box<dyn SendSyncError> = Box::new(MsgError::from("error context"));
    let original = context.as_ref() as *const dyn SendSyncError as *const ();
    let error = error
        .context("static")
        .context(String::from("owned"))
        .context(context);
    assert_eq!(error.contexts().len(), 3);
    let frames: Vec<&eros::ContextFrame> = error.contexts().collect();
    assert_eq!(frames[0].value().as_str(), Some("static"));
    assert_eq!(frames[1].value().as_str(), Some("owned"));
    let error_value = frames[2].value().as_boxed().unwrap();
    assert_eq!(
        error_value as *const dyn SendSyncDebugDisplay as *const (),
        original
    );
    assert!((error_value as &dyn core::any::Any).is::<MsgError>());
    #[cfg(feature = "user_context")]
    assert!(frames.iter().all(|frame| !frame.is_user_context()));
    assert_eq!(
        error
            .contexts()
            .rev()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["error context", "owned", "static"]
    );
}

#[test]
#[cfg(any(
    feature = "error_location",
    all(feature = "context", feature = "context_location")
))]
fn locations_expose_original_capture_and_each_context_attachment() {
    let _root_line = line!() + 1;
    let error: ErrorUnion<(MsgError,)> = ErrorUnion::new(MsgError::from("root"));
    let _first_line = line!() + 1;
    let error = error.context("first");
    let _second_line = line!() + 1;
    let error = error.with_context(|| "second");
    // These operations must keep the capture site, not report the mapper's site.
    let error = error.map_single(|error| error);
    let error: ErrorUnion<(std::fmt::Error, MsgError)> = error.widen();
    let error: ErrorUnion = error.into();

    #[cfg(feature = "error_location")]
    {
        let location = error.location();
        assert_eq!(location.file(), file!());
        assert_eq!(location.line(), _root_line);
        assert!(location.column() > 0);
    }
    #[cfg(all(feature = "context", feature = "context_location"))]
    for (frame, line) in error.contexts().zip([_first_line, _second_line]) {
        let location = frame.location();
        assert_eq!(location.file(), file!());
        assert_eq!(location.line(), line);
        assert!(location.column() > 0);
    }
}

// Each adapter may create a root and attach context in the same call. Check
// both locations independently so either feature preserves the public call site.
#[test]
#[cfg(any(
    feature = "error_location",
    all(feature = "context", feature = "context_location")
))]
fn context_adapters_capture_the_public_call_site() {
    macro_rules! check {
        ($error:expr) => {{
            let expected_line = line!();
            let error = $error;
            #[cfg(feature = "error_location")]
            {
                assert_eq!(error.location().file(), file!());
                assert_eq!(error.location().line(), expected_line);
                assert!(core::ptr::eq(error.location(), error.report().location()));
            }
            #[cfg(all(feature = "context", feature = "context_location"))]
            for frame in error.contexts() {
                assert_eq!(frame.location().file(), file!());
                assert_eq!(frame.location().line(), expected_line);
            }
        }};
    }
    let failure = || Err::<(), _>(std::fmt::Error);
    check!(failure().context("eager").unwrap_err());
    check!(failure().with_context(|| "lazy").unwrap_err());
    check!(None::<()>.context("eager").unwrap_err());
    check!(None::<()>.with_context(|| "lazy").unwrap_err());
    #[cfg(feature = "user_context")]
    {
        check!(failure().user_context("eager").unwrap_err());
        check!(failure().with_user_context(|| "lazy").unwrap_err());
        check!(None::<()>.user_context("eager").unwrap_err());
        check!(None::<()>.with_user_context(|| "lazy").unwrap_err());
    }
}

#[test]
#[cfg(feature = "user_context")]
fn contexts_include_user_context_flags_and_locations() {
    let error = eros::error!("root").context("internal");
    let _public_line = line!() + 1;
    let error = error.user_context("public");
    let _lazy_line = line!() + 1;
    let error = error.with_user_context(|| "lazy public");
    assert_eq!(
        error
            .contexts()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["internal", "public", "lazy public"]
    );
    assert_eq!(
        error
            .contexts()
            .map(|frame| frame.is_user_context())
            .collect::<Vec<_>>(),
        [false, true, true]
    );
    assert_eq!(
        error
            .contexts()
            .filter(|frame| frame.is_user_context())
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["public", "lazy public"]
    );
    #[cfg(feature = "context_location")]
    for (frame, line) in error.contexts().skip(1).zip([_public_line, _lazy_line]) {
        assert_eq!(frame.location().line(), line);
    }
}

#[test]
fn context_values_expose_messages_and_preserve_boxed_identity() {
    for source in [ContextValue::from("static"), Cow::Borrowed("static").into()] {
        assert_eq!(source.as_str(), Some("static"));
        assert!(source.as_boxed().is_none());
        assert_eq!(source.to_string(), "static");
        assert_eq!(format!("{source:?}"), "Message(\"static\")");
    }
    for source in [
        ContextValue::from(String::from("owned")),
        Cow::<str>::Owned(String::from("owned")).into(),
    ] {
        assert_eq!(source.as_str(), Some("owned"));
        assert!(source.as_boxed().is_none());
        assert_eq!(source.to_string(), "owned");
    }
    let boxed: Box<dyn SendSyncError> = Box::new(MsgError::from("error"));
    let ptr = boxed.as_ref() as *const dyn SendSyncError as *const ();
    let source = ContextValue::from(boxed);
    assert_eq!(source.to_string(), "error");
    assert!(source.as_str().is_none());
    let error = source.as_boxed().unwrap();
    assert_eq!(error as *const dyn SendSyncDebugDisplay as *const (), ptr);
    assert!((error as &dyn core::any::Any).is::<MsgError>());

    let boxed: Box<dyn SendSyncDebugDisplay> = Box::new(String::from("boxed value"));
    let ptr = boxed.as_ref() as *const dyn SendSyncDebugDisplay as *const ();
    let source = ContextValue::from(boxed);
    assert!(source.as_str().is_none());
    assert_eq!(source.to_string(), "boxed value");
    assert_eq!(format!("{source:?}"), "Boxed(\"boxed value\")");
    let value = source.as_boxed().unwrap();
    assert_eq!(value as *const dyn SendSyncDebugDisplay as *const (), ptr);
    assert_eq!(
        (value as &dyn core::any::Any)
            .downcast_ref::<String>()
            .unwrap(),
        "boxed value"
    );
}

// Conversion must be deferred too: even eager context values should only be
// converted on failures when context recording is enabled.
struct CountConversion<'a>(&'a Cell<usize>);

impl From<CountConversion<'_>> for ContextValue {
    fn from(value: CountConversion<'_>) -> Self {
        value.0.set(value.0.get() + 1);
        "counted context".into()
    }
}

macro_rules! context_contract {
    ($test:ident, $success:expr, $failure:expr) => {
        #[test]
        fn $test() {
            let calls = Cell::new(0);
            let conversions = Cell::new(0);
            let success = $success;
            assert_eq!(
                success.context(CountConversion(&conversions)).unwrap(),
                "success"
            );
            let success = $success;
            assert_eq!(
                success
                    .with_context(|| {
                        calls.set(calls.get() + 1);
                        CountConversion(&conversions)
                    })
                    .unwrap(),
                "success"
            );
            assert_eq!(calls.get(), 0);
            assert_eq!(conversions.get(), 0);

            let failure = $failure;
            let error = failure.context(CountConversion(&conversions)).unwrap_err();
            assert_eq!(conversions.get(), usize::from(cfg!(feature = "context")));
            assert_eq!(
                format!("{error:#?}").contains("counted context"),
                cfg!(feature = "context")
            );

            let failure = $failure;
            let error = failure
                .with_context(|| {
                    calls.set(calls.get() + 1);
                    CountConversion(&conversions)
                })
                .unwrap_err();
            assert_eq!(calls.get(), usize::from(cfg!(feature = "context")));
            assert_eq!(
                conversions.get(),
                2 * usize::from(cfg!(feature = "context"))
            );
            assert_eq!(
                format!("{error:#?}").contains("counted context"),
                cfg!(feature = "context")
            );
        }
    };
}

context_contract!(
    plain_result_context_is_lazy,
    Ok::<_, MsgError>("success"),
    Err::<(), _>(MsgError::from("root"))
);
context_contract!(
    typed_result_context_is_lazy,
    Ok::<_, ErrorUnion<(MsgError,)>>("success"),
    Err::<(), ErrorUnion<(MsgError,)>>(ErrorUnion::new(MsgError::from("root")))
);
context_contract!(
    erased_result_context_is_lazy,
    Ok::<_, ErrorUnion>("success"),
    Err::<(), ErrorUnion>(eros::error!("root"))
);
context_contract!(option_context_is_lazy, Some("success"), None::<()>);

#[test]
fn direct_union_context_is_lazy_when_the_feature_is_disabled() {
    let calls = Cell::new(0);
    let conversions = Cell::new(0);
    let error = eros::error!("root")
        .context(CountConversion(&conversions))
        .with_context(|| {
            calls.set(calls.get() + 1);
            CountConversion(&conversions)
        });
    assert_eq!(calls.get(), usize::from(cfg!(feature = "context")));
    assert_eq!(
        conversions.get(),
        2 * usize::from(cfg!(feature = "context"))
    );
    assert_eq!(error.to_string(), "root");
    assert_eq!(
        format!("{error:#?}").matches("counted context").count(),
        2 * usize::from(cfg!(feature = "context"))
    );
}

#[test]
fn option_failure_has_the_absent_value_root_with_or_without_context() {
    for error in [
        None::<()>.context("required").unwrap_err(),
        None::<()>.with_context(|| "required").unwrap_err(),
    ] {
        assert_eq!(error.to_string(), "An `Option` was unexpectedly `None`");
        assert!(error.source().is_none());
        assert_eq!(error.into_single(), eros::AbsentValueError);
    }
}

#[cfg(feature = "user_context")]
mod user_context {
    use super::*;

    macro_rules! user_contract {
        ($test:ident, $success:expr, $failure:expr) => {
            #[test]
            fn $test() {
                let calls = Cell::new(0);
                let conversions = Cell::new(0);
                assert_eq!(
                    ($success)
                        .user_context(CountConversion(&conversions))
                        .unwrap(),
                    "success"
                );
                assert_eq!(
                    ($success)
                        .with_user_context(|| {
                            calls.set(calls.get() + 1);
                            CountConversion(&conversions)
                        })
                        .unwrap(),
                    "success"
                );
                assert_eq!(calls.get(), 0);
                assert_eq!(conversions.get(), 0);
                let error = ($failure)
                    .user_context("first")
                    .with_user_context(|| {
                        calls.set(calls.get() + 1);
                        CountConversion(&conversions)
                    })
                    .unwrap_err();
                assert_eq!(calls.get(), 1);
                assert_eq!(conversions.get(), 1);
                assert_eq!(
                    error
                        .contexts()
                        .filter(|frame| frame.is_user_context())
                        .map(ToString::to_string)
                        .collect::<Vec<_>>(),
                    ["first", "counted context"]
                );
            }
        };
    }

    user_contract!(
        plain_result,
        Ok::<_, MsgError>("success"),
        Err::<(), _>(MsgError::from("root"))
    );
    user_contract!(
        typed_result,
        Ok::<_, ErrorUnion<(MsgError,)>>("success"),
        Err::<(), ErrorUnion<(MsgError,)>>(ErrorUnion::new(MsgError::from("root")))
    );
    user_contract!(
        erased_result,
        Ok::<_, ErrorUnion>("success"),
        Err::<(), ErrorUnion>(eros::error!("root"))
    );
    user_contract!(option, Some("success"), None::<()>);

    #[test]
    fn direct_contexts_filter_internal_details_and_survive_mapping_and_erasure() {
        let error: ErrorUnion<(MsgError,)> = ErrorUnion::new(MsgError::from("root"));
        assert_eq!(
            error
                .contexts()
                .filter(|frame| frame.is_user_context())
                .count(),
            0
        );
        let calls = Cell::new(0);
        let boxed: Box<dyn SendSyncDebugDisplay> = Box::new(String::from("public error"));
        let error = error
            .context("internal before")
            .user_context("public first")
            .with_user_context(|| {
                calls.set(calls.get() + 1);
                String::from("public second")
            })
            .context("internal between")
            .user_context(boxed)
            .context("internal after");
        assert_eq!(calls.get(), 1);
        let error = error
            .map_single(|error| error)
            .map_inner(|_| MsgError::from("replacement"));
        let error: ErrorUnion = error.into();
        assert_eq!(
            error
                .contexts()
                .filter(|frame| frame.is_user_context())
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            ["public first", "public second", "public error"]
        );
        assert!(
            error
                .contexts()
                .filter(|frame| frame.is_user_context())
                .last()
                .unwrap()
                .value()
                .as_boxed()
                .is_some()
        );
    }
}
