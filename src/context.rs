use std::any::Any;

use crate::{
    generic_error::{BoxedError, TracedError},
    str_error::StrError,
    type_set::TypeSet,
    ErrorUnion,
};

/// Provides `context` methods to add context to `Result`.
pub trait Context<O> {
    /// Adds additional context.
    fn context<C: Into<StrError>>(self, context: C) -> O;

    /// Lazily adds additional context.
    fn with_context<F, C: Into<StrError>>(self, f: F) -> O
    where
        F: FnOnce() -> C;
}

impl<T, E> Context<Result<T, ErrorUnion<E>>> for Result<T, ErrorUnion<E>>
where
    E: TypeSet,
{
    fn context<C: Into<StrError>>(self, context: C) -> Result<T, ErrorUnion<E>> {
        self.map_err(|mut e| {
            e.value.add_context(context.into());
            e
        })
    }

    fn with_context<F, C: Into<StrError>>(self, context: F) -> Result<T, ErrorUnion<E>>
    where
        F: FnOnce() -> C,
    {
        self.map_err(|mut e| {
            e.value.add_context(context().into());
            e
        })
    }
}

impl<T, E: BoxedError> Context<Result<T, TracedError<E>>> for Result<T, TracedError<E>> {
    fn context<C: Into<StrError>>(self, context: C) -> Result<T, TracedError<E>> {
        self.map_err(|e| e.context(context))
    }

    fn with_context<F, C: Into<StrError>>(self, context: F) -> Result<T, TracedError<E>>
    where
        F: FnOnce() -> C,
    {
        self.map_err(|e| e.with_context(context))
    }
}

impl<T: BoxedError> Context<TracedError<T>> for TracedError<T> {
    fn context<C: Into<StrError>>(mut self, context: C) -> TracedError<T> {
        self.context.push(context.into());
        self
    }

    fn with_context<F, C: Into<StrError>>(mut self, f: F) -> TracedError<T>
    where
        F: FnOnce() -> C,
    {
        self.context.push(f().into());
        self
    }
}

pub trait Contextable: Any {
    fn add_context(&mut self, context: StrError);
}

impl<T: 'static> Contextable for T {
    default fn add_context(&mut self, _context: StrError) {}
}

impl Contextable for Box<dyn Contextable + '_> {
    fn add_context(&mut self, context: StrError) {
        (**self).add_context(context);
    }
}

impl<T: BoxedError> Contextable for TracedError<T> {
    fn add_context(&mut self, context: StrError) {
        self.context.push(context);
    }
}