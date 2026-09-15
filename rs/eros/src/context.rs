use alloc::{borrow::Cow, boxed::Box, string::String};
use core::fmt::Display;
use core::result::Result;

use crate::{ErrorUnion, SendSyncError, type_set::TypeSet};

#[derive(Debug)]
pub struct ErosContext {
    pub(crate) context: ContextValue,
    #[cfg(feature = "location")]
    pub(crate) location: &'static core::panic::Location<'static>,
    #[cfg(feature = "user_context")]
    pub(crate) is_user_facing: bool,
}

impl ErosContext {
    #[cfg_attr(feature = "location", track_caller)]
    pub fn new(context: ContextValue) -> Self {
        Self {
            context,
            #[cfg(feature = "location")]
            location: core::panic::Location::caller(),
            #[cfg(feature = "user_context")]
            is_user_facing: false,
        }
    }

    #[cfg(feature = "user_context")]
    #[cfg_attr(feature = "location", track_caller)]
    pub fn new_user_facing(context: ContextValue) -> Self {
        Self {
            context,
            #[cfg(feature = "location")]
            location: core::panic::Location::caller(),
            is_user_facing: true,
        }
    }
}

/// A context value containing a static message, an owned message, or an error.
#[derive(Debug)]
pub enum ContextValue {
    Static(&'static str),
    Owned(String),
    Error(Box<dyn SendSyncError>),
}

impl core::fmt::Display for ContextValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ContextValue::Static(s) => write!(f, "{}", s),
            ContextValue::Owned(s) => write!(f, "{}", s),
            ContextValue::Error(e) => write!(f, "{}", e),
        }
    }
}

impl From<&'static str> for ContextValue {
    fn from(s: &'static str) -> ContextValue {
        ContextValue::Static(s)
    }
}

impl From<String> for ContextValue {
    fn from(s: String) -> ContextValue {
        ContextValue::Owned(s)
    }
}

impl From<Cow<'static, str>> for ContextValue {
    fn from(s: Cow<'static, str>) -> ContextValue {
        match s {
            Cow::Borrowed(s) => ContextValue::Static(s),
            Cow::Owned(s) => ContextValue::Owned(s),
        }
    }
}

impl From<Box<dyn SendSyncError>> for ContextValue {
    fn from(e: Box<dyn SendSyncError>) -> Self {
        ContextValue::Error(e)
    }
}

/// Provides `context` methods to add context to `Result`.
pub trait Context {
    /// The success value returned after attaching context.
    type Ok;
    type OutSet: TypeSet;

    /// Adds additional context. This becomes a no-op if the `context` feature is disabled.
    fn context<C: Into<ContextValue>>(
        self,
        context: C,
    ) -> Result<Self::Ok, ErrorUnion<Self::OutSet>>;

    /// Lazily adds additional context. This becomes a no-op if the `context` feature is disabled.
    fn with_context<F, C: Into<ContextValue>>(
        self,
        f: F,
    ) -> Result<Self::Ok, ErrorUnion<Self::OutSet>>
    where
        F: FnOnce() -> C;

    /// Adds additional context that is user facing. This becomes a no-op if the `context` feature is disabled.
    #[cfg(feature = "user_context")]
    fn user_context<C: Into<ContextValue>>(
        self,
        context: C,
    ) -> Result<Self::Ok, ErrorUnion<Self::OutSet>>;

    /// Lazily adds additional user facing context. This becomes a no-op if the `context` feature is disabled.
    #[cfg(feature = "user_context")]
    fn with_user_context<F, C: Into<ContextValue>>(
        self,
        f: F,
    ) -> Result<Self::Ok, ErrorUnion<Self::OutSet>>
    where
        F: FnOnce() -> C;
}

impl<T, InSet: TypeSet> Context for Result<T, ErrorUnion<InSet>> {
    type Ok = T;
    type OutSet = InSet;

    #[allow(unused_variables)]
    #[cfg_attr(feature = "location", track_caller)]
    fn context<C: Into<ContextValue>>(self, context: C) -> Result<T, ErrorUnion<Self::OutSet>> {
        // Note: We use match so the call location gets passed through
        #[cfg(feature = "context")]
        return match self {
            Ok(val) => Ok(val),
            Err(e) => Err(e.context(context)),
        };
        #[cfg(not(feature = "context"))]
        return self;
    }

    #[allow(unused_variables)]
    #[cfg_attr(feature = "location", track_caller)]
    fn with_context<F, C: Into<ContextValue>>(self, f: F) -> Result<T, ErrorUnion<Self::OutSet>>
    where
        F: FnOnce() -> C,
    {
        // Note: We use match so the call location gets passed through
        #[cfg(feature = "context")]
        return match self {
            Ok(val) => Ok(val),
            Err(e) => Err(e.with_context(f)),
        };
        #[cfg(not(feature = "context"))]
        return self;
    }

    #[cfg(feature = "user_context")]
    #[allow(unused_variables)]
    #[cfg_attr(feature = "location", track_caller)]
    fn user_context<C: Into<ContextValue>>(
        self,
        context: C,
    ) -> Result<T, ErrorUnion<Self::OutSet>> {
        // Note: We use match so the call location gets passed through
        #[cfg(feature = "context")]
        return match self {
            Ok(val) => Ok(val),
            Err(e) => Err(e.user_context(context)),
        };
        #[cfg(not(feature = "context"))]
        return self;
    }

    #[cfg(feature = "user_context")]
    #[allow(unused_variables)]
    #[cfg_attr(feature = "location", track_caller)]
    fn with_user_context<F, C: Into<ContextValue>>(
        self,
        f: F,
    ) -> Result<T, ErrorUnion<Self::OutSet>>
    where
        F: FnOnce() -> C,
    {
        // Note: We use match so the call location gets passed through
        #[cfg(feature = "context")]
        return match self {
            Ok(val) => Ok(val),
            Err(e) => Err(e.with_user_context(f)),
        };
        #[cfg(not(feature = "context"))]
        return self;
    }
}

impl<T, E: SendSyncError> Context for Result<T, E> {
    type Ok = T;
    type OutSet = (E,);

    #[allow(unused_variables)]
    #[cfg_attr(feature = "location", track_caller)]
    fn context<C: Into<ContextValue>>(self, context: C) -> Result<T, ErrorUnion<Self::OutSet>> {
        // Note: We use match so the call location gets passed through
        #[cfg(feature = "context")]
        return match self {
            Ok(val) => Ok(val),
            Err(e) => {
                let widened: ErrorUnion<Self::OutSet> = ErrorUnion::new(e);
                Err(widened.context(context))
            }
        };
        // Note: We use match so the call location gets passed through
        #[cfg(not(feature = "context"))]
        return match self {
            Ok(val) => Ok(val),
            Err(e) => Err(ErrorUnion::new(e)),
        };
    }

    #[allow(unused_variables)]
    #[cfg_attr(feature = "location", track_caller)]
    fn with_context<F, C: Into<ContextValue>>(self, f: F) -> Result<T, ErrorUnion<Self::OutSet>>
    where
        F: FnOnce() -> C,
    {
        // Note: We use match so the call location gets passed through
        #[cfg(feature = "context")]
        return match self {
            Ok(val) => Ok(val),
            Err(e) => {
                let widened: ErrorUnion<Self::OutSet> = ErrorUnion::new(e);
                Err(widened.with_context(f))
            }
        };
        // Note: We use match so the call location gets passed through
        #[cfg(not(feature = "context"))]
        return match self {
            Ok(val) => Ok(val),
            Err(e) => Err(ErrorUnion::new(e)),
        };
    }

    #[cfg(feature = "user_context")]
    #[allow(unused_variables)]
    #[cfg_attr(feature = "location", track_caller)]
    fn user_context<C: Into<ContextValue>>(
        self,
        context: C,
    ) -> Result<T, ErrorUnion<Self::OutSet>> {
        // Note: We use match so the call location gets passed through
        #[cfg(feature = "context")]
        return match self {
            Ok(val) => Ok(val),
            Err(e) => {
                let widened: ErrorUnion<Self::OutSet> = ErrorUnion::new(e);
                Err(widened.user_context(context))
            }
        };
        // Note: We use match so the call location gets passed through
        #[cfg(not(feature = "context"))]
        return match self {
            Ok(val) => Ok(val),
            Err(e) => Err(ErrorUnion::new(e)),
        };
    }

    #[cfg(feature = "user_context")]
    #[allow(unused_variables)]
    #[cfg_attr(feature = "location", track_caller)]
    fn with_user_context<F, C: Into<ContextValue>>(
        self,
        f: F,
    ) -> Result<T, ErrorUnion<Self::OutSet>>
    where
        F: FnOnce() -> C,
    {
        // Note: We use match so the call location gets passed through
        #[cfg(feature = "context")]
        return match self {
            Ok(val) => Ok(val),
            Err(e) => {
                let widened: ErrorUnion<Self::OutSet> = ErrorUnion::new(e);
                Err(widened.with_user_context(f))
            }
        };
        // Note: We use match so the call location gets passed through
        #[cfg(not(feature = "context"))]
        return match self {
            Ok(val) => Ok(val),
            Err(e) => Err(ErrorUnion::new(e)),
        };
    }
}

// todo implement when never type is stabilized
// impl<T, E: SendSyncError> Context for E {
//     type Ok = !;

//     type OutSet = (E,)

//     fn context<OutSet, Index, C: Into<StrContext>>(
//         self,
//         context: C,
//     ) -> Result<Self::Ok, ErrorUnion<OutSet>>
//     where
//         OutSet: TypeSet,
//         OutSet::Variants: SupersetOf<<Self::InSet as TypeSet>::Variants, Index>,
//         ErrorUnion<Self::InSet>: Into<ErrorUnion<OutSet>> {
//         todo!()
//     }

//     fn with_context<OutSet: TypeSet, Index, F, C: Into<StrContext>>(
//         self,
//         f: F,
//     ) -> Result<Self::Ok, ErrorUnion<OutSet>>
//     where
//         OutSet::Variants: SupersetOf<<Self::InSet as TypeSet>::Variants, Index>,
//         ErrorUnion<Self::InSet>: Into<ErrorUnion<OutSet>>,
//         F: FnOnce() -> C {
//         todo!()
//     }
// }

//************************************************************************//

impl<T> Context for Option<T> {
    type Ok = T;
    type OutSet = (AbsentValueError,);

    /// This is used for unwrapping an `Option` that is `None`, but expected to be `Some`
    /// and it is desired to propagate this information rather than immediately
    /// panic with `.expect(..)` - presumably to capture additional context up the call stack.
    /// The inner error type is the non-descriptive [`AbsentValueError`], which is type erased,
    /// since the type should not be used to identify the type of error.
    /// Constructing this type is always paired with information ([`Context::context`])
    /// to further explain why the value should exist or provided additional context
    /// around the operation.
    #[allow(unused_variables)]
    #[cfg_attr(feature = "location", track_caller)]
    fn context<C: Into<ContextValue>>(self, context: C) -> Result<T, ErrorUnion<Self::OutSet>> {
        // Note: We use match so the call location gets passed through
        #[cfg(feature = "context")]
        return match self {
            Some(val) => Ok(val),
            None => {
                let widened: ErrorUnion<Self::OutSet> = ErrorUnion::new(AbsentValueError);
                Err(widened.context(context))
            }
        };
        // Note: We use match so the call location gets passed through
        #[cfg(not(feature = "context"))]
        return match self {
            Some(val) => Ok(val),
            None => Err(ErrorUnion::new(AbsentValueError)),
        };
    }

    /// This is used for unwrapping an `Option` that is `None`, but expected to be `Some`
    /// and it is desired to propagate this information rather than immediately
    /// panic with `.expect(..)` - presumably to capture additional context up the call stack.
    /// The inner error type is the non-descriptive [`AbsentValueError`], which is type erased,
    /// since the type should not be used to identify the type of error.
    /// Constructing this type is always paired with information ([`Context::context`])
    /// to further explain why the value should exist or provided additional context
    /// around the operation.
    #[allow(unused_variables)]
    #[cfg_attr(feature = "location", track_caller)]
    fn with_context<F, C: Into<ContextValue>>(self, f: F) -> Result<T, ErrorUnion<Self::OutSet>>
    where
        F: FnOnce() -> C,
    {
        // Note: We use match so the call location gets passed through
        #[cfg(feature = "context")]
        return match self {
            Some(val) => Ok(val),
            None => {
                let widened: ErrorUnion<Self::OutSet> = ErrorUnion::new(AbsentValueError);
                Err(widened.with_context(f))
            }
        };
        // Note: We use match so the call location gets passed through
        #[cfg(not(feature = "context"))]
        return match self {
            Some(val) => Ok(val),
            None => Err(ErrorUnion::new(AbsentValueError)),
        };
    }

    #[cfg(feature = "user_context")]
    #[allow(unused_variables)]
    #[cfg_attr(feature = "location", track_caller)]
    fn user_context<C: Into<ContextValue>>(
        self,
        context: C,
    ) -> Result<T, ErrorUnion<Self::OutSet>> {
        // Note: We use match so the call location gets passed through
        #[cfg(feature = "context")]
        return match self {
            Some(val) => Ok(val),
            None => {
                let widened: ErrorUnion<Self::OutSet> = ErrorUnion::new(AbsentValueError);
                Err(widened.user_context(context))
            }
        };
        // Note: We use match so the call location gets passed through
        #[cfg(not(feature = "context"))]
        return match self {
            Some(val) => Ok(val),
            None => Err(ErrorUnion::new(AbsentValueError)),
        };
    }

    #[cfg(feature = "user_context")]
    #[allow(unused_variables)]
    #[cfg_attr(feature = "location", track_caller)]
    fn with_user_context<F, C: Into<ContextValue>>(
        self,
        f: F,
    ) -> Result<T, ErrorUnion<Self::OutSet>>
    where
        F: FnOnce() -> C,
    {
        // Note: We use match so the call location gets passed through
        #[cfg(feature = "context")]
        return match self {
            Some(val) => Ok(val),
            None => {
                let widened: ErrorUnion<Self::OutSet> = ErrorUnion::new(AbsentValueError);
                Err(widened.with_user_context(f))
            }
        };
        // Note: We use match so the call location gets passed through
        #[cfg(not(feature = "context"))]
        return match self {
            Some(val) => Ok(val),
            None => Err(ErrorUnion::new(AbsentValueError)),
        };
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "An `Option` was unexpectedly `None`")
    }
}

impl core::error::Error for AbsentValueError {}
