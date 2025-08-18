use std::{backtrace::Backtrace, borrow::Cow, error::Error, fmt};

use crate::{
    str::Str,
    type_set::{SupersetOf, TypeSet},
    Cons, End, ErrorUnion,
};

/// A generic error for when one wishes to propagate information about an issue, but the caller would not care about
/// type of issue. And context can be added at different levels in the call stack.
pub struct TracedError {
    source: AnyError,
    backtrace: Backtrace,
    pub(crate) context: Vec<Str>,
}

impl TracedError {
    pub fn new(source: AnyError) -> Self {
        Self {
            source,
            backtrace: Backtrace::capture(),
            context: Vec::new(),
        }
    }

    pub fn msg(message: impl Into<Str>) -> Self {
        Self::new(AnyError::msg(message))
    }

    pub fn source<T: std::error::Error + Send + Sync + 'static>(source: T) -> Self {
        Self::new(AnyError::source(source))
    }

    pub fn any(any: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self {
        Self::new(AnyError::any(any))
    }

    /// Adds additional context.
    pub fn context<C: Into<Str>>(mut self, context: C) -> Self {
        self.context.push(context.into());
        self
    }

    pub fn inflate<Other, Index>(self) -> ErrorUnion<Other>
    where
        Other: TypeSet,
        Other::Variants: SupersetOf<Cons<AnyError, End>, Index>,
    {
        let error: ErrorUnion<(AnyError,)> = self.into();
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

impl From<AnyError> for TracedError {
    fn from(e: AnyError) -> Self {
        Self::new(e)
    }
}

impl From<TracedError> for ErrorUnion<(AnyError,)> {
    fn from(value: TracedError) -> Self {
        let error: ErrorUnion<(AnyError,)> =
            ErrorUnion::new_internal(value.source, value.context, value.backtrace);
        error
    }
}

impl From<String> for TracedError {
    fn from(s: String) -> Self {
        TracedError::msg(s)
    }
}

impl From<&'static str> for TracedError {
    fn from(s: &'static str) -> Self {
        TracedError::msg(s)
    }
}

impl From<Cow<'static, str>> for TracedError {
    fn from(s: Cow<'static, str>) -> Self {
        TracedError::msg(s)
    }
}

//************************************************************************//

/// A generic error for when one wishes to propagate information about an issue, but the caller would not care about
/// the type of issue. Thus the type is erased. This more efficiently handles `String` like types with `Str` so no double heap allocation is needed.
#[derive(Debug)]
pub enum AnyError {
    /// An error that is just a message
    Msg(Str),
    /// An error that comes from another error which type is erased
    Source(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl AnyError {
    pub fn msg(message: impl Into<Str>) -> Self {
        AnyError::Msg(message.into())
    }

    pub fn source<T: std::error::Error + Send + Sync + 'static>(source: T) -> Self {
        AnyError::Source(Box::new(source))
    }

    pub fn any(any: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self {
        AnyError::Source(any)
    }

    pub fn inflate<Other, Index>(self) -> ErrorUnion<Other>
    where
        Other: TypeSet,
        Other::Variants: SupersetOf<Cons<AnyError, End>, Index>,
    {
        let error: ErrorUnion<(AnyError,)> = self.into();
        error.inflate()
    }
}

impl fmt::Display for AnyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnyError::Msg(msg) => write!(f, "{}", msg),
            AnyError::Source(source) => write!(f, "{}", source),
        }
    }
}

impl std::error::Error for AnyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AnyError::Msg(_) => None,
            AnyError::Source(source) => Some(&**source),
        }
    }
}

impl From<String> for AnyError {
    fn from(s: String) -> Self {
        AnyError::msg(s)
    }
}

impl From<&'static str> for AnyError {
    fn from(s: &'static str) -> Self {
        AnyError::msg(s)
    }
}

impl From<Cow<'static, str>> for AnyError {
    fn from(s: Cow<'static, str>) -> Self {
        AnyError::msg(s)
    }
}

// This does not work
// impl<T> From<T> for AnyError where T: std::error::Error + Send + Sync + 'static {
//     fn from(e: Box<T>) -> Self {
//         AnyError::Source(e)
//     }
// }

// Note: This trait exists since the above does not work
pub trait IntoAnyError<O> {
    fn any(self) -> O;
}

impl<S, E> IntoAnyError<Result<S, AnyError>> for Result<S, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn any(self) -> Result<S, AnyError> {
        self.map_err(|e| AnyError::source(e))
    }
}

impl<E> IntoAnyError<AnyError> for E
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn any(self) -> AnyError {
        AnyError::source(self)
    }
}

//************************************************************************//

pub trait IntoGenericResult<S> {
    fn any(self) -> Result<S, AnyError>;

    fn traced(self) -> Result<S, TracedError>;

    fn inflate<Other, Index>(self) -> Result<S, ErrorUnion<Other>>
    where
        Other: TypeSet,
        Other::Variants: SupersetOf<Cons<AnyError, End>, Index>;
}

impl<S, E> IntoGenericResult<S> for Result<S, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn any(self) -> Result<S, AnyError> {
        self.map_err(AnyError::source)
    }

    fn traced(self) -> Result<S, TracedError> {
        self.map_err(TracedError::source)
    }

    fn inflate<Other, Index>(self) -> Result<S, ErrorUnion<Other>>
    where
        Other: TypeSet,
        Other::Variants: SupersetOf<Cons<AnyError, End>, Index>,
    {
        self.map_err(|e| TracedError::source(e))
            .map_err(TracedError::inflate)
    }
}

// pub trait IntoGenericError {
//     fn any(self) -> AnyError;

//     fn traced(self) -> TracedError;

//     fn inflate<Other, Index>(self) -> ErrorUnion<Other>
//     where
//         Other: TypeSet,
//         Other::Variants: SupersetOf<Cons<AnyError, End>, Index>;
// }

// impl<E> IntoGenericError for E where 
//     E: std::error::Error + Send + Sync + 'static,
// {
//     fn any(self) -> AnyError {
//         AnyError::source(self)
//     }

//     fn traced(self) -> TracedError {
//         TracedError::source(self)
//     }

//     fn inflate<Other, Index>(self) -> ErrorUnion<Other>
//     where
//         Other: TypeSet,
//         Other::Variants: SupersetOf<Cons<AnyError, End>, Index>,
//     {
//         ErrorUnion::new(self)
//     }
// }
