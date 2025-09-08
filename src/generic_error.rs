#[cfg(feature = "traced")]
use std::backtrace::Backtrace;
use std::{
    borrow::Cow,
    fmt::{self},
};

use crate::{str_error::StrError, ErrorUnion};

/// Any error that satisfies this trait's bounds can be used in a `TracedError`
pub trait AnyError: std::any::Any + std::error::Error + Send + Sync + 'static {}

impl<T> AnyError for T where T: std::error::Error + Send + Sync + 'static {}

impl std::error::Error for Box<dyn AnyError> {}

/// `TracedError` allows adding context to an error throughout the callstack with the `context` or `with_context` methods.
/// This context may be information such as variable values or ongoing operations while the error occurred.
/// If the error is handled higher in the stack, then this can be disregarded (no log pollution).
/// Otherwise you can log it (or panic), capturing all the relevant information in one log.
///
/// A backtrace is captured and added to the log if `RUST_BACKTRACE` is set.
///
/// Use `TracedError` if the underlying error type does not matter.
/// Otherwise, the type can be specified with `TracedError<T>`.
pub struct TracedError<T = Box<dyn AnyError>>
where
    T: AnyError,
{
    inner: T,
    #[cfg(feature = "traced")]
    pub(crate) backtrace: Backtrace,
    #[cfg(feature = "traced")]
    pub(crate) context: Vec<StrError>,
}

impl TracedError {
    /// Create a dynamic type erased `TracedError`
    pub fn boxed<E: AnyError>(source: E) -> Self {
        TracedError::new(Box::new(source))
    }

    /// Attempts to downcast a `Box<dyn AnyError>` into a `TracedError`.
    pub fn from_dyn_error(error: Box<dyn AnyError>) -> Option<Self> {
        // let any = &*error as &dyn std::any::Any;
        // if any.is::<DynTracedErrorWrapper>() {
        //     let wrapper = *(error as Box<dyn std::any::Any>)
        //         .downcast::<DynTracedErrorWrapper>()
        //         .unwrap();
        //     return wrapper.0;
        // }
        // TracedError::new(error)
        (error as Box<dyn std::any::Any>)
            .downcast::<DynTracedErrorWrapper>()
            .ok()
            .map(|e| (*e).0)
    }

    /// Properly erases this `TracedError` into a `Box<dyn AnyError>`.
    pub fn into_any_error(self) -> Box<dyn AnyError> {
        Box::new(DynTracedErrorWrapper(self))
    }
}

impl<T: AnyError> TracedError<T> {
    pub fn new(source: T) -> Self {
        Self {
            inner: source,
            #[cfg(feature = "traced")]
            backtrace: Backtrace::capture(),
            #[cfg(feature = "traced")]
            context: Vec::new(),
        }
    }

    /// Converts these `TracedError` into dynamic type erased `TracedError`
    pub fn traced_dyn(self) -> TracedError {
        debug_assert!(
            std::any::TypeId::of::<T>() != std::any::TypeId::of::<Box<dyn AnyError>>(),
            "traced_dyn() called on already boxed TracedError"
        );

        TracedError {
            inner: Box::new(self.inner),
            #[cfg(feature = "traced")]
            backtrace: self.backtrace,
            #[cfg(feature = "traced")]
            context: self.context,
        }
    }

    /// Converts into the inner type
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Gets a reference to the inner type
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Gets a mutable reference to the inner type
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Maps the inner error to another while preserving context and backtrace
    pub fn map<U, F>(self, f: F) -> TracedError<U>
    where
        U: AnyError,
        F: FnOnce(T) -> U,
    {
        TracedError {
            inner: f(self.inner),
            #[cfg(feature = "traced")]
            backtrace: self.backtrace,
            #[cfg(feature = "traced")]
            context: self.context,
        }
    }

    /// Adds additional context. This becomes a no-op if the `traced` feature is disabled.
    #[allow(unused_mut)]
    #[allow(unused_variables)]
    pub fn context<C: Into<StrError>>(mut self, context: C) -> Self {
        #[cfg(feature = "traced")]
        self.context.push(context.into());
        self
    }

    /// Adds additional context lazily. This becomes a no-op if the `traced` feature is disabled.
    #[allow(unused_mut)]
    #[allow(unused_variables)]
    pub fn with_context<F, C: Into<StrError>>(mut self, f: F) -> TracedError<T>
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "traced")]
        self.context.push(f().into());
        self
    }

    // Note: Even though `std::error::Error` is implemented for Deref.
    // We still redeclare `source` here to tie the lifetime to this,
    // rather than another deref
    /// Returns the lower-level source of this error, if any.
    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

impl<T: AnyError> fmt::Display for TracedError<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.inner)?;
        #[cfg(feature = "traced")]
        {
            if !self.context.is_empty() {
                write!(formatter, "\n\nContext:")?;
                for context_item in self.context.iter() {
                    write!(formatter, "\n\t- {}", context_item)?;
                }
            }
        }
        Ok(())
    }
}

impl<T: AnyError> fmt::Debug for TracedError<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.inner)?;
        #[cfg(feature = "traced")]
        {
            if !self.context.is_empty() {
                write!(formatter, "\n\nContext:")?;
                for context_item in self.context.iter() {
                    write!(formatter, "\n\t- {}", context_item)?;
                }
            }
            write!(formatter, "\n\nBacktrace:\n")?;
            fmt::Display::fmt(&self.backtrace, formatter)?;
        }
        Ok(())
    }
}

// Into `TracedError`
//************************************************************************//

// Note: This is not implemented so something like `TracedError<TracedError<T>>` is not possible.
// Also this allows us to implement `Context` on `E: AnyError` since it does not conflict with itself.
// e.g. In `impl<E: AnyError> Context<TracedError<E>> for E` `TracedError<E>` cannot be `E`.
// Thus `context` can be called directly with no `traced()`/`traced_dyn()` call.
// impl<T: AnyError> std::error::Error for TracedError<T> {
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         self.inner.source()
//     }
// }

impl<T: AnyError> std::error::Error for &TracedError<T> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

impl<T: AnyError> std::error::Error for &mut TracedError<T> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

impl From<String> for TracedError {
    fn from(s: String) -> Self {
        TracedError::new(Box::new(StrError::from(s)))
    }
}

impl From<&'static str> for TracedError {
    fn from(s: &'static str) -> Self {
        TracedError::new(Box::new(StrError::from(s)))
    }
}

impl From<Cow<'static, str>> for TracedError {
    fn from(s: Cow<'static, str>) -> Self {
        TracedError::new(Box::new(StrError::from(s)))
    }
}

impl<T: AnyError> From<T> for TracedError<T> {
    fn from(e: T) -> Self {
        TracedError::new(e)
    }
}

//************************************************************************//

pub trait IntoDynTracedError<O2> {
    /// Convert Error to `TracedError` without caring about the underlying type
    fn traced_dyn(self) -> O2;
}

impl<E> IntoDynTracedError<TracedError> for E
where
    E: AnyError,
{
    #[cfg(feature = "min_specialization")]
    default fn traced_dyn(self) -> TracedError {
        TracedError::new(Box::new(self))
    }

    #[cfg(not(feature = "min_specialization"))]
    fn traced_dyn(self) -> TracedError {
        TracedError::new(Box::new(self))
    }
}

#[cfg(feature = "min_specialization")]
impl IntoDynTracedError<TracedError> for Box<dyn AnyError + '_> {
    fn traced_dyn(self) -> TracedError {
        TracedError::new(self)
    }
}

impl<S, E> IntoDynTracedError<Result<S, TracedError>> for Result<S, E>
where
    E: AnyError,
{
    fn traced_dyn(self) -> Result<S, TracedError> {
        self.map_err(|e| e.traced_dyn())
    }
}

impl<S, E: AnyError> IntoDynTracedError<Result<S, TracedError>> for Result<S, TracedError<E>> {
    #[cfg(feature = "min_specialization")]
    default fn traced_dyn(self) -> Result<S, TracedError> {
        self.map_err(|e| e.traced_dyn())
    }

    #[cfg(not(feature = "min_specialization"))]
    fn traced_dyn(self) -> Result<S, TracedError> {
        self.map_err(|e| e.traced_dyn())
    }
}

#[cfg(feature = "min_specialization")]
impl<S> IntoDynTracedError<Result<S, TracedError<Box<dyn AnyError + '_>>>>
    for Result<S, TracedError<Box<dyn AnyError + '_>>>
{
    fn traced_dyn(self) -> Result<S, TracedError> {
        self
    }
}

//************************************************************************//

pub trait IntoConcreteTracedError<O1> {
    /// Convert Error to `TracedError` keeping the underlying type
    fn traced(self) -> O1;
}

impl<E> IntoConcreteTracedError<TracedError<E>> for E
where
    E: AnyError,
{
    fn traced(self) -> TracedError<E> {
        TracedError::new(self)
    }
}
impl<S, E> IntoConcreteTracedError<Result<S, TracedError<E>>> for Result<S, E>
where
    E: AnyError,
{
    fn traced(self) -> Result<S, TracedError<E>> {
        self.map_err(|e| e.traced())
    }
}

impl<S, E> IntoConcreteTracedError<Result<S, TracedError<E>>>
    for Result<S, ErrorUnion<(TracedError<E>,)>>
where
    E: AnyError,
{
    fn traced(self) -> Result<S, TracedError<E>> {
        self.map_err(|e| e.into_inner())
    }
}

//************************************************************************//

pub trait OptionTracedExt<S> {
    fn ok_or_traced(self) -> Result<S, TracedError>;
}

impl<S> OptionTracedExt<S> for Option<S> {
    fn ok_or_traced(self) -> Result<S, TracedError> {
        self.ok_or_else(|| TracedError::boxed(StrError::Static("`Option` is `None`")))
    }
}

//************************************************************************//

/// Internal struct to convert a `TracedError` into `AnyError`.
///
/// Note: `TracedError` is not `AnyError` since otherwise there would be conflicting trait implementations
#[derive(Debug)]
struct DynTracedErrorWrapper(TracedError);

impl std::error::Error for DynTracedErrorWrapper {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.inner.source()
    }
}

impl fmt::Display for DynTracedErrorWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

//************************************************************************//

#[cfg(feature = "min_specialization")]
#[cfg(feature = "traced")]
#[cfg(test)]
mod test {
    use crate::{Context, ErrorUnion, StrError, TracedError};

    #[test]
    fn adding_context_to_union() {
        let concrete_traced_error: TracedError<std::io::Error> = TracedError::new(
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use"),
        );
        let concrete_union_error: ErrorUnion<(
            TracedError<std::io::Error>,
            i32,
            TracedError<StrError>,
        )> = ErrorUnion::new(concrete_traced_error);
        let result: Result<
            (),
            ErrorUnion<(TracedError<std::io::Error>, i32, TracedError<StrError>)>,
        > = Err(concrete_union_error).context("Context 1");
        let concrete_union_error = result.unwrap_err();
        let concrete_traced_error: TracedError<std::io::Error> =
            match concrete_union_error.to_enum() {
                crate::E3::A(traced_error) => traced_error,
                _ => panic!("Wrong type"),
            };
        assert_eq!(
            concrete_traced_error.context,
            vec![StrError::from("Context 1")]
        );
    }
}
