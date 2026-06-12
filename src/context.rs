use std::fmt::Display;

use crate::{str_error::StrContext, type_set::TypeSet, SendSyncError, ErrorUnion};

/// Provides `context` methods to add context to `Result`.
pub trait Context {
    type Okay;
    type OutSet: TypeSet;

    /// Adds additional context. This becomes a no-op if the `traced` feature is disabled.
    fn context<C: Into<StrContext>>(
        self,
        context: C,
    ) -> Result<Self::Okay, ErrorUnion<Self::OutSet>>;

    /// Lazily adds additional context. This becomes a no-op if the `traced` feature is disabled.
    fn with_context<F, C: Into<StrContext>>(
        self,
        f: F,
    ) -> Result<Self::Okay, ErrorUnion<Self::OutSet>>
    where
        F: FnOnce() -> C;
}

impl<T, InSet: TypeSet> Context for Result<T, ErrorUnion<InSet>> {
    type Okay = T;
    type OutSet = InSet;

    #[allow(unused_variables)]
    fn context<C: Into<StrContext>>(self, context: C) -> Result<T, ErrorUnion<Self::OutSet>> {
        #[cfg(feature = "context")]
        return self.map_err(|e| e.context(context));
        #[cfg(not(feature = "context"))]
        return self;
    }

    #[allow(unused_variables)]
    fn with_context<F, C: Into<StrContext>>(self, f: F) -> Result<T, ErrorUnion<Self::OutSet>>
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "context")]
        return self.map_err(|e| e.with_context(f));
        #[cfg(not(feature = "context"))]
        return self;
    }
}

impl<T, E: SendSyncError> Context for Result<T, E> {
    type Okay = T;
    type OutSet = (E,);

    #[cfg_attr(feature = "location", track_caller)]
    #[allow(unused_variables)]
    fn context<C: Into<StrContext>>(self, context: C) -> Result<T, ErrorUnion<Self::OutSet>> {
        #[cfg(feature = "context")]
        return self.map_err(|e| {
            let widened: ErrorUnion<Self::OutSet> = ErrorUnion::new(e);
            widened.context(context)
        });
        #[cfg(not(feature = "context"))]
        return self.map_err(|e| ErrorUnion::new(e));
    }

    #[cfg_attr(feature = "location", track_caller)]
    #[allow(unused_variables)]
    fn with_context<F, C: Into<StrContext>>(self, f: F) -> Result<T, ErrorUnion<Self::OutSet>>
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "context")]
        return self.map_err(|e| {
            let widened: ErrorUnion<Self::OutSet> = ErrorUnion::new(e);
            widened.with_context(f)
        });
        #[cfg(not(feature = "context"))]
        return self.map_err(|e| ErrorUnion::new(e));
    }
}

// todo implement when never type is stabilized
// impl<T, E: SendSyncError> Context for E {
//     type Okay = !;

//     type OutSet = (E,)

//     fn context<OutSet, Index, C: Into<StrContext>>(
//         self,
//         context: C,
//     ) -> Result<Self::Okay, ErrorUnion<OutSet>>
//     where
//         OutSet: TypeSet,
//         OutSet::Variants: SupersetOf<<Self::InSet as TypeSet>::Variants, Index>,
//         ErrorUnion<Self::InSet>: Into<ErrorUnion<OutSet>> {
//         todo!()
//     }

//     fn with_context<OutSet: TypeSet, Index, F, C: Into<StrContext>>(
//         self,
//         f: F,
//     ) -> Result<Self::Okay, ErrorUnion<OutSet>>
//     where
//         OutSet::Variants: SupersetOf<<Self::InSet as TypeSet>::Variants, Index>,
//         ErrorUnion<Self::InSet>: Into<ErrorUnion<OutSet>>,
//         F: FnOnce() -> C {
//         todo!()
//     }
// }

//************************************************************************//

impl<T> Context for Option<T> {
    type Okay = T;
    type OutSet = (AbsentValueError,);

    /// This is used for unwrapping an `Option` that is `None`, but expected to be `Some`
    /// and it is desired to propagate this information rather than immediately
    /// panic with `.expect(..)` - presumably to capture additional context up the call stack.
    /// The inner error type is the non-descriptive [`AbsentValueError`], which is type erased,
    /// since the type should not be used to identify the type of error.
    /// Constructing this type is always paired with information ([`context`])
    /// to further explain why the value should exist or provided additional context
    /// around the operation.
    #[cfg_attr(feature = "location", track_caller)]
    #[allow(unused_variables)]
    fn context<C: Into<StrContext>>(self, context: C) -> Result<T, ErrorUnion<Self::OutSet>>
where {
        #[cfg(feature = "context")]
        return self.ok_or_else(|| {
            let widened: ErrorUnion<Self::OutSet> = ErrorUnion::new(AbsentValueError);
            widened.context(context)
        });
        #[cfg(not(feature = "context"))]
        return self.ok_or_else(|| ErrorUnion::new(AbsentValueError));
    }

    /// This is used for unwrapping an `Option` that is `None`, but expected to be `Some`
    /// and it is desired to propagate this information rather than immediately
    /// panic with `.expect(..)` - presumably to capture additional context up the call stack.
    /// The inner error type is the non-descriptive [`AbsentValueError`], which is type erased,
    /// since the type should not be used to identify the type of error.
    /// Constructing this type is always paired with information ([`context`])
    /// to further explain why the value should exist or provided additional context
    /// around the operation.
    #[cfg_attr(feature = "location", track_caller)]
    #[allow(unused_variables)]
    fn with_context<F, C: Into<StrContext>>(self, f: F) -> Result<T, ErrorUnion<Self::OutSet>>
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "context")]
        return self.ok_or_else(|| {
            let widened: ErrorUnion<Self::OutSet> = ErrorUnion::new(AbsentValueError);
            widened.with_context(f)
        });
        #[cfg(not(feature = "context"))]
        return self.ok_or_else(|| ErrorUnion::new(AbsentValueError));
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
pub struct AbsentValueError;

impl Display for AbsentValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "An `Option` was unexpectedly `None`")
    }
}

impl std::error::Error for AbsentValueError {}
