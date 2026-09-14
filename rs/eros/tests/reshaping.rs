use eros::{E2, ErrorUnion, IntoDynUnion, IntoUnion, ReshapeUnion, SendSyncError, StrError};
use std::fmt;

type Pair = (StrError, fmt::Error);
type Triple = (StrError, fmt::Error, std::io::Error);

#[test]
fn subset_rejects_errors_outside_the_requested_types() {
    let error: ErrorUnion<Triple> = ErrorUnion::new(StrError::from("original"));
    let error = error.context("preserved context");
    let report = format!("{error:?}");
    let result = error.subset::<(fmt::Error,), _>();
    // Check the branch before accessing the typed value: an incorrect Ok would
    // violate the invariant used by the enum conversions' unchecked downcasts.
    assert!(result.is_err(), "a StrError is not a fmt::Error");
    let remainder: ErrorUnion<(StrError, std::io::Error)> = result.unwrap_err();
    assert_eq!(format!("{remainder:?}"), report);
    assert_eq!(
        remainder.narrow::<StrError, _>().unwrap().as_str(),
        "original"
    );
}

#[test]
fn subset_accepts_each_member_in_requested_order() {
    let errors: [ErrorUnion<Triple>; 2] = [
        ErrorUnion::new(StrError::from("message")),
        ErrorUnion::new(fmt::Error),
    ];
    for (index, error) in errors.into_iter().enumerate() {
        let error = error.context("context");
        let report = format!("{error:?}");
        let subset = error.subset::<(fmt::Error, StrError), _>().unwrap();
        assert_eq!(format!("{subset:?}"), report);
        match (index, subset.to_enum()) {
            (0, E2::B(error)) => assert_eq!(error.as_str(), "message"),
            (1, E2::A(fmt::Error)) => {}
            _ => panic!("subset changed the active variant"),
        }
    }
}

#[test]
fn empty_subset_always_returns_the_original_union() {
    let error: ErrorUnion<Pair> = ErrorUnion::new(StrError::from("original"));
    let result = error.subset::<(), _>();
    assert!(result.is_err());
    let remainder: ErrorUnion<Pair> = result.unwrap_err();
    assert_eq!(remainder.to_string(), "original");
}

#[test]
fn full_subset_can_be_reordered_and_has_an_empty_remainder() {
    let error: ErrorUnion<Pair> = ErrorUnion::new(fmt::Error);
    let subset: Result<ErrorUnion<(fmt::Error, StrError)>, ErrorUnion<()>> = error.subset();
    assert!(matches!(subset.unwrap().to_enum(), E2::A(fmt::Error)));
}

#[test]
fn result_narrow_covers_success_matching_error_and_remainder() {
    let success: eros::Result<String, Pair> = Ok("value".into());
    assert_eq!(
        success.narrow::<StrError, _>().unwrap_err().unwrap(),
        "value"
    );

    let matching: eros::Result<(), Pair> = Err(ErrorUnion::new(StrError::from("match")));
    assert_eq!(matching.narrow::<StrError, _>().unwrap().as_str(), "match");

    let other: ErrorUnion<Pair> = ErrorUnion::new(fmt::Error);
    let other = other.context("retain me");
    let report = format!("{other:?}");
    let result: eros::Result<(), Pair> = Err(other);
    let remainder: eros::Result<(), (fmt::Error,)> = result.narrow::<StrError, _>().unwrap_err();
    let error = remainder.unwrap_err();
    assert_eq!(format!("{error:?}"), report);
    assert_eq!(error.into_single(), fmt::Error);
}

#[test]
fn result_conversions_preserve_success_values_and_error_metadata() {
    let success: Result<String, StrError> = Ok("typed".into());
    let success: eros::Result<_, Pair> = success.into_union();
    let success: eros::Result<_, Triple> = success.widen();
    assert_eq!(success.into_dyn_union().unwrap(), "typed");

    let success: Result<String, StrError> = Ok("erased".into());
    assert_eq!(success.into_dyn_union().unwrap(), "erased");

    let error: eros::Result<(), Pair> = Err(StrError::from("typed")).into_union();
    assert!(error.as_ref().unwrap_err().is_inner::<StrError>());
    let error = error.unwrap_err().context("metadata");
    let report = format!("{error:?}");
    let result: eros::Result<(), Pair> = Err(error);
    let widened: eros::Result<(), Triple> = result.widen();
    let erased = widened.into_dyn_union().unwrap_err();
    assert_eq!(format!("{erased:?}"), report);
    let result: eros::Result<()> = Err(erased);
    assert_eq!(
        format!("{:?}", result.into_dyn_union().unwrap_err()),
        report
    );

    let erased: eros::Result<()> = Err(StrError::from("erased")).into_dyn_union();
    assert_eq!(
        erased
            .unwrap_err()
            .downcast_inner::<StrError>()
            .unwrap()
            .as_str(),
        "erased"
    );
}

#[test]
fn singleton_borrowing_and_mapping_preserve_owned_values_and_metadata() {
    let mut error: ErrorUnion<(StrError,)> = StrError::from("before").into();
    assert_eq!(error.as_str(), "before"); // Deref
    assert!(std::ptr::eq::<StrError>(&*error, error.as_ref()));
    *error.as_mut() = StrError::from(String::from("after"));
    let error = error.context("mapping");
    let report = format!("{error:?}");
    let mut calls = 0;
    let mapped = error.map(|error| {
        calls += 1;
        assert_eq!(error.as_str(), "after");
        error
    });
    assert_eq!(calls, 1);
    assert_eq!(format!("{mapped:?}"), report);
    assert_eq!(mapped.into_single().as_str(), "after");

    #[derive(Debug)]
    struct Converted(StrError);
    impl fmt::Display for Converted {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.fmt(f)
        }
    }
    impl std::error::Error for Converted {}
    impl From<StrError> for Converted {
        fn from(error: StrError) -> Self {
            Self(error)
        }
    }
    let from_convertible: ErrorUnion<(Converted,)> = StrError::from("converted").into();
    assert_eq!(from_convertible.into_single().0.as_str(), "converted");
}

#[test]
fn mutable_erased_root_and_boxed_error_trait_refer_to_the_actual_error() {
    let mut error: ErrorUnion = ErrorUnion::new(StrError::from("before"));
    let root: &mut dyn std::any::Any = error.inner_mut();
    *root.downcast_mut::<StrError>().unwrap() = StrError::from("after");
    assert_eq!(
        error
            .inner_ref()
            .as_any()
            .downcast_ref::<StrError>()
            .unwrap()
            .as_str(),
        "after"
    );
    assert!(error.source().is_none());

    let boxed = error.into_inner();
    assert!(boxed.as_any().is::<Box<dyn eros::SendSyncError>>());
    assert!(boxed.as_ref().as_any().is::<StrError>());
    let source = std::error::Error::source(&boxed).unwrap();
    assert_eq!(source.downcast_ref::<StrError>().unwrap().as_str(), "after");
}

#[test]
fn failed_native_adapter_downcast_retains_the_original_adapter() {
    let error: ErrorUnion<Pair> = ErrorUnion::new(StrError::from("original"));
    let error = error.context("retained context");
    let report = format!("{error:?}");
    let adapter = error.into_dyn_error();
    let original = &*adapter as *const dyn eros::SendSyncError as *const ();
    let adapter = ErrorUnion::<(StrError,)>::from_dyn_error(adapter).unwrap_err();
    assert_eq!(
        &*adapter as *const dyn eros::SendSyncError as *const (),
        original
    );
    let recovered = ErrorUnion::<Pair>::from_dyn_error(adapter).unwrap();
    assert_eq!(format!("{recovered:?}"), report);
}

#[test]
fn public_unions_and_adapters_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ErrorUnion>();
    assert_send_sync::<ErrorUnion<Pair>>();
    assert_send_sync::<Box<dyn eros::SendSyncError>>();
}
