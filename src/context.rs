use crate::{generic_error::TracedError, str::Str, type_set::TypeSet, ErrorUnion};

/// Provides `context` methods to add context to `Result`.
pub trait Context<T, E> {
    /// Adds additional context.
    fn context<C: Into<Str>>(self, context: C) -> Result<T, E>;

    /// Lazily adds additional context.
    fn with_context<F, C: Into<Str>>(self, f: F) -> Result<T, E>
    where
        F: FnOnce() -> C;
}

impl<T, E> Context<T, ErrorUnion<E>> for Result<T, ErrorUnion<E>>
where
    E: TypeSet,
{
    fn context<C: Into<Str>>(self, context: C) -> Result<T, ErrorUnion<E>> {
        self.map_err(|mut e| {
            if let Some(traced_error) = e.value.downcast_mut::<TracedError>() {
                traced_error.context.push(context.into());
            }
            e
        })
    }

    fn with_context<F, C: Into<Str>>(self, context: F) -> Result<T, ErrorUnion<E>>
    where
        F: FnOnce() -> C,
    {
        self.map_err(|mut e| {
            if let Some(traced_error) = e.value.downcast_mut::<TracedError>() {
                traced_error.context.push(context().into());
            }
            e
        })
    }
}

impl<T> Context<T, TracedError> for Result<T, TracedError> {
    fn context<C: Into<Str>>(self, context: C) -> Result<T, TracedError> {
        self.map_err(|mut e| {
            e.context.push(context.into());
            e
        })
    }

    fn with_context<F, C: Into<Str>>(self, context: F) -> Result<T, TracedError>
    where
        F: FnOnce() -> C,
    {
        self.map_err(|mut e| {
            e.context.push(context().into());
            e
        })
    }
}
