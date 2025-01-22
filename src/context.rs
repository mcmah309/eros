use crate::{string_kind::StringKind, type_set::TypeSet, U};

/// Provides `context` methods to add context to `Result`.
pub trait Context<T, E>
where
    E: TypeSet,
{
    /// Adds additional context.
    #[allow(private_bounds)]
    fn context<C: Into<StringKind>>(self, context: C) -> Result<T, U<E>>;

    /// Lazily adds additional context.
    #[allow(private_bounds)]
    fn with_context<F, C: Into<StringKind>>(self, f: F) -> Result<T, U<E>>
    where
        F: FnOnce() -> C;
}

impl<T, E> Context<T, E> for Result<T, U<E>>
where
    E: TypeSet,
{
    #[allow(private_bounds)]
    fn context<C: Into<StringKind>>(self, context: C) -> Result<T, U<E>> {
        self.map_err(|mut e| {
            e.context.push(context.into());
            e
        })
    }

    #[allow(private_bounds)]
    fn with_context<F, C: Into<StringKind>>(self, context: F) -> Result<T, U<E>>
    where
        F: FnOnce() -> C,
    {
        self.map_err(|mut e| {
            e.context.push(context().into());
            e
        })
    }
}
