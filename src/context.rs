use std::{any::Any, fmt::Display};

use crate::{
    generic_error::{AnyError, TracedError},
    str_error::StrError,
};
#[cfg(feature = "min_specialization")]
use crate::{type_set::TypeSet, ErrorUnion};

/// Provides `context` methods to add context to `Result`.
pub trait Context<O> {
    /// Adds additional context. This becomes a no-op if the `traced` feature is disabled.
    fn context<C: Into<StrError>>(self, context: C) -> O;

    /// Lazily adds additional context. This becomes a no-op if the `traced` feature is disabled.
    fn with_context<F, C: Into<StrError>>(self, f: F) -> O
    where
        F: FnOnce() -> C;
}

#[cfg(feature = "min_specialization")]
impl<T, E> Context<Result<T, ErrorUnion<E>>> for Result<T, ErrorUnion<E>>
where
    E: TypeSet,
{
    #[allow(unused_variables)]
    fn context<C: Into<StrError>>(self, context: C) -> Result<T, ErrorUnion<E>> {
        #[cfg(feature = "context")]
        return self.map_err(|mut e| {
            e.value.add_context(context.into());
            e
        });
        #[cfg(not(feature = "context"))]
        return self;
    }

    #[allow(unused_variables)]
    fn with_context<F, C: Into<StrError>>(self, context: F) -> Result<T, ErrorUnion<E>>
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "context")]
        return self.map_err(|mut e| {
            e.value.add_context(context().into());
            e
        });
        #[cfg(not(feature = "context"))]
        return self;
    }
}

impl<T, E: AnyError> Context<Result<T, TracedError<E>>> for Result<T, TracedError<E>> {
    #[allow(unused_variables)]
    fn context<C: Into<StrError>>(self, context: C) -> Result<T, TracedError<E>> {
        #[cfg(feature = "context")]
        return self.map_err(|e| e.context(context));
        #[cfg(not(feature = "context"))]
        return self;
    }

    #[allow(unused_variables)]
    fn with_context<F, C: Into<StrError>>(self, context: F) -> Result<T, TracedError<E>>
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "context")]
        return self.map_err(|e| e.with_context(context));
        #[cfg(not(feature = "context"))]
        return self;
    }
}

impl<T, E: AnyError> Context<Result<T, TracedError>> for Result<T, E> {
    #[allow(unused_variables)]
    fn context<C: Into<StrError>>(self, context: C) -> Result<T, TracedError> {
        #[cfg(feature = "context")]
        return self.map_err(|e| TracedError::boxed(e).context(context));
        #[cfg(not(feature = "context"))]
        return self.map_err(TracedError::boxed);
    }

    #[allow(unused_variables)]
    fn with_context<F, C: Into<StrError>>(self, context: F) -> Result<T, TracedError>
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "context")]
        return self.map_err(|e| TracedError::boxed(e).with_context(context));
        #[cfg(not(feature = "context"))]
        return self.map_err(TracedError::boxed);
    }
}

impl<E: AnyError> Context<TracedError> for E {
    #[allow(unused_variables)]
    fn context<C: Into<StrError>>(self, context: C) -> TracedError {
        #[cfg(feature = "context")]
        return TracedError::boxed(self).context(context);
        #[cfg(not(feature = "context"))]
        return TracedError::boxed(self);
    }

    #[allow(unused_variables)]
    fn with_context<F, C: Into<StrError>>(self, context: F) -> TracedError
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "context")]
        return TracedError::boxed(self).with_context(context);
        #[cfg(not(feature = "context"))]
        return TracedError::boxed(self);
    }
}

//************************************************************************//

impl<T> Context<Result<T, TracedError>> for Option<T> {
    #[allow(unused_variables)]
    fn context<C: Into<StrError>>(self, context: C) -> Result<T, TracedError> {
        #[cfg(feature = "context")]
        return self.ok_or_else(|| TracedError::boxed(AbsentValueError(())).context(context));
        #[cfg(not(feature = "context"))]
        return self.ok_or_else(|| TracedError::boxed(AbsentValueError(())));
    }

    #[allow(unused_variables)]
    fn with_context<F, C: Into<StrError>>(self, context: F) -> Result<T, TracedError>
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "context")]
        return self.ok_or_else(|| TracedError::boxed(AbsentValueError(())).with_context(context));
        #[cfg(not(feature = "context"))]
        return self.ok_or_else(|| TracedError::boxed(AbsentValueError(())));
    }
}

/// An Error type for unwrapping an `Option` that is `None`, but expected to be `Some`.
/// This is used when it is desired to propagate this information rather than immediately
/// panic with `.expect(..)` - presumably to capture additional context up the call stack.
/// This is created by calling `.context(..)` on an `Option<T>`
/// that was `None`. Thus constructing this type is always paired with information
/// to further explain why the value should exist or provided additional context
/// around the operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AbsentValueError(());

impl Display for AbsentValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "An `Option` was unexpectedly `None`")
    }
}

impl std::error::Error for AbsentValueError {}

//************************************************************************//

/// Used internally to allow adding context directly to a `ErrorUnion`
pub trait Contextable: Any {
    #[allow(unused_variables)]
    fn add_context(&mut self, context: StrError);
}

impl<T: 'static> Contextable for T {
    #[cfg(feature = "min_specialization")]
    #[allow(unused_variables)]
    default fn add_context(&mut self, context: StrError) {}
    #[cfg(not(feature = "min_specialization"))]
    #[allow(unused_variables)]
    fn add_context(&mut self, context: StrError) {}
}
#[cfg(feature = "min_specialization")]
impl Contextable for Box<dyn Contextable + '_> {
    #[allow(unused_variables)]
    fn add_context(&mut self, context: StrError) {
        #[cfg(feature = "context")]
        (**self).add_context(context);
    }
}
#[cfg(feature = "min_specialization")]
impl<T: AnyError> Contextable for TracedError<T> {
    #[allow(unused_variables)]
    fn add_context(&mut self, context: StrError) {
        #[cfg(feature = "context")]
        self.context.push(context);
    }
}
