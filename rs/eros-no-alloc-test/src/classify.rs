//! Public classification APIs compiled with a core-only Eros dependency.
//! Downstream tests enable Eros features without changing these functions.
use core::{error::Error, fmt};
use eros::{ErrorUnion, InnerError, ReshapeUnion};

#[derive(Debug, PartialEq, Eq)]
pub struct Code(pub usize);
impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "code {}", self.0)
    }
}
impl Error for Code {}

#[derive(Debug, PartialEq, Eq)]
pub struct DomainError(pub usize);
impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "domain {}", self.0)
    }
}
impl Error for DomainError {}

#[derive(Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct WrappedCode(pub Code);
impl fmt::Display for WrappedCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "wrapped {}", self.0)
    }
}
impl Error for WrappedCode {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

pub type Errors = (DomainError,);

pub fn code_error(code: usize) -> ErrorUnion {
    ErrorUnion::new(Code(code))
}

// The owned argument is identical before and after downstream feature unification.
pub fn category(mut inner: InnerError) -> DomainError {
    if let Some(code) = inner.downcast_mut::<Code>() {
        code.0 = code.0.wrapping_add(1);
        // Ownership extraction has the same result type with and without alloc.
        let code = inner.downcast::<Code>().unwrap();
        DomainError(code.0)
    } else {
        // A failed extraction must return the live, unmodified owner.
        let inner = inner.downcast::<Code>().unwrap_err();
        assert!(!inner.is::<Code>());
        DomainError(0)
    }
}

pub fn classify_error(error: ErrorUnion) -> ErrorUnion<Errors> {
    error.classify(category)
}

pub fn classify_result(result: eros::Result<u8>) -> eros::Result<u8, Errors> {
    result.classify(category)
}

pub fn wrap_concrete(error: ErrorUnion) -> ErrorUnion<(WrappedCode,)> {
    error.map_inner(|inner| WrappedCode(inner.downcast::<Code>().unwrap()))
}

pub fn run_checks() {
    let classified = classify_error(code_error(41));
    assert_eq!(
        classified.narrow::<DomainError, _>().unwrap(),
        DomainError(42)
    );
    let classified = classify_result(Err(code_error(usize::MAX))).unwrap_err();
    assert_eq!(
        classified.narrow::<DomainError, _>().unwrap(),
        DomainError(0)
    );
    assert_eq!(classify_result(Ok(7)).unwrap(), 7);
    let represented = classify_error(ErrorUnion::new(fmt::Error));
    assert_eq!(represented.into_single(), DomainError(0));
    let wrapped = wrap_concrete(code_error(37)).into_single();
    assert_eq!(wrapped, WrappedCode(Code(37)));
    assert!(wrapped.source().unwrap().is::<Code>());
    assert_eq!(size_of::<WrappedCode>(), size_of::<usize>());
}
