use std::{any::Any, fmt::Display};

use crate::{str_error::StrContext, type_set::TypeSet, SendSyncError, TracedUnion};

/// Provides `context` methods to add context to `Result`.
pub trait Context<O> {
    /// Adds additional context. This becomes a no-op if the `traced` feature is disabled.
    fn context<C: Into<StrContext>>(self, context: C) -> O;

    /// Lazily adds additional context. This becomes a no-op if the `traced` feature is disabled.
    fn with_context<F, C: Into<StrContext>>(self, f: F) -> O
    where
        F: FnOnce() -> C;
}

impl<T, E: TypeSet + ?Sized> Context<Result<T, TracedUnion<E>>> for Result<T, TracedUnion<E>> {
    #[allow(unused_variables)]
    fn context<C: Into<StrContext>>(self, context: C) -> Result<T, TracedUnion<E>> {
        #[cfg(feature = "context")]
        return self.map_err(|e| e.context(context));
        #[cfg(not(feature = "context"))]
        return self;
    }

    #[allow(unused_variables)]
    fn with_context<F, C: Into<StrContext>>(self, context: F) -> Result<T, TracedUnion<E>>
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "context")]
        return self.map_err(|e| e.with_context(context));
        #[cfg(not(feature = "context"))]
        return self;
    }
}

impl<T, E: SendSyncError> Context<Result<T, TracedUnion>> for Result<T, E> {
    #[allow(unused_variables)]
    fn context<C: Into<StrContext>>(self, context: C) -> Result<T, TracedUnion> {
        #[cfg(feature = "context")]
        return self
            .map_err(|e| TracedUnion::<(dyn SendSyncError,)>::any_error(e).context(context));
        #[cfg(not(feature = "context"))]
        return self.map_err(TracedUnion::<(dyn SendSyncError,)>::any_error);
    }

    #[allow(unused_variables)]
    fn with_context<F, C: Into<StrContext>>(self, context: F) -> Result<T, TracedUnion>
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "context")]
        return self
            .map_err(|e| TracedUnion::<(dyn SendSyncError,)>::any_error(e).with_context(context));
        #[cfg(not(feature = "context"))]
        return self.map_err(TracedUnion::<(dyn SendSyncError,)>::any_error);
    }
}

impl<E: SendSyncError> Context<TracedUnion> for E {
    #[allow(unused_variables)]
    fn context<C: Into<StrContext>>(self, context: C) -> TracedUnion {
        #[cfg(feature = "context")]
        return TracedUnion::<(dyn SendSyncError,)>::any_error(self).context(context);
        #[cfg(not(feature = "context"))]
        return TracedUnion::<(dyn SendSyncError,)>::any_error(self);
    }

    #[allow(unused_variables)]
    fn with_context<F, C: Into<StrContext>>(self, context: F) -> TracedUnion
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "context")]
        return TracedUnion::<(dyn SendSyncError,)>::any_error(self).with_context(context);
        #[cfg(not(feature = "context"))]
        return TracedUnion::<(dyn SendSyncError,)>::any_error(self);
    }
}

//************************************************************************//

impl<T> Context<Result<T, TracedUnion>> for Option<T> {
    /// This is used for unwrapping an `Option` that is `None`, but expected to be `Some`
    /// and it is desired to propagate this information rather than immediately
    /// panic with `.expect(..)` - presumably to capture additional context up the call stack.
    /// The inner error type is the non-descriptive [`AbsentValueError`], which is type erased,
    /// since the type should not be used to identify the type of error.
    /// Constructing this type is always paired with information ([`context`])
    /// to further explain why the value should exist or provided additional context
    /// around the operation.
    #[allow(unused_variables)]
    fn context<C: Into<StrContext>>(self, context: C) -> Result<T, TracedUnion> {
        #[cfg(feature = "context")]
        return self.ok_or_else(|| {
            TracedUnion::<(dyn SendSyncError,)>::any_error(AbsentValueError(())).context(context)
        });
        #[cfg(not(feature = "context"))]
        return self
            .ok_or_else(|| TracedUnion::<(dyn SendSyncError,)>::any_error(AbsentValueError(())));
    }

    /// This is used for unwrapping an `Option` that is `None`, but expected to be `Some`
    /// and it is desired to propagate this information rather than immediately
    /// panic with `.expect(..)` - presumably to capture additional context up the call stack.
    /// The inner error type is the non-descriptive [`AbsentValueError`], which is type erased,
    /// since the type should not be used to identify the type of error.
    /// Constructing this type is always paired with information ([`context`])
    /// to further explain why the value should exist or provided additional context
    /// around the operation.
    #[allow(unused_variables)]
    fn with_context<F, C: Into<StrContext>>(self, context: F) -> Result<T, TracedUnion>
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "context")]
        return self.ok_or_else(|| {
            TracedUnion::<(dyn SendSyncError,)>::any_error(AbsentValueError(()))
                .with_context(context)
        });
        #[cfg(not(feature = "context"))]
        return self
            .ok_or_else(|| TracedUnion::<(dyn SendSyncError,)>::any_error(AbsentValueError(())));
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
