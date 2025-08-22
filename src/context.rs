use std::any::Any;

use crate::{
    generic_error::{AnyError, TracedError},
    str_error::StrError,
};
#[cfg(feature = "min_specialization")]
use crate::{type_set::TypeSet, ErrorUnion};

/// Provides `context` methods to add context to `Result`.
pub trait Context<O> {
    /// Adds additional context.
    fn context<C: Into<StrError>>(self, context: C) -> O;

    /// Lazily adds additional context.
    fn with_context<F, C: Into<StrError>>(self, f: F) -> O
    where
        F: FnOnce() -> C;
}

#[cfg(feature = "min_specialization")]
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

impl<T, E: AnyError> Context<Result<T, TracedError<E>>> for Result<T, TracedError<E>> {
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

//************************************************************************//

pub trait Contextable: Any {
    fn add_context(&mut self, _context: StrError) {}
}

impl<T: 'static> Contextable for T {
    #[cfg(feature = "min_specialization")]
    default fn add_context(&mut self, _context: StrError) {}
    #[cfg(not(feature = "min_specialization"))]
    fn add_context(&mut self, _context: StrError) {}
}
#[cfg(feature = "min_specialization")]
impl Contextable for Box<dyn Contextable + '_> {
    fn add_context(&mut self, context: StrError) {
        (**self).add_context(context);
    }
}
#[cfg(feature = "min_specialization")]
impl<T: AnyError> Contextable for TracedError<T> {
    fn add_context(&mut self, context: StrError) {
        self.context.push(context);
    }
}
