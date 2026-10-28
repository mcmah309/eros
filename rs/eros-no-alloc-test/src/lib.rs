#![no_std]

pub mod classify;

use core::{
    error::Error,
    fmt::{self, Write},
    sync::atomic::{AtomicUsize, Ordering},
};
use eros::{ErrorUnion, IntoUnion, ReshapeUnion};

#[path = "../../eros/tests/no_alloc/classification.rs"]
mod classification;
#[path = "../../eros/tests/no_alloc/composition.rs"]
mod composition;
#[path = "../../eros/tests/common/payloads.rs"]
mod payloads;
// Shared tests also exercise Eros's alloc configuration in its own harness.
#[allow(unexpected_cfgs)]
#[path = "../../eros/tests/no_alloc/messages.rs"]
mod messages;

static DROPS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
#[repr(align(256))]
struct Failure;
impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("failure\r")?;
        f.write_str("\n")?;
        f.write_str("detail\n")
    }
}
impl Error for Failure {}
impl Drop for Failure {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

#[eros::error_enum(NamedError)]
#[eros::error_enum_ref(NamedRef)]
#[eros::error_enum_mut(NamedMut)]
type Failures = (Failure, fmt::Error);

struct Buffer {
    bytes: [u8; 256],
    len: usize,
}
impl Buffer {
    fn new() -> Self {
        Self {
            bytes: [0; 256],
            len: 0,
        }
    }
    fn contents(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}
impl Write for Buffer {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.len + text.len();
        if end > self.bytes.len() {
            return Err(fmt::Error);
        }
        self.bytes[self.len..end].copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}

/// Exercised both by the host tests and by a binary with no global allocator.
pub fn run_checks() {
    classify::run_checks();
    classification::run_checks();
    // Host tests discover these scenarios individually. Freestanding builds
    // execute them here, including their ownership counters, once per run.
    #[cfg(not(test))]
    {
        payloads::payload_values_survive_mutation_moving_reshaping_and_enum_extraction();
        payloads::full_word_payload_survives_multiple_moves_and_shared_interior_mutation();
        payloads::thin_static_reference_preserves_pointer_provenance_and_source();
        payloads::non_copy_owners_drop_once_after_recovery_mapping_and_extraction();
    }
    messages::run_checks();
    composition::run_checks();
    let mut union: ErrorUnion<Failures> = ErrorUnion::new(Failure);
    #[cfg(feature = "error_location")]
    let location = union.location();
    assert!(union.is_inner::<Failure>());
    assert!(union.downcast_inner_ref::<u64>().is_none());
    assert!(matches!(NamedRef::from(&union), NamedRef::Failure(_)));
    assert!(matches!(NamedMut::from(&mut union), NamedMut::Failure(_)));
    let mut buffer = Buffer::new();
    write!(&mut buffer, "{union}").unwrap();
    assert_eq!(buffer.contents(), b"failure\ndetail");
    let mut buffer = Buffer::new();
    write!(&mut buffer, "{union:#?}").unwrap();
    assert_eq!(buffer.contents(), b"error:\n  failure\n  detail");
    let mut buffer = Buffer::new();
    write!(&mut buffer, "{union:?}").unwrap();
    let mut expected = Buffer::new();
    write!(&mut expected, "error:\n  failure\n  detail").unwrap();
    #[cfg(feature = "error_location")]
    write!(&mut expected, "\nlocation: {location}").unwrap();
    assert_eq!(buffer.contents(), expected.contents());
    let union = union.narrow::<(Failure,), _>().unwrap();
    let adapter = eros::ErrorUnionAdapter::from(union);
    let mut buffer = Buffer::new();
    write!(&mut buffer, "{adapter}").unwrap();
    let union = adapter.union();
    #[cfg(feature = "error_location")]
    assert!(core::ptr::eq(location, union.location()));
    let before = DROPS.load(Ordering::SeqCst);
    let mapped = union.map_single(|_failure| fmt::Error);
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 1);
    #[cfg(feature = "error_location")]
    assert!(core::ptr::eq(location, mapped.location()));
    let _ = mapped.into_single();

    let result: eros::Result<(), Failures> = Err::<(), _>(Failure).union();
    result
        .recover(|_: ErrorUnion<(Failure,)>| ())
        .recover(|_: ErrorUnion<(fmt::Error,)>| ())
        .into_value();
    let union: ErrorUnion<Failures> = ErrorUnion::new(Failure);
    let named: NamedError = union.into();
    assert!(matches!(named, NamedError::Failure(_)));
    drop(named);
    let erased = eros::error!(Failure);
    let before = DROPS.load(Ordering::SeqCst);
    let extracted = erased.downcast_inner::<Failure>().unwrap();
    assert_eq!(DROPS.load(Ordering::SeqCst), before);
    drop(extracted);
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 1);
}

#[cfg(test)]
mod tests {
    #[test]
    fn public_apis_work_without_alloc() {
        super::run_checks();
    }
}
