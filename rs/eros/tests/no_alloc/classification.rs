use core::{
    error::Error,
    fmt,
    sync::atomic::{AtomicUsize, Ordering},
};
use eros::{ErrorUnion, ReshapeUnion};

static DROPS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct ImplementationError;
impl fmt::Display for ImplementationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("implementation failure")
    }
}
impl Error for ImplementationError {}
impl Drop for ImplementationError {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

#[derive(Debug)]
struct DomainError;
impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("domain failure")
    }
}
impl Error for DomainError {}

type PublicErrors = (fmt::Error, DomainError);

// Also runs in the freestanding binary, without a global allocator.
pub fn run_checks() {
    let known: ErrorUnion = ErrorUnion::new(fmt::Error);
    #[cfg(feature = "error_location")]
    let location = known.location();
    let kept: ErrorUnion<PublicErrors> = known.classify(|_| -> DomainError {
        panic!("represented types must bypass classification")
    });
    assert!(kept.is_inner::<fmt::Error>());
    #[cfg(feature = "error_location")]
    assert!(core::ptr::eq(location, kept.location()));

    let before = DROPS.load(Ordering::SeqCst);
    let unknown: ErrorUnion = ErrorUnion::new(ImplementationError);
    #[cfg(feature = "error_location")]
    let location = unknown.location();
    let mut calls = 0;
    let classified: ErrorUnion<PublicErrors> = unknown.classify(|original| {
        calls += 1;
        assert!((original as &dyn core::any::Any).is::<ImplementationError>());
        assert_eq!(DROPS.load(Ordering::SeqCst), before);
        DomainError
    });
    assert_eq!(calls, 1);
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 1);
    assert!(classified.is_inner::<DomainError>());
    #[cfg(feature = "error_location")]
    assert!(core::ptr::eq(location, classified.location()));

    let success: eros::Result<u8> = Ok(7);
    let success: eros::Result<u8, PublicErrors> = success.classify(|_| -> DomainError {
        panic!("successes must bypass classification")
    });
    assert_eq!(success.unwrap(), 7);

    let failure: eros::Result<u8> = Err(ErrorUnion::new(ImplementationError));
    let failure: eros::Result<u8, PublicErrors> = failure.classify(|_| DomainError);
    assert!(failure.unwrap_err().is_inner::<DomainError>());
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 2);

    let failure: eros::Result<u8> = Err(ErrorUnion::new(fmt::Error));
    let failure: eros::Result<u8, PublicErrors> = failure.classify(|_| -> DomainError {
        panic!("represented result errors must bypass classification")
    });
    assert!(failure.unwrap_err().is_inner::<fmt::Error>());
}
