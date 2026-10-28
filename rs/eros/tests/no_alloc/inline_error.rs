use core::{
    any::TypeId,
    fmt,
    fmt::Write,
    sync::atomic::{AtomicUsize, Ordering},
};
use eros::{Erase, ErrorUnion, InnerError, IntoUnion, ReshapeUnion, TypeSet};

/// An error that can be constructed and recovered by downstream crates.
#[eros::inline_error("operation timed out")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Timeout;

#[eros::inline_error("operation timed out")]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct OtherError;

#[eros::inline_error("device unavailable")]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceUnavailable;

#[eros::inline_error("device operation failed")]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DomainFailure;

#[eros::inline_error("device failed: code {code:04x}", code = self.code)]
#[derive(PartialEq, Eq)]
pub struct DeviceFailure {
    pub code: usize,
}

#[eros::inline_error(
    "device {kind}: {code:04x}",
    kind = match self { Self::Disconnected => "disconnected", Self::InvalidCode(_) => "invalid code" },
    code = match self { Self::Disconnected => 0, Self::InvalidCode(code) => *code },
)]
#[derive(PartialEq, Eq)]
#[repr(u8)]
pub enum DeviceStatus {
    Disconnected,
    InvalidCode(u16),
}

#[derive(Debug)]
pub struct Details {
    pub message: &'static str,
    pub bytes: [u8; 64],
}

pub static DETAILS: Details = Details {
    message: "static details",
    bytes: [7; 64],
};

#[eros::inline_error("device failed: {}", self.0.message)]
pub struct StaticFailure(pub &'static Details);

#[eros::inline_error("atomic device failure: {}", self.0.load(Ordering::SeqCst))]
pub struct AtomicFailure(pub AtomicUsize);

#[eros::inline_error]
pub struct ManualDisplay {
    pub code: usize,
}

impl fmt::Display for ManualDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "manual {:04x}", self.code)
    }
}

#[eros::inline_error()]
#[repr(u8)]
enum ManualStatus {
    Disconnected,
    InvalidCode(u16),
}

impl fmt::Display for ManualStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disconnected => f.write_str("manual disconnected"),
            Self::InvalidCode(code) => write!(f, "manual invalid {code}"),
        }
    }
}

#[eros::inline_error("tracked device failure")]
struct TrackedFailure(&'static AtomicUsize);

impl Drop for TrackedFailure {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

pub type SelectedPayloads = (
    StaticFailure,
    DeviceStatus,
    DeviceFailure,
    AtomicFailure,
    ManualDisplay,
);
pub type PayloadErrors = (
    DomainFailure,
    DeviceFailure,
    AtomicFailure,
    DeviceStatus,
    StaticFailure,
    ManualDisplay,
);
pub type ClassifiedPayloads = (DomainFailure, DeviceFailure);

fn payload_source(mode: u8) -> eros::Result<u8, SelectedPayloads> {
    match mode {
        0 => Err(DeviceFailure { code: 41 }).union(),
        1 => Err(DeviceStatus::Disconnected).union(),
        2 => Err(DeviceStatus::InvalidCode(513)).union(),
        3 => Err(StaticFailure(&DETAILS)).union(),
        4 => Err(AtomicFailure(AtomicUsize::new(41))).union(),
        7 => Err(ManualDisplay { code: 42 }).union(),
        _ => Ok(42),
    }
}

pub fn payload_pipeline(mode: u8) -> eros::Result<u8, PayloadErrors> {
    let value = payload_source(mode).widen()?;
    eros::ensure!(mode != 6, DomainFailure);
    Ok(value + 1)
}

pub fn erase_payload_pipeline(mode: u8) -> eros::Result<u8> {
    payload_pipeline(mode).erase()
}

pub fn restore_payloads(
    result: eros::Result<u8>,
) -> Result<ErrorUnion<SelectedPayloads>, eros::Result<u8>> {
    result.narrow::<SelectedPayloads, _>()
}

pub fn widen_payloads(error: ErrorUnion<SelectedPayloads>) -> ErrorUnion<PayloadErrors> {
    error.widen()
}

fn payload_domain(inner: InnerError) -> DeviceFailure {
    let code = if let Some(error) = inner.downcast_ref::<DeviceFailure>() {
        error.code + 1
    } else if let Some(error) = inner.downcast_ref::<DeviceStatus>() {
        match error {
            DeviceStatus::Disconnected => 2,
            DeviceStatus::InvalidCode(code) => usize::from(*code),
        }
    } else if let Some(error) = inner.downcast_ref::<StaticFailure>() {
        assert!(core::ptr::eq(error.0, &DETAILS));
        usize::from(error.0.bytes[0])
    } else if let Some(error) = inner.downcast_ref::<AtomicFailure>() {
        error.0.load(Ordering::SeqCst) + 1
    } else if let Some(error) = inner.downcast_ref::<ManualDisplay>() {
        error.code + 1
    } else {
        assert!(inner.is::<DomainFailure>());
        0
    };
    DeviceFailure { code }
}

pub fn classify_payload<E: TypeSet>(
    result: eros::Result<u8, E>,
) -> eros::Result<u8, ClassifiedPayloads> {
    result.classify(payload_domain)
}

pub fn classify_payload_error<E: TypeSet>(error: ErrorUnion<E>) -> ErrorUnion<ClassifiedPayloads> {
    error.classify(payload_domain)
}

fn assert_payload(error: &ErrorUnion<impl TypeSet>, mode: u8) {
    let message = match mode {
        0 => {
            assert_eq!(
                error.downcast_inner_ref::<DeviceFailure>().unwrap().code,
                41
            );
            "device failed: code 0029"
        }
        1 | 2 => {
            assert_eq!(
                error.downcast_inner_ref::<DeviceStatus>().unwrap(),
                &if mode == 1 {
                    DeviceStatus::Disconnected
                } else {
                    DeviceStatus::InvalidCode(513)
                }
            );
            if mode == 1 {
                "device disconnected: 0000"
            } else {
                "device invalid code: 0201"
            }
        }
        3 => {
            let details = error.downcast_inner_ref::<StaticFailure>().unwrap().0;
            assert!(core::ptr::eq(details, &DETAILS));
            assert_eq!(details.message, "static details");
            assert_eq!(details.bytes, [7; 64]);
            "device failed: static details"
        }
        4 => {
            assert_eq!(
                error
                    .downcast_inner_ref::<AtomicFailure>()
                    .unwrap()
                    .0
                    .load(Ordering::SeqCst),
                41
            );
            "atomic device failure: 41"
        }
        6 => {
            assert!(error.is_inner::<DomainFailure>());
            "device operation failed"
        }
        7 => {
            assert_eq!(
                error.downcast_inner_ref::<ManualDisplay>().unwrap().code,
                42
            );
            "manual 002a"
        }
        _ => panic!("success must not produce an error"),
    };
    assert_display(error, message);
}

#[cfg_attr(test, test)]
pub fn payloads_compose_across_function_boundaries_without_allocation() {
    assert_eq!(size_of::<DeviceFailure>(), size_of::<usize>());
    assert_eq!(size_of::<StaticFailure>(), size_of::<usize>());
    assert_eq!(size_of::<AtomicFailure>(), size_of::<usize>());
    assert!(size_of::<Details>() > size_of::<usize>());
    for mode in 0..=7 {
        let input = erase_payload_pipeline(mode);
        if mode == 5 {
            assert_eq!(
                classify_payload(restore_payloads(input).unwrap_err()).unwrap(),
                43
            );
            continue;
        }
        assert_payload(input.as_ref().unwrap_err(), mode);
        #[cfg(feature = "error_location")]
        let location = input.as_ref().unwrap_err().location();
        let error = match restore_payloads(input) {
            Ok(selected) => {
                assert!(mode < 5 || mode == 7);
                assert_payload(&selected, mode);
                widen_payloads(selected).erase()
            }
            Err(remaining) => {
                assert_eq!(mode, 6);
                remaining.unwrap_err()
            }
        };
        assert_payload(&error, mode);
        let error = error
            .narrow::<SelectedPayloads, _>()
            .map(|selected| widen_payloads(selected).erase())
            .unwrap_or_else(|remaining| remaining);
        assert_payload(&error, mode);
        let result = classify_payload(Err(error));
        let classified = result.unwrap_err();
        #[cfg(feature = "error_location")]
        assert!(core::ptr::eq(location, classified.location()));
        let expected_message = match mode {
            0 | 4 => "device failed: code 002a",
            1 => "device failed: code 0002",
            2 => "device failed: code 0201",
            3 => "device failed: code 0007",
            6 => "device failed: code 0000",
            7 => "device failed: code 002b",
            _ => unreachable!(),
        };
        assert_display(&classified, expected_message);
        let expected = [42, 2, 513, 7, 42, 0, 0, 43][usize::from(mode)];
        assert_eq!(
            classified.downcast_inner::<DeviceFailure>().unwrap().code,
            expected
        );

        let classified = classify_payload(payload_pipeline(mode)).unwrap_err();
        assert_eq!(
            classified.narrow::<DeviceFailure, _>().unwrap().code,
            expected
        );
        let classified = classify_payload_error(payload_pipeline(mode).unwrap_err());
        assert_eq!(
            classified.downcast_inner::<DeviceFailure>().unwrap().code,
            expected
        );
        let classified = classify_payload_error(erase_payload_pipeline(mode).unwrap_err());
        assert_eq!(
            classified.downcast_inner::<DeviceFailure>().unwrap().code,
            expected
        );
    }
}

#[cfg_attr(test, test)]
pub fn non_copy_payloads_are_mutable_and_drop_once() {
    static DROPS: AtomicUsize = AtomicUsize::new(0);
    let before = DROPS.load(Ordering::SeqCst);
    let error: ErrorUnion<(TrackedFailure, DeviceFailure)> =
        ErrorUnion::new(TrackedFailure(&DROPS));
    let error = error.erase().narrow::<(TrackedFailure,), _>().unwrap();
    assert_eq!(DROPS.load(Ordering::SeqCst), before);
    let error = error.downcast_inner::<TrackedFailure>().unwrap();
    drop(error);
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 1);

    let error: ErrorUnion<(TrackedFailure,)> = ErrorUnion::new(TrackedFailure(&DROPS));
    let classified: ErrorUnion<(DeviceFailure,)> = error.classify(|inner| {
        assert!(inner.is::<TrackedFailure>());
        DeviceFailure { code: 7 }
    });
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 2);
    assert_eq!(classified.into_single().code, 7);
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 2);

    let mut error = erase_payload_pipeline(0).unwrap_err();
    error.downcast_inner_mut::<DeviceFailure>().unwrap().code = 42;
    assert_display(&error, "device failed: code 002a");
    error.downcast_inner_mut::<DeviceFailure>().unwrap().code = usize::MAX;
    assert_eq!(error.narrow::<DeviceFailure, _>().unwrap().code, usize::MAX);

    let error = erase_payload_pipeline(4).unwrap_err();
    error
        .downcast_inner_ref::<AtomicFailure>()
        .unwrap()
        .0
        .fetch_add(1, Ordering::SeqCst);
    assert_display(&error, "atomic device failure: 42");
    let error = error.narrow::<(AtomicFailure,), _>().unwrap();
    assert_eq!(error.into_single().0.into_inner(), 42);
}

#[eros::inline_error("{label}: {0:>width$} {{ok}} {self:?}", self.0, label = "code", width = 4)]
struct Formatted(u8);

#[eros::inline_error("trailing comma")]
struct TrailingComma;

#[cfg_attr(test, test)]
pub fn format_arguments_specifiers_captures_and_escaping_work() {
    assert_display(&Formatted(7), "code:    7 {ok} Formatted(7)");
    assert_display(&eros::error!(Formatted(7)), "code:    7 {ok} Formatted(7)");
    assert_display(&TrailingComma, "trailing comma");
    assert_display(
        &eros::error!(ManualStatus::Disconnected),
        "manual disconnected",
    );
    assert_display(
        &eros::error!(ManualStatus::InvalidCode(9)),
        "manual invalid 9",
    );
}

#[eros::inline_error("literal {{braces}}: 日本語")]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(align(256))]
struct AlignedError;

mod other {
    #[eros::inline_error("operation timed out")]
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub(super) struct Timeout;
}

// Disabled declarations must also disable all generated implementations.
#[eros::inline_error("disabled")]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg(any())]
struct Timeout;

#[eros::inline_error("disabled")]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(all(), cfg(any()), derive(Default))]
struct Timeout;

struct Buffer {
    bytes: [u8; 128],
    len: usize,
}

impl Write for Buffer {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.len + text.len();
        let dest = self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?;
        dest.copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}

fn assert_display(value: &impl fmt::Display, expected: &str) {
    let mut buffer = Buffer {
        bytes: [0; 128],
        len: 0,
    };
    write!(&mut buffer, "{value}").unwrap();
    assert_eq!(&buffer.bytes[..buffer.len], expected.as_bytes());
}

pub fn literal_error() -> ErrorUnion {
    eros::error!("message here")
}

fn classify_literal(inner: InnerError) -> DeviceFailure {
    assert_display(&inner, "message here");
    assert!(!inner.is::<DeviceFailure>());
    DeviceFailure { code: 42 }
}

fn classify_literal_result(result: eros::Result<u8>) -> eros::Result<u8, ClassifiedPayloads> {
    result.classify(classify_literal)
}

#[cfg_attr(test, test)]
pub fn literal_message_errors_compose_with_inline_errors() {
    let error = literal_error();
    assert_display(&error, "message here");
    let type_id = core::any::Any::type_id(error.inner());
    #[cfg(feature = "error_location")]
    let location = error.location();

    // A literal error is not one of our declared inline error types. Failed
    // narrowing and repeated erasure must retain its identity and message.
    let error = error.narrow::<(Timeout, DeviceFailure), _>().unwrap_err();
    let result: eros::Result<u8> = Err(error);
    let error = result.erase().erase().unwrap_err();
    assert_eq!(core::any::Any::type_id(error.inner()), type_id);
    assert_display(&error, "message here");

    // Classify the literal into a named inline error across a function boundary,
    // then widen, erase, and narrow it back without losing its payload.
    let typed = classify_literal_result(Err(error)).unwrap_err();
    assert_display(&typed, "device failed: code 002a");
    #[cfg(feature = "error_location")]
    assert!(core::ptr::eq(location, typed.location()));
    let widened: ErrorUnion<(DomainFailure, DeviceFailure, Timeout)> = typed.widen();
    let selected = widened.erase().narrow::<(DeviceFailure,), _>().unwrap();
    assert_eq!(selected.into_single().code, 42);

    let typed: ErrorUnion<(DeviceFailure,)> = literal_error().classify(classify_literal);
    assert_eq!(typed.downcast_inner::<DeviceFailure>().unwrap().code, 42);
    assert_eq!(classify_literal_result(Ok(7)).unwrap(), 7);
    assert_display(&eros::error!("message {{here}}"), "message {here}");
}

/// Returns a library-defined error across a crate boundary.
pub fn erased_timeout() -> ErrorUnion {
    let error: ErrorUnion<(Timeout,)> = ErrorUnion::new(Timeout);
    error.erase()
}

pub type SelectedErrors = (DeviceUnavailable, Timeout);
pub type PipelineErrors = (OtherError, Timeout, DeviceUnavailable);
pub type ClassifiedErrors = (Timeout, DomainFailure);

fn connect(mode: u8) -> Result<u8, Timeout> {
    if mode == 0 { Err(Timeout) } else { Ok(10) }
}

fn read(mode: u8) -> eros::Result<u8, (Timeout, DeviceUnavailable)> {
    let value = connect(mode).union()?;
    eros::ensure!(mode != 1, DeviceUnavailable);
    Ok(value + 1)
}

/// Widens and propagates generated errors returned by another function.
pub fn pipeline(mode: u8) -> eros::Result<u8, PipelineErrors> {
    let value = read(mode).widen()?;
    eros::ensure!(mode != 2, OtherError);
    Ok(value + 1)
}

pub fn erase_pipeline(mode: u8) -> eros::Result<u8> {
    pipeline(mode).erase()
}

pub fn select_result(
    result: eros::Result<u8>,
) -> Result<ErrorUnion<SelectedErrors>, eros::Result<u8>> {
    result.narrow::<SelectedErrors, _>()
}

pub fn select_error(error: ErrorUnion) -> Result<ErrorUnion<SelectedErrors>, ErrorUnion> {
    error.narrow::<SelectedErrors, _>()
}

pub fn widen_selected(error: ErrorUnion<SelectedErrors>) -> ErrorUnion<PipelineErrors> {
    error.widen()
}

pub fn erase_error(error: ErrorUnion<PipelineErrors>) -> ErrorUnion {
    error.erase()
}

fn classify_domain(inner: InnerError) -> DomainFailure {
    if inner.is::<Timeout>() {
        assert_eq!(inner.downcast::<Timeout>().unwrap(), Timeout);
    } else if inner.is::<DeviceUnavailable>() {
        assert_eq!(
            inner.downcast::<DeviceUnavailable>().unwrap(),
            DeviceUnavailable
        );
    } else {
        assert_eq!(inner.downcast::<OtherError>().unwrap(), OtherError);
    }
    DomainFailure
}

pub fn classify_error<E: TypeSet>(error: ErrorUnion<E>) -> ErrorUnion<ClassifiedErrors> {
    error.classify(classify_domain)
}

pub fn classify_result<E: TypeSet>(
    result: eros::Result<u8, E>,
) -> eros::Result<u8, ClassifiedErrors> {
    result.classify(classify_domain)
}

fn assert_pipeline_error(error: &ErrorUnion<impl TypeSet>, mode: u8) {
    let (type_id, message) = match mode {
        0 => (TypeId::of::<Timeout>(), "operation timed out"),
        1 => (TypeId::of::<DeviceUnavailable>(), "device unavailable"),
        2 => (TypeId::of::<OtherError>(), "operation timed out"),
        _ => panic!("success must not produce an error"),
    };
    assert_eq!(core::any::Any::type_id(error.inner()), type_id);
    assert_display(error, message);
}

#[cfg_attr(test, test)]
pub fn union_widen_narrow_classify_and_erase_compose_across_functions() {
    for mode in 0..=3 {
        // A plain Result enters a typed union, widens through another function,
        // and returns across an erased function boundary before narrowing.
        let result = erase_pipeline(mode);
        if mode == 3 {
            let result = select_result(result).unwrap_err();
            // Successful values bypass the named classifier.
            assert_eq!(classify_result(result).unwrap(), 12);
            continue;
        }
        assert_pipeline_error(result.as_ref().unwrap_err(), mode);
        #[cfg(feature = "error_location")]
        let location = result.as_ref().unwrap_err().location();

        let error = match select_result(result) {
            Ok(selected) => {
                assert!(mode < 2);
                assert_pipeline_error(&selected, mode);
                let widened = widen_selected(selected);
                assert_pipeline_error(&widened, mode);
                erase_error(widened)
            }
            Err(remaining) => {
                assert_eq!(mode, 2);
                remaining.unwrap_err()
            }
        };
        assert_pipeline_error(&error, mode);
        #[cfg(feature = "error_location")]
        assert!(core::ptr::eq(location, error.location()));

        // Exercise the union form of narrowing through a separate function,
        // keeping both the selected and unmatched branches intact.
        let error = match select_error(error) {
            Ok(selected) => erase_error(widen_selected(selected)),
            Err(remaining) => remaining,
        };
        assert_pipeline_error(&error, mode);
        #[cfg(feature = "error_location")]
        assert!(core::ptr::eq(location, error.location()));

        let classified = classify_error(error);
        assert!(classified.is_inner::<DomainFailure>());
        assert!(!classified.is_inner::<Timeout>());
        assert_display(&classified, "device operation failed");
        #[cfg(feature = "error_location")]
        assert!(core::ptr::eq(location, classified.location()));
        assert_eq!(
            classified.narrow::<DomainFailure, _>().unwrap(),
            DomainFailure
        );

        // The Result form must classify every failure, including Timeout,
        // which already belongs to the destination set.
        let result = erase_pipeline(mode);
        #[cfg(feature = "error_location")]
        let location = result.as_ref().unwrap_err().location();
        let classified = classify_result(result).unwrap_err();
        assert_display(&classified, "device operation failed");
        assert_eq!(
            core::any::Any::type_id(classified.inner()),
            TypeId::of::<DomainFailure>()
        );
        #[cfg(feature = "error_location")]
        assert!(core::ptr::eq(location, classified.location()));
        assert_eq!(
            classified.downcast_inner::<DomainFailure>().unwrap(),
            DomainFailure
        );

        // Typed inputs cross the same classification boundaries without first
        // being erased; their concrete source types must reach the callback.
        let classified = classify_error(pipeline(mode).unwrap_err());
        assert_eq!(
            classified.downcast_inner::<DomainFailure>().unwrap(),
            DomainFailure
        );
        let classified = classify_result(pipeline(mode)).unwrap_err();
        assert_display(&classified, "device operation failed");
        assert_eq!(
            classified.downcast_inner::<DomainFailure>().unwrap(),
            DomainFailure
        );
    }
}

#[cfg_attr(test, test)]
pub fn identity_display_and_downcasts_survive_erasure() {
    let type_id = TypeId::of::<Timeout>();
    assert_eq!(size_of::<Timeout>(), 0);
    assert_ne!(type_id, TypeId::of::<OtherError>());
    assert_ne!(type_id, TypeId::of::<other::Timeout>());

    let mut error = erased_timeout().downcast_inner::<OtherError>().unwrap_err();
    assert_display(&error, "operation timed out");
    assert_eq!(core::any::Any::type_id(error.inner()), type_id);
    assert!(error.downcast_inner_ref::<other::Timeout>().is_none());
    assert_eq!(error.downcast_inner_ref::<Timeout>(), Some(&Timeout));
    *error.downcast_inner_mut::<Timeout>().unwrap() = Timeout;
    assert_eq!(error.downcast_inner::<Timeout>().unwrap(), Timeout);

    let error = eros::error!(AlignedError);
    assert_eq!(size_of::<AlignedError>(), 0);
    assert_display(&error, "literal {braces}: 日本語");
    assert!(error.downcast_inner::<AlignedError>().is_ok());
}

#[cfg_attr(test, test)]
pub fn generated_traits_and_typed_control_flow_work() {
    fn assert_traits<T: eros::SendSyncError + Copy + Clone + Eq + core::hash::Hash>() {}
    assert_traits::<Timeout>();
    assert_eq!(Timeout::default(), Timeout);
    assert!(core::error::Error::source(&Timeout).is_none());
    let mut buffer = Buffer {
        bytes: [0; 128],
        len: 0,
    };
    write!(&mut buffer, "{Timeout:?}").unwrap();
    assert_eq!(&buffer.bytes[..buffer.len], b"Timeout");

    fn poll(ready: bool) -> eros::Result<(), (Timeout, OtherError)> {
        eros::ensure!(ready, Timeout);
        eros::bail!(OtherError);
    }

    let error = poll(false).unwrap_err().erase();
    assert_eq!(error.narrow::<Timeout, _>().unwrap(), Timeout);
    let result = poll(true)
        .recover::<Timeout, _>(|_| ())
        .recover::<OtherError, _>(|_| ())
        .into_value();
    assert_eq!(result, ());
}

#[cfg_attr(test, allow(dead_code))]
pub fn run_checks() {
    literal_message_errors_compose_with_inline_errors();
    format_arguments_specifiers_captures_and_escaping_work();
    payloads_compose_across_function_boundaries_without_allocation();
    #[cfg(not(test))]
    non_copy_payloads_are_mutable_and_drop_once();
    identity_display_and_downcasts_survive_erasure();
    generated_traits_and_typed_control_flow_work();
    union_widen_narrow_classify_and_erase_compose_across_functions();
}
