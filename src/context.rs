use crate::{generic_error::GenericCtxError, string_kind::StringKind, type_set::TypeSet, ErrorUnion};

/// Provides `context` methods to add context to `Result`.
pub trait Context<T, E>
{
    /// Adds additional context.
    fn context<C: Into<StringKind>>(self, context: C) -> Result<T, E>;

    /// Lazily adds additional context.
    fn with_context<F, C: Into<StringKind>>(self, f: F) -> Result<T, E>
    where
        F: FnOnce() -> C;
}

impl<T, E> Context<T, ErrorUnion<E>> for Result<T, ErrorUnion<E>>
where
    E: TypeSet,
{
    fn context<C: Into<StringKind>>(self, context: C) -> Result<T, ErrorUnion<E>> {
        self.map_err(|mut e| {
            e.context.push(context.into());
            e
        })
    }

    fn with_context<F, C: Into<StringKind>>(self, context: F) -> Result<T, ErrorUnion<E>>
    where
        F: FnOnce() -> C,
    {
        self.map_err(|mut e| {
            e.context.push(context().into());
            e
        })
    }
}

impl<T> Context<T,GenericCtxError> for Result<T,GenericCtxError> {
    fn context<C: Into<StringKind>>(self, context: C) -> Result<T, GenericCtxError> {
        self.map_err(|mut e| {
            e.context.push(context.into());
            e
        })
    }

    fn with_context<F, C: Into<StringKind>>(self, context: F) -> Result<T, GenericCtxError>
    where
        F: FnOnce() -> C,
    {
        self.map_err(|mut e| {
            e.context.push(context().into());
            e
        })
    }
}

