use std::{backtrace::Backtrace, borrow::Cow, fmt};

use crate::{
    string_kind::StringKind,
    type_set::{SupersetOf, TypeSet},
    Cons, End, ErrorUnion,
};

/// A generic error for when one wishes to propagate information about an issue, but the caller would not care about
/// type of issue. And context can be added at different levels in the call stack.
pub struct GenericCtxError {
    source: GenericError,
    backtrace: Backtrace,
    pub(crate) context: Vec<StringKind>,
}

impl GenericCtxError {
    pub fn new(source: GenericError) -> Self {
        Self {
            source,
            backtrace: Backtrace::capture(),
            context: Vec::new(),
        }
    }

    pub fn msg(message: impl Into<StringKind>) -> Self {
        Self::new(GenericError::msg(message))
    }

    pub fn source<T: std::error::Error + Send + Sync + 'static>(source: T) -> Self {
        Self::new(GenericError::source(source))
    }

    pub fn any(any: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self {
        Self::new(GenericError::any(any))
    }

    /// Adds additional context.
    pub fn context<C: Into<StringKind>>(mut self, context: C) -> Self {
        self.context.push(context.into());
        self
    }

    pub fn inflate<Other, Index>(self) -> ErrorUnion<Other>
    where
        Other: TypeSet,
        Other::Variants: SupersetOf<Cons<GenericError, End>, Index>,
    {
        let error: ErrorUnion<(GenericError,)> = self.into();
        error.inflate()
    }
}

impl fmt::Display for GenericCtxError {
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

impl fmt::Debug for GenericCtxError {
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

impl std::error::Error for GenericCtxError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.source()
    }
}

impl From<GenericError> for GenericCtxError {
    fn from(e: GenericError) -> Self {
        Self::new(e)
    }
}

impl From<GenericCtxError> for ErrorUnion<(GenericError,)> {
    fn from(value: GenericCtxError) -> Self {
        let error: ErrorUnion<(GenericError,)> =
            ErrorUnion::new_internal(value.source, value.context, value.backtrace);
        error
    }
}

/// A generic error for when one wishes to propagate information about an issue, but the caller would not care about
/// the type of issue.
#[derive(Debug)]
pub enum GenericError {
    Msg(StringKind),
    Source(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl GenericError {
    pub fn msg(message: impl Into<StringKind>) -> Self {
        GenericError::Msg(message.into())
    }

    pub fn source<T: std::error::Error + Send + Sync + 'static>(source: T) -> Self {
        GenericError::Source(Box::new(source))
    }

    pub fn any(any: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self {
        GenericError::Source(any)
    }

    pub fn inflate<Other, Index>(self) -> ErrorUnion<Other>
    where
        Other: TypeSet,
        Other::Variants: SupersetOf<Cons<GenericError, End>, Index>,
    {
        let error: ErrorUnion<(GenericError,)> = self.into();
        error.inflate()
    }
}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GenericError::Msg(msg) => write!(f, "{}", msg),
            GenericError::Source(source) => write!(f, "{}", source),
        }
    }
}

impl std::error::Error for GenericError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GenericError::Msg(_) => None,
            GenericError::Source(source) => Some(&**source),
        }
    }
}

impl From<String> for GenericError {
    fn from(s: String) -> Self {
        GenericError::Msg(StringKind::Owned(s))
    }
}

impl From<&'static str> for GenericError {
    fn from(s: &'static str) -> Self {
        GenericError::Msg(StringKind::Static(s))
    }
}

impl From<Cow<'static, str>> for GenericError {
    fn from(s: Cow<'static, str>) -> Self {
        GenericError::Msg(s.into())
    }
}


// Note: This trait is used since
// This does not work
// impl<T> From<T> for GenericError where T: std::error::Error + Send + Sync + 'static {
//     fn from(e: Box<T>) -> Self {
//         GenericError::Source(Box::new(e))
//     }
// }
pub trait Generalize<S> {
    fn generalize(self) -> Result<S,GenericCtxError>;
}

impl<S,E> Generalize<S> for Result<S,E> where E: std::error::Error + Send + Sync + 'static {
    fn generalize(self) -> Result<S,GenericCtxError> {
        self.map_err(|e| {
            GenericCtxError::source(e)
        })
    }
}
