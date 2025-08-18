use crate::{generic_error::TracedError, str_error::StrError, type_set::TypeSet, ErrorUnion};

/// Provides `context` methods to add context to `Result`.
pub trait Context<T, E> {
    /// Adds additional context.
    fn context<C: Into<StrError>>(self, context: C) -> Result<T, E>;

    /// Lazily adds additional context.
    fn with_context<F, C: Into<StrError>>(self, f: F) -> Result<T, E>
    where
        F: FnOnce() -> C;
}

impl<T, E> Context<T, ErrorUnion<E>> for Result<T, ErrorUnion<E>>
where
    E: TypeSet,
{
    fn context<C: Into<StrError>>(self, context: C) -> Result<T, ErrorUnion<E>> {
        self.map_err(|mut e| {
            // todo Note: This will fail to add context if `TracedError` is anything but the default type. Fix when specialization is stabilized.
            if let Some(traced_error) = e.value.downcast_mut::<TracedError>() {
                traced_error.context.push(context.into());
            }
            e
        })
    }

    fn with_context<F, C: Into<StrError>>(self, context: F) -> Result<T, ErrorUnion<E>>
    where
        F: FnOnce() -> C,
    {
        self.map_err(|mut e| {
            // todo Note: This will fail to add context if `TracedError` is anything but the default type. Fix when specialization is stabilized.
            if let Some(traced_error) = e.value.downcast_mut::<TracedError>() {
                traced_error.context.push(context().into());
            }
            e
        })
    }
}

impl<T, E> Context<T, TracedError<E>> for Result<T, TracedError<E>> {
    fn context<C: Into<StrError>>(self, context: C) -> Result<T, TracedError<E>> {
        self.map_err(|mut e| {
            e.context.push(context.into());
            e
        })
    }

    fn with_context<F, C: Into<StrError>>(self, context: F) -> Result<T, TracedError<E>>
    where
        F: FnOnce() -> C,
    {
        self.map_err(|mut e| {
            e.context.push(context().into());
            e
        })
    }
}
