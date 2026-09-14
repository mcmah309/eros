use eros::{Context, ContextSource, ErrorUnion, SendSyncError, StrError};
use std::{borrow::Cow, cell::Cell};

#[test]
fn context_sources_preserve_storage_and_error_identity() {
    for source in [
        ContextSource::from("static"),
        Cow::Borrowed("static").into(),
    ] {
        assert!(matches!(source, ContextSource::Static("static")));
        assert_eq!(source.to_string(), "static");
        assert_eq!(format!("{source:?}"), "Static(\"static\")");
    }
    for source in [
        ContextSource::from(String::from("owned")),
        Cow::<str>::Owned(String::from("owned")).into(),
    ] {
        assert!(matches!(&source, ContextSource::Owned(value) if value == "owned"));
        assert_eq!(source.to_string(), "owned");
    }
    let boxed: Box<dyn SendSyncError> = Box::new(StrError::from("error"));
    let ptr = boxed.as_ref() as *const dyn SendSyncError as *const ();
    let source = ContextSource::from(boxed);
    assert_eq!(source.to_string(), "error");
    let ContextSource::Error(error) = source else {
        panic!("expected an error context")
    };
    assert_eq!(error.as_ref() as *const dyn SendSyncError as *const (), ptr);
    assert!(error.as_ref().as_any().is::<StrError>());
}

// Conversion must be deferred too: even eager context values should only be
// converted on failures when context recording is enabled.
struct CountConversion<'a>(&'a Cell<usize>);

impl From<CountConversion<'_>> for ContextSource {
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
    Ok::<_, StrError>("success"),
    Err::<(), _>(StrError::from("root"))
);
context_contract!(
    typed_result_context_is_lazy,
    Ok::<_, ErrorUnion<(StrError,)>>("success"),
    Err::<(), ErrorUnion<(StrError,)>>(ErrorUnion::new(StrError::from("root")))
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
        #[cfg(feature = "context")]
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
                        .user_contexts()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>(),
                    ["first", "counted context"]
                );
            }
        };
    }

    user_contract!(
        plain_result,
        Ok::<_, StrError>("success"),
        Err::<(), _>(StrError::from("root"))
    );
    user_contract!(
        typed_result,
        Ok::<_, ErrorUnion<(StrError,)>>("success"),
        Err::<(), ErrorUnion<(StrError,)>>(ErrorUnion::new(StrError::from("root")))
    );
    user_contract!(
        erased_result,
        Ok::<_, ErrorUnion>("success"),
        Err::<(), ErrorUnion>(eros::error!("root"))
    );
    user_contract!(option, Some("success"), None::<()>);

    #[test]
    fn direct_contexts_filter_internal_details_and_survive_mapping_and_erasure() {
        let error: ErrorUnion<(StrError,)> = ErrorUnion::new(StrError::from("root"));
        assert_eq!(error.user_contexts().count(), 0);
        let calls = Cell::new(0);
        let boxed: Box<dyn SendSyncError> = Box::new(StrError::from("public error"));
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
        assert_eq!(error.latest_error().to_string(), "public error");
        let error = error
            .map(|error| error)
            .map_root(|_| StrError::from("replacement"));
        let error: ErrorUnion = error.into();
        assert_eq!(
            error
                .user_contexts()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            ["public first", "public second", "public error"]
        );
        assert!(matches!(
            error.user_contexts().last(),
            Some(ContextSource::Error(_))
        ));
    }
}
