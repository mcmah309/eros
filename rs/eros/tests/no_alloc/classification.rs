use core::{
    error::Error,
    fmt,
    sync::atomic::{AtomicUsize, Ordering},
};
use eros::{ErrorUnion, InnerError, ReshapeUnion};

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
    classify_word_payloads();
    classify_error_values();
    let known: ErrorUnion = ErrorUnion::new(fmt::Error);
    #[cfg(feature = "error_location")]
    let location = known.location();
    let kept: ErrorUnion<PublicErrors> = known.classify(|_: eros::InnerError| -> DomainError {
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
    let classified: ErrorUnion<PublicErrors> = unknown.classify(|original: eros::InnerError| {
        calls += 1;
        assert!(original.is::<ImplementationError>());
        assert_eq!(DROPS.load(Ordering::SeqCst), before);
        DomainError
    });
    assert_eq!(calls, 1);
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 1);
    assert!(classified.is_inner::<DomainError>());
    #[cfg(feature = "error_location")]
    assert!(core::ptr::eq(location, classified.location()));

    let success: eros::Result<u8> = Ok(7);
    let success: eros::Result<u8, PublicErrors> =
        success.classify(|_: eros::InnerError| -> DomainError {
            panic!("successes must bypass classification")
        });
    assert_eq!(success.unwrap(), 7);

    let failure: eros::Result<u8> = Err(ErrorUnion::new(ImplementationError));
    let failure: eros::Result<u8, PublicErrors> =
        failure.classify(|_: eros::InnerError| DomainError);
    assert!(failure.unwrap_err().is_inner::<DomainError>());
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 2);

    let failure: eros::Result<u8> = Err(ErrorUnion::new(fmt::Error));
    let failure: eros::Result<u8, PublicErrors> =
        failure.classify(|_: eros::InnerError| -> DomainError {
            panic!("represented result errors must bypass classification")
        });
    assert!(failure.unwrap_err().is_inner::<fmt::Error>());
}

#[derive(Debug, PartialEq, Eq)]
#[repr(usize)]
enum Status {
    Bus,
    Timeout,
}
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
impl Error for Status {}

#[derive(Debug)]
struct Code {
    error_code: usize,
}
impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "code {}", self.error_code)
    }
}
impl Error for Code {}

fn classify_code(original: InnerError) -> Status {
    let code = original.downcast_ref::<Code>().unwrap();
    if code.error_code == 0 {
        Status::Bus
    } else {
        Status::Timeout
    }
}

fn classify_word_payloads() {
    for code in [0, usize::MAX] {
        let error: ErrorUnion = ErrorUnion::new(Code { error_code: code });
        #[cfg(feature = "error_location")]
        let location = error.location();
        // A named callback fixes the public argument type across feature sets.
        let classified: ErrorUnion<(Status, fmt::Error)> = error.classify(classify_code);
        #[cfg(feature = "error_location")]
        assert!(core::ptr::eq(location, classified.location()));
        let status = classified.narrow::<Status, _>().unwrap();
        assert_eq!(
            status,
            if code == 0 {
                Status::Bus
            } else {
                Status::Timeout
            }
        );
    }
    // Distinct enum variants retain one concrete member of the tuple.
    for status in [Status::Bus, Status::Timeout] {
        let error: ErrorUnion<(Status, fmt::Error)> = ErrorUnion::new(status);
        let kept: ErrorUnion<(Status,)> = error.classify(|_: eros::InnerError| -> Status {
            panic!("all variants of a represented enum must pass through")
        });
        assert!(kept.is_inner::<Status>());
    }
}

fn classify_error_values() {
    let original: ErrorUnion = ErrorUnion::new(Code { error_code: 7 });
    #[cfg(feature = "error_location")]
    let location = original.location();
    let classified: ErrorUnion<PublicErrors> = original.classify(DomainError);
    assert!(classified.is_inner::<DomainError>());
    #[cfg(feature = "error_location")]
    assert!(core::ptr::eq(location, classified.location()));

    let original: ErrorUnion = ErrorUnion::new(DomainError);
    let classified: ErrorUnion<(Code,)> = original.classify(Code { error_code: 37 });
    assert_eq!(classified.into_single().error_code, 37);

    let success: eros::Result<u8> = Ok(7);
    let success: eros::Result<u8, PublicErrors> = success.classify(DomainError);
    assert_eq!(success.unwrap(), 7);

    let failure: eros::Result<u8> = Err(ErrorUnion::new(Code { error_code: 7 }));
    let failure: eros::Result<u8, PublicErrors> = failure.classify(DomainError);
    assert!(failure.unwrap_err().is_inner::<DomainError>());

    let known: eros::Result<u8> = Err(ErrorUnion::new(fmt::Error));
    let known: eros::Result<u8, PublicErrors> = known.classify(DomainError);
    assert!(known.unwrap_err().is_inner::<fmt::Error>());
}
