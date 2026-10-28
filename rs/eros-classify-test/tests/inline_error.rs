use core::any::TypeId;
use eros::{Erase, InnerError, IntoUnion, ReshapeUnion};
use eros_no_alloc_test::inline_error as library;

#[eros::inline_error("operation timed out")]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Timeout;

#[eros::inline_error("application device failed: {}", self.code)]
struct DeviceFailure {
    code: usize,
}

#[eros::inline_error("application static failure: {}", self.0.message)]
struct StaticFailure(&'static library::Details);

fn classify_library_payload(
    mode: u8,
) -> eros::Result<
    u8,
    (
        DeviceFailure,
        library::DeviceFailure,
        library::DomainFailure,
    ),
> {
    let result = match library::restore_payloads(library::erase_payload_pipeline(mode)) {
        Ok(selected) => Err(library::widen_payloads(selected).erase()),
        Err(remaining) => remaining,
    };
    library::classify_payload(result).widen()
}

#[test]
fn word_enum_and_static_reference_payloads_cross_crate_boundaries() {
    assert_ne!(
        TypeId::of::<DeviceFailure>(),
        TypeId::of::<library::DeviceFailure>()
    );
    let error = library::erase_payload_pipeline(3)
        .unwrap_err()
        .downcast_inner::<StaticFailure>()
        .unwrap_err();
    let original = error.downcast_inner::<library::StaticFailure>().unwrap();
    assert!(core::ptr::eq(original.0, &library::DETAILS));
    assert_eq!(original.0.bytes, [7; 64]);
    let local = eros::error!(StaticFailure(original.0));
    assert_eq!(
        local.to_string(),
        "application static failure: static details"
    );
    assert!(core::ptr::eq(
        local.downcast_inner::<StaticFailure>().unwrap().0,
        &library::DETAILS
    ));

    for mode in 0..=7 {
        let result = classify_library_payload(mode);
        if mode == 5 {
            assert_eq!(result.unwrap(), 43);
            continue;
        }
        let error = result.unwrap_err().erase();
        let expected = [42, 2, 513, 7, 42, 0, 0, 43][usize::from(mode)];
        assert_eq!(
            error.to_string(),
            format!("device failed: code {expected:04x}")
        );
        let error = error.narrow::<DeviceFailure, _>().unwrap_err();
        let original = error.narrow::<library::DeviceFailure, _>().unwrap();
        assert_eq!(original.code, expected);
        let local = eros::error!(DeviceFailure {
            code: original.code
        });
        assert_eq!(
            local.to_string(),
            format!("application device failed: {expected}")
        );
        assert_eq!(
            local.downcast_inner::<DeviceFailure>().unwrap().code,
            expected
        );
    }
}

type ApplicationErrors = (
    Timeout,
    library::Timeout,
    library::DeviceUnavailable,
    library::OtherError,
);

fn application_pipeline(mode: u8) -> eros::Result<u8, ApplicationErrors> {
    if mode == 4 {
        Err(Timeout).union()
    } else {
        library::pipeline(mode).widen()
    }
}

fn select_application(mode: u8) -> eros::Result<u8> {
    match library::select_result(application_pipeline(mode).erase()) {
        Ok(selected) => Err(library::erase_error(library::widen_selected(selected))),
        Err(remaining) => remaining,
    }
}

fn application_failure(inner: InnerError) -> Timeout {
    match inner.downcast::<library::DomainFailure>() {
        Ok(failure) => assert_eq!(failure, library::DomainFailure),
        Err(inner) => assert_eq!(inner.downcast::<Timeout>().unwrap(), Timeout),
    }
    Timeout
}

fn classify_application(result: eros::Result<u8>) -> eros::Result<u8, (Timeout,)> {
    let local_error = result
        .as_ref()
        .is_err_and(|error| error.is_inner::<Timeout>());
    let result = if local_error {
        result
    } else {
        library::classify_result(result).erase()
    };
    result.classify(application_failure)
}

#[test]
fn generated_errors_compose_across_library_and_application_functions() {
    for mode in 0..=4 {
        let result = select_application(mode);
        if mode == 3 {
            assert_eq!(classify_application(result).unwrap(), 12);
            continue;
        }
        let original = result.as_ref().unwrap_err();
        let (type_id, message) = match mode {
            0 => (TypeId::of::<library::Timeout>(), "operation timed out"),
            1 => (
                TypeId::of::<library::DeviceUnavailable>(),
                "device unavailable",
            ),
            2 => (TypeId::of::<library::OtherError>(), "operation timed out"),
            4 => (TypeId::of::<Timeout>(), "operation timed out"),
            _ => unreachable!(),
        };
        assert_eq!(core::any::Any::type_id(original.inner()), type_id);
        assert_eq!(original.to_string(), message);
        #[cfg(feature = "diagnostics")]
        let location = original.location();

        let error = classify_application(result).unwrap_err();
        assert_eq!(
            core::any::Any::type_id(error.inner()),
            TypeId::of::<Timeout>()
        );
        assert_eq!(error.to_string(), "operation timed out");
        #[cfg(feature = "diagnostics")]
        assert!(core::ptr::eq(location, error.location()));

        // The application and library errors share a name and message, but
        // narrowing in the library must reject the application's type.
        let error = library::select_error(error.erase()).unwrap_err();
        assert!(!error.is_inner::<library::Timeout>());
        assert_eq!(error.narrow::<Timeout, _>().unwrap(), Timeout);
    }
}

#[test]
fn identical_names_and_messages_in_different_crates_keep_distinct_identities() {
    assert_ne!(TypeId::of::<Timeout>(), TypeId::of::<library::Timeout>());
    let mut error = library::erased_timeout()
        .downcast_inner::<Timeout>()
        .unwrap_err();
    assert_eq!(error.to_string(), "operation timed out");
    assert_eq!(
        core::any::Any::type_id(error.inner()),
        TypeId::of::<library::Timeout>(),
    );
    assert_eq!(
        error.downcast_inner_ref::<library::Timeout>(),
        Some(&library::Timeout)
    );
    *error.downcast_inner_mut::<library::Timeout>().unwrap() = library::Timeout;
    assert_eq!(
        error.downcast_inner::<library::Timeout>().unwrap(),
        library::Timeout
    );

    let error = eros::error!(Timeout)
        .downcast_inner::<library::Timeout>()
        .unwrap_err();
    assert_eq!(error.to_string(), "operation timed out");
    assert_eq!(error.downcast_inner::<Timeout>().unwrap(), Timeout);
}
