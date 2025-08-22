use std::any::Any;

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
        #[cfg(feature = "traced")]
        return self.map_err(|mut e| {
            e.value.add_context(context.into());
            e
        });
        #[cfg(not(feature = "traced"))]
        return self;
    }

    #[allow(unused_variables)]
    fn with_context<F, C: Into<StrError>>(self, context: F) -> Result<T, ErrorUnion<E>>
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "traced")]
        return self.map_err(|mut e| {
            e.value.add_context(context().into());
            e
        });
        return self;
    }
}

impl<T, E: AnyError> Context<Result<T, TracedError<E>>> for Result<T, TracedError<E>> {
    #[allow(unused_variables)]
    fn context<C: Into<StrError>>(self, context: C) -> Result<T, TracedError<E>> {
        #[cfg(feature = "traced")]
        return self.map_err(|e| e.context(context));
        #[cfg(not(feature = "traced"))]
        return self;
    }

    #[allow(unused_variables)]
    fn with_context<F, C: Into<StrError>>(self, context: F) -> Result<T, TracedError<E>>
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "traced")]
        return self.map_err(|e| e.with_context(context));
        #[cfg(not(feature = "traced"))]
        return self;
    }
}

//************************************************************************//

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
        #[cfg(feature = "traced")]
        (**self).add_context(context);
    }
}
#[cfg(feature = "min_specialization")]
impl<T: AnyError> Contextable for TracedError<T> {
    #[allow(unused_variables)]
    fn add_context(&mut self, context: StrError) {
        #[cfg(feature = "traced")]
        self.context.push(context);
    }
}
