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

type PublicErrors = (DomainError,);

// Also runs in the freestanding binary, without a global allocator.
pub fn run_checks() {
    classify_word_payloads();
    map_inner_checks();
    classify_checks();
    into_inner_checks();
    let known: ErrorUnion = ErrorUnion::new(fmt::Error);
    #[cfg(feature = "error_location")]
    let location = known.location();
    let kept: ErrorUnion<PublicErrors> = known.map_inner(|inner| {
        assert!(inner.is::<fmt::Error>());
        DomainError
    });
    assert!(kept.is_inner::<DomainError>());
    #[cfg(feature = "error_location")]
    assert!(core::ptr::eq(location, kept.location()));

    let before = DROPS.load(Ordering::SeqCst);
    let unknown: ErrorUnion = ErrorUnion::new(ImplementationError);
    #[cfg(feature = "error_location")]
    let location = unknown.location();
    let mut calls = 0;
    let classified: ErrorUnion<PublicErrors> = unknown.map_inner(|original| {
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
        success.map_inner(|_| -> DomainError { panic!("successes must bypass classification") });
    assert_eq!(success.unwrap(), 7);

    let failure: eros::Result<u8> = Err(ErrorUnion::new(ImplementationError));
    let failure: eros::Result<u8, PublicErrors> = failure.map_inner(|_| DomainError);
    assert!(failure.unwrap_err().is_inner::<DomainError>());
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 2);

    let failure: eros::Result<u8> = Err(ErrorUnion::new(fmt::Error));
    let failure: eros::Result<u8, PublicErrors> = failure.map_inner(|inner| {
        assert!(inner.is::<fmt::Error>());
        DomainError
    });
    assert!(failure.unwrap_err().is_inner::<DomainError>());
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
        let classified: ErrorUnion<(Status,)> = error.map_inner(classify_code);
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
    // Already represented enum values also reach the mapping callback.
    for status in [Status::Bus, Status::Timeout] {
        let error: ErrorUnion<(Status, fmt::Error)> = ErrorUnion::new(status);
        let kept: ErrorUnion<(Status,)> =
            error.map_inner(|inner| inner.downcast::<Status>().unwrap());
        assert!(kept.is_inner::<Status>());
    }
}

fn map_inner_checks() {
    // A named owner callback compiles with and without downstream feature unification.
    fn replace(inner: InnerError) -> Code {
        Code {
            error_code: inner.downcast::<Code>().unwrap().error_code + 1,
        }
    }
    let error: ErrorUnion<(Code,)> = ErrorUnion::new(Code { error_code: 41 });
    #[cfg(feature = "error_location")]
    let location = error.location();
    let error = error.map_inner(replace);
    #[cfg(feature = "error_location")]
    assert!(core::ptr::eq(location, error.location()));
    assert_eq!(error.into_single().error_code, 42);

    let failure: eros::Result<u8, (Code,)> = Err(ErrorUnion::new(Code { error_code: 41 }));
    let failure = failure.map_inner(replace);
    assert_eq!(failure.unwrap_err().into_single().error_code, 42);
    let success: eros::Result<u8, (Code,)> = Ok(7);
    let success = success.map_inner(|_: InnerError| -> Code {
        panic!("map_inner must not run for successful values")
    });
    assert_eq!(success.unwrap(), 7);

    let error: ErrorUnion = ErrorUnion::new(Code { error_code: 41 });
    let mapped: ErrorUnion<(Status,)> = error.map_inner(classify_code);
    assert_eq!(mapped.into_single(), Status::Timeout);
    let failure: eros::Result<u8> = Err(ErrorUnion::new(Code { error_code: 0 }));
    assert_eq!(
        failure.map_inner(classify_code).unwrap_err().into_single(),
        Status::Bus
    );

    let error: ErrorUnion<(Code,)> = ErrorUnion::new(Code { error_code: 7 });
    let mapped: ErrorUnion<(Code,)> = error.map_inner(replace);
    assert_eq!(mapped.into_single().error_code, 8);
}

fn into_inner_checks() {
    let error: ErrorUnion = ErrorUnion::new(Code { error_code: 41 });
    let mut inner: InnerError = error.into_inner();
    assert!(inner.is::<Code>());
    inner.downcast_mut::<Code>().unwrap().error_code += 1;
    assert_eq!(inner.downcast_ref::<Code>().unwrap().error_code, 42);
    let inner = inner.downcast::<fmt::Error>().unwrap_err();
    assert_eq!(inner.downcast::<Code>().unwrap().error_code, 42);

    let before = DROPS.load(Ordering::SeqCst);
    let error: ErrorUnion = ErrorUnion::new(ImplementationError);
    let inner = error.into_inner();
    assert_eq!(DROPS.load(Ordering::SeqCst), before);
    drop(inner);
    assert_eq!(DROPS.load(Ordering::SeqCst), before + 1);
}

fn classify_checks() {
    type Destination = (fmt::Error, Code);
    for is_code in [false, true] {
        let input: ErrorUnion<(Code, fmt::Error)> = if is_code {
            ErrorUnion::new(Code { error_code: 41 })
        } else {
            ErrorUnion::new(fmt::Error)
        };
        #[cfg(feature = "error_location")]
        let location = input.location();
        let mut calls = 0;
        let output: ErrorUnion<Destination> = input.classify(|inner| {
            calls += 1;
            Code {
                error_code: inner
                    .downcast::<Code>()
                    .map_or(0, |code| code.error_code + 1),
            }
        });
        // Both input types already belong to the destination. Neither may bypass f.
        assert_eq!(calls, 1);
        #[cfg(feature = "error_location")]
        assert!(core::ptr::eq(location, output.location()));
        assert_eq!(
            output.narrow::<Code, _>().unwrap().error_code,
            if is_code { 42 } else { 0 }
        );
    }

    let failure: eros::Result<u8> = Err(ErrorUnion::new(Code { error_code: 0 }));
    let output: eros::Result<u8, (fmt::Error, Status)> = failure.classify(classify_code);
    assert_eq!(
        output.unwrap_err().narrow::<Status, _>().unwrap(),
        Status::Bus
    );

    let success: eros::Result<u8, (Code,)> = Ok(7);
    let output: eros::Result<u8, (Status, fmt::Error)> =
        success.classify(|_| -> Status { panic!("classify must not run for successful values") });
    assert_eq!(output.unwrap(), 7);

    let error: ErrorUnion = ErrorUnion::new(Code { error_code: 41 });
    let output: ErrorUnion = error.classify(classify_code);
    assert!(output.is_inner::<Status>());
}
