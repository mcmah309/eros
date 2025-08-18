use std::{
    backtrace::Backtrace,
    borrow::Cow,
    fmt::{self},
};

use crate::{
    str_error::StrError,
    type_set::{SupersetOf, TypeSet},
    Cons, End, ErrorUnion,
};

pub trait BoxedError: std::error::Error + Send + Sync + 'static {}

impl<T> BoxedError for T where T: std::error::Error + Send + Sync + 'static {}

impl std::error::Error for Box<dyn BoxedError> {}

/// A generic error for propagating information about the error context. The caller may or may not care about
/// type the underlying error type depending on if `T` is provided.
///
/// Context is intended to be added at different levels in the call stack with `context` or `with_context` methods.
pub struct TracedError<T = Box<dyn BoxedError>>
where
    T: BoxedError,
{
    source: T,
    pub(crate) backtrace: Backtrace,
    pub(crate) context: Vec<StrError>,
}

impl<T: BoxedError> TracedError<T> {
    pub fn new(source: T) -> Self {
        Self {
            source: source,
            backtrace: Backtrace::capture(),
            context: Vec::new(),
        }
    }

    pub fn into_source(self) -> T {
        self.source
    }

    pub fn source(&self) -> &T {
        &self.source
    }

    pub fn source_mut(&mut self) -> &mut T {
        &mut self.source
    }

    /// Adds additional context.
    pub fn context<C: Into<StrError>>(mut self, context: C) -> Self {
        self.context.push(context.into());
        self
    }

    pub fn inflate<Other, Index>(self) -> ErrorUnion<Other>
    where
        Other: TypeSet,
        Other::Variants: SupersetOf<Cons<TracedError<T>, End>, Index>,
    {
        let error: ErrorUnion<(TracedError<T>,)> = self.into();
        error.inflate()
    }
}

impl<T: BoxedError> fmt::Display for TracedError<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.source)?;
        if !self.context.is_empty() {
            write!(formatter, "\n\nContext:")?;
            for context_item in self.context.iter() {
                write!(formatter, "\n\t- {}", context_item)?;
            }
        }
        Ok(())
    }
}

impl<T: BoxedError> fmt::Debug for TracedError<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.source)?;
        if !self.context.is_empty() {
            write!(formatter, "\n\nContext:")?;
            for context_item in self.context.iter() {
                write!(formatter, "\n\t- {}", context_item)?;
            }
        }
        write!(formatter, "\n\nBacktrace:\n")?;
        fmt::Display::fmt(&self.backtrace, formatter)?;
        Ok(())
    }
}

impl<T: BoxedError> std::error::Error for TracedError<T> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.source()
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

//************************************************************************//

pub trait IntoTracedError<O1, O2> {
    /// Convert Error to `TraceError` keeping the underlying type
    fn traced(self) -> O1;
    /// Convert Error to `TraceError` without caring about the underlying type
    fn traced_dyn(self) -> O2;
}

impl<E> IntoTracedError<TracedError<E>, TracedError> for E
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn traced(self) -> TracedError<E> {
        TracedError::new(self)
    }

    fn traced_dyn(self) -> TracedError {
        TracedError::new(Box::new(self))
    }
}

impl<S, E> IntoTracedError<Result<S, TracedError<E>>, Result<S, TracedError>> for Result<S, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn traced(self) -> Result<S, TracedError<E>> {
        self.map_err(|e| e.traced())
    }

    fn traced_dyn(self) -> Result<S, TracedError> {
        self.map_err(|e| e.traced_dyn())
    }
}

//************************************************************************//

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
        )> = concrete_traced_error.inflate();
        let result: Result<
            (),
            ErrorUnion<(TracedError<std::io::Error>, i32, TracedError<StrError>)>,
        > = Err(concrete_union_error).context("Context 1");
        let concrete_union_error = result.unwrap_err();
        let concrete_traced_error = concrete_union_error
            .downcast::<TracedError<std::io::Error>>()
            .unwrap();
        assert_eq!(concrete_traced_error.context, vec![StrError::from("Context 1")]);
    }
}
