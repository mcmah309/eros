use std::{backtrace::Backtrace, borrow::Cow, fmt};

use crate::{
    str_error::StrError,
    type_set::{SupersetOf, TypeSet},
    Cons, End, ErrorUnion,
};

/// A generic error for propagating information about the error context. The caller may or may not care about
/// type the underlying error type depending on if `T` is provided.
/// 
/// Context is intended to be added at different levels in the call stack with `context` or `with_context` methods.
pub struct TracedError<T = Box<dyn std::error::Error + Send + Sync>>
where
    T: 'static,
{
    source: T,
    pub(crate) backtrace: Backtrace,
    pub(crate) context: Vec<StrError>,
}

impl<T> TracedError<T> {
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

impl fmt::Display for TracedError {
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

impl fmt::Debug for TracedError {
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

impl std::error::Error for TracedError {
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
