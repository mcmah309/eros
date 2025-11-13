use core::any::Any;
use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;

use crate::type_set::{
    Contains, DebugFold, DisplayFold, ErrorFold, IsFold, Narrow, SupersetOf, TupleForm, TypeSet,
};

use crate::{AnyError, Cons, End, StrContext};

/// Any error that satisfies this trait's bounds can be used in a `TracedError`
pub trait SendSyncError: std::any::Any + std::error::Error + Send + Sync + 'static {}

impl<T> SendSyncError for T where T: std::error::Error + Send + Sync + 'static {}

impl std::error::Error for Box<dyn SendSyncError> {}

/* ------------------------- ErrorUnion ----------------------- */

/// `ErrorUnion` is an open sum type. It differs from an enum
/// in that you do not need to define any actual new type
/// in order to hold some specific combination of variants,
/// but rather you simply describe the ErrorUnion as holding
/// one value out of several specific possibilities,
/// defined by using a tuple of those possible variants
/// as the generic parameter for the `ErrorUnion`.
///
/// For example, a `ErrorUnion<(String, u32)>` contains either
/// a `String` or a `u32`. The benefit of this over creating
/// specific enums for each function become apparent in larger
/// codebases where error handling needs to occur in
/// different places for different errors. As such, `ErrorUnion` allows
/// you to quickly specify a function's return value as
/// involving a precise subset of errors that the caller
/// can clearly reason about. Providing maximum composability with
/// no boilerplate.
pub struct TracedUnion<E: TypeSet = (Box<dyn SendSyncError>,)> {
    pub(crate) inner: Box<dyn Any>,
    _pd: PhantomData<E>,
    #[cfg(feature = "backtrace")]
    pub(crate) backtrace: std::backtrace::Backtrace,
    #[cfg(feature = "context")]
    pub(crate) context: Vec<StrContext>,
}

impl<T> Deref for TracedUnion<(T,)>
where
    T: 'static,
{
    type Target = T;

    fn deref(&self) -> &T {
        (self.inner.as_ref() as &dyn Any)
            .downcast_ref::<T>()
            .unwrap()
    }
}

// impl<T> From<T> for TracedUnion<(Box<dyn SendSyncError>,)>
// where
//     T: SendSyncError,
// {
//     fn from(t: T) -> TracedUnion<(Box<dyn SendSyncError>,)> {
//         TracedUnion::any_error(t)
//     }
// }

impl<T> From<T> for TracedUnion<(T,)>
where
    T: SendSyncError,
{
    fn from(t: T) -> TracedUnion<(T,)> {
        TracedUnion::error(t)
    }
}

impl<E> fmt::Debug for TracedUnion<E>
where
    E: TypeSet,
    E::Variants: fmt::Debug + DebugFold,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        E::Variants::debug_fold(
            self.inner.as_ref() as &dyn Any,
            formatter,
            #[cfg(feature = "context")]
            &self.context,
            #[cfg(feature = "backtrace")]
            &self.backtrace,
        )?;
        Ok(())
    }
}

impl<E> fmt::Display for TracedUnion<E>
where
    E: TypeSet,
    E::Variants: fmt::Display + DisplayFold,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        E::Variants::display_fold(self.inner.as_ref() as &dyn Any, formatter)?;
        Ok(())
    }
}

//************************************************************************//

fn _send_sync_error_assert() {
    use std::io;

    fn is_send<T: Send>(_: &T) {}
    fn is_sync<T: Sync>(_: &T) {}
    fn is_error<T: core::error::Error>(_: &T) {}

    let error_union: TracedUnion<(io::Error, fmt::Error)> =
        TracedUnion::error(io::Error::new(io::ErrorKind::Other, "yooo"));
    is_send(&error_union);
    is_sync(&error_union);
    is_error(&&error_union);
}

unsafe impl<T> Send for TracedUnion<T> where T: TypeSet + Send {}
unsafe impl<T> Sync for TracedUnion<T> where T: TypeSet + Sync {}

// Note: Can't implement directly since `Context` trait then has conflicting impls and we could now
// accidentally nest this type
// impl<E> core::error::Error for TracedErrorUnion<E>
// where
//     E: TypeSet,
//     E::Variants: core::error::Error + DebugFold + DisplayFold + ErrorFold,
// {
//     fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
//         E::Variants::source_fold(self.inner.as_ref() as &dyn Any)
//     }
// }

impl<E> core::error::Error for &TracedUnion<E>
where
    E: TypeSet,
    E::Variants: core::error::Error + DebugFold + DisplayFold + ErrorFold,
{
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        E::Variants::source_fold(self.inner.as_ref() as &dyn Any)
    }
}

impl<E> TracedUnion<E>
where
    E: TypeSet + 'static,
    E::Variants: core::error::Error + DebugFold + DisplayFold + ErrorFold,
{
    /// Returns the lower-level source of this error, if any.
    // Note: Even though `std::error::Error` is implemented for Deref.
    // We still redeclare `source` here to tie the lifetime to this,
    // rather than another deref
    pub fn source<'a>(&'a self) -> Option<&'a (dyn std::error::Error + 'static)> {
        let this: &&TracedUnion<E> = &self;
        let source = core::error::Error::source(this);
        // SAFETY: We need to call with `&&` since we need the `&` specialization trick to get the source,
        // since `TracedUnion` directly can't implement `Error` due to trait collisions.
        // This resolves lifetimes correctly back to this call.
        // The underlying `source` will still exist as long as this exists.
        // Since T is `'static` and borrowed for `'a`, and the underlying source is owned
        // by this type, so it can't be moved or dropped while this is borrowed
        let source = unsafe {
            std::mem::transmute::<
                Option<&(dyn core::error::Error + 'static)>,
                Option<&'a (dyn core::error::Error + 'static)>,
            >(source)
        };
        source
    }
}

//************************************************************************//

impl<E> TracedUnion<E>
where
    E: TypeSet,
{
    pub fn new<T, Index>(t: T) -> TracedUnion<E>
    where
        T: 'static,
        E::Variants: Contains<T, Index>,
    {
        TracedUnion {
            inner: Box::new(t),
            _pd: PhantomData,
            #[cfg(feature = "backtrace")]
            backtrace: std::backtrace::Backtrace::capture(),
            #[cfg(feature = "context")]
            context: Vec::new(),
        }
    }

    /// Create a new `ErrorUnion`.
    pub fn error<T, Index>(t: T) -> TracedUnion<E>
    where
        T: SendSyncError,
        E::Variants: Contains<T, Index>,
    {
        TracedUnion {
            inner: Box::new(t),
            _pd: PhantomData,
            #[cfg(feature = "backtrace")]
            backtrace: std::backtrace::Backtrace::capture(),
            #[cfg(feature = "context")]
            context: Vec::new(),
        }
    }

    /// Create a dynamic type erased `TracedError`
    pub fn any_error<T, Index>(source: T) -> TracedUnion<E>
    where
        T: SendSyncError,
        E::Variants: Contains<Box<dyn SendSyncError>, Index>,
    {
        TracedUnion::error(Box::new(source) as Box<dyn SendSyncError>)
    }

    /// Attempt to downcast the `ErrorUnion` into a specific type, and
    /// if that fails, return a `ErrorUnion` which does not contain that
    /// type as one of its possible variants.
    pub fn narrow<Target, Index>(
        self,
    ) -> Result<
        Target,
        TracedUnion<<<E::Variants as Narrow<Target, Index>>::Remainder as TupleForm>::Tuple>,
    >
    where
        Target: 'static,
        E::Variants: Narrow<Target, Index>,
    {
        if (self.inner.as_ref() as &dyn Any).is::<Target>() {
            Ok(*(self.inner as Box<dyn Any>).downcast::<Target>().unwrap())
        } else {
            Err(TracedUnion {
                inner: self.inner,
                _pd: PhantomData,
                #[cfg(feature = "backtrace")]
                backtrace: self.backtrace,
                #[cfg(feature = "context")]
                context: self.context,
            })
        }
    }

    /// Turns the `ErrorUnion` into a `ErrorUnion` with a set of variants
    /// which is a superset of the current one. This may also be
    /// the same set of variants, but in a different order.
    pub fn widen<Other, Index>(self) -> TracedUnion<Other>
    where
        Other: TypeSet,
        Other::Variants: SupersetOf<E::Variants, Index>,
    {
        TracedUnion {
            inner: self.inner,
            _pd: PhantomData,
            #[cfg(feature = "backtrace")]
            backtrace: self.backtrace,
            #[cfg(feature = "context")]
            context: self.context,
        }
    }

    /// Attempt to split a subset of variants out of the `ErrorUnion`,
    /// returning the remainder of possible variants if the value
    /// does not have one of the `TargetList` types.
    pub fn subset<TargetList, Index>(
        self,
    ) -> Result<
        TracedUnion<TargetList>,
        TracedUnion<<<E::Variants as SupersetOf<TargetList::Variants, Index>>::Remainder as TupleForm>::Tuple>,
    >
    where
        TargetList: TypeSet,
        E::Variants: IsFold + SupersetOf<TargetList::Variants, Index>,
    {
        if E::Variants::is_fold(self.inner.as_ref() as &dyn Any) {
            Ok(TracedUnion {
                inner: self.inner,
                _pd: PhantomData,
                #[cfg(feature = "backtrace")]
                backtrace: self.backtrace,
                #[cfg(feature = "context")]
                context: self.context,
            })
        } else {
            Err(TracedUnion {
                inner: self.inner,
                _pd: PhantomData,
                #[cfg(feature = "backtrace")]
                backtrace: self.backtrace,
                #[cfg(feature = "context")]
                context: self.context,
            })
        }
    }

    /// For a `ErrorUnion` with a single variant, return
    /// the contained value.
    pub fn take<Target>(self) -> Target
    where
        Target: 'static,
        E: TypeSet<Variants = Cons<Target, End>>,
    {
        *(self.inner as Box<dyn Any>).downcast::<Target>().unwrap()
    }

    /// Convert the `ErrorUnion` to an owned enum for
    /// use in pattern matching etc...
    pub fn to_enum(self) -> E::Enum
    where
        E::Enum: From<Self>,
    {
        E::Enum::from(self)
    }

    /// Borrow the enum as an enum for use in
    /// pattern matching etc...
    pub fn ref_enum<'a>(&'a self) -> E::RefEnum<'a>
    where
        E::RefEnum<'a>: From<&'a Self>,
    {
        E::RefEnum::from(self)
    }

    pub fn mut_enum<'a>(&'a mut self) -> E::MutEnum<'a>
    where
        E::MutEnum<'a>: From<&'a mut Self>,
    {
        E::MutEnum::from(self)
    }

    #[allow(unused_mut)]
    #[allow(unused_variables)]
    pub fn context<C: Into<StrContext>>(mut self, context: C) -> Self {
        #[cfg(feature = "context")]
        self.context.push(context.into());
        self
    }

    /// Adds additional context lazily. This becomes a no-op if the `traced` feature is disabled.
    #[allow(unused_mut)]
    #[allow(unused_variables)]
    pub fn with_context<F, C: Into<StrContext>>(mut self, f: F) -> Self
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "context")]
        self.context.push(f().into());
        self
    }
}

impl<A: 'static> TracedUnion<(A,)> {
    /// Convert to the inner type of an ErrorUnion with a single possible type.
    pub fn into_inner(self) -> A {
        *self.inner.downcast().unwrap()
    }

    /// Gets a reference to the inner type
    pub fn inner(&self) -> &A {
        self.inner.downcast_ref().unwrap()
    }

    /// Gets a mutable reference to the inner type
    pub fn inner_mut(&mut self) -> &mut A {
        self.inner.downcast_mut().unwrap()
    }

    pub fn map<U, F>(self, f: F) -> TracedUnion<(U,)>
    where
        U: 'static,
        F: FnOnce(A) -> U,
    {
        TracedUnion {
            inner: Box::new(f(*self.inner.downcast().unwrap())),
            _pd: PhantomData,
            #[cfg(feature = "backtrace")]
            backtrace: self.backtrace,
            #[cfg(feature = "context")]
            context: self.context,
        }
    }
}

//************************************************************************//

/// Run widen and narrow directly on Results with ErrorUnions
pub trait ReshapeUnion<S, E>
where
    E: TypeSet,
{
    /// Turns the `ErrorUnion` into a `ErrorUnion` with a set of variants
    /// which is a superset of the current one. This may also be
    /// the same set of variants, but in a different order.
    fn widen<Other, Index>(self) -> Result<S, TracedUnion<Other>>
    where
        Other: TypeSet,
        Other::Variants: SupersetOf<E::Variants, Index>;

    /// Attempt to downcast the `ErrorUnion` into a specific type, and
    /// if that fails, return a `Result` with the `ErrorUnion` wither the remainder
    /// which does not contain that type as one of its possible variants.
    fn narrow<Target, Index>(
        self,
    ) -> Result<
        Target,
        Result<
            S,
            TracedUnion<<<E::Variants as Narrow<Target, Index>>::Remainder as TupleForm>::Tuple>,
        >,
    >
    where
        Target: 'static,
        E::Variants: Narrow<Target, Index>;
}

impl<S, E> ReshapeUnion<S, E> for Result<S, TracedUnion<E>>
where
    E: TypeSet,
{
    fn widen<Other, Index>(self) -> Result<S, TracedUnion<Other>>
    where
        Other: TypeSet,
        Other::Variants: SupersetOf<E::Variants, Index>,
    {
        self.map_err(|e| e.widen())
    }

    fn narrow<Target, Index>(
        self,
    ) -> Result<
        Target,
        Result<
            S,
            TracedUnion<<<E::Variants as Narrow<Target, Index>>::Remainder as TupleForm>::Tuple>,
        >,
    >
    where
        Target: 'static,
        E::Variants: Narrow<Target, Index>,
    {
        match self {
            Ok(value) => Err(Ok(value)),
            Err(err) => match err.narrow() {
                Ok(value) => return Ok(value),
                Err(err) => Err(Err(err)),
            },
        }
    }
}

//************************************************************************//

pub trait Union<S, F> {
    /// Creates an `ErrorUnion` for this type.
    fn union<Index, Other>(self) -> Result<S, TracedUnion<Other>>
    where
        Other: TypeSet,
        Other::Variants: Contains<F, Index>;
}

impl<S, F: SendSyncError> Union<S, F> for Result<S, F> {
    fn union<Index, Other>(self) -> Result<S, TracedUnion<Other>>
    where
        Other: TypeSet,
        // Other::Variants: SupersetOf<Cons<F, End>, Index>,
        Other::Variants: Contains<F, Index>,
    {
        self.map_err(TracedUnion::error)
    }
}

pub trait IntoUnion<S, F> {
    /// Creates an `ErrorUnion` for this type.
    fn into_union<Index, Other>(self) -> Result<S, TracedUnion<Other>>
    where
        Other: TypeSet,
        Other::Variants: Contains<F, Index>;
}

impl<S, F1, F2> IntoUnion<S, F2> for Result<S, F1>
where
    F1: Into<F2> + SendSyncError, // `SendSyncError` is used to ensure it does not overlap with below
    F2: 'static,
{
    fn into_union<Index, Other>(self) -> Result<S, TracedUnion<Other>>
    where
        Other: TypeSet,
        Other::Variants: Contains<F2, Index>,
    {
        self.map_err(|e| TracedUnion::new(e.into()))
    }
}

// todo better name?
pub trait IntoUnionSingle<S, F> {
    fn into_union(self) -> Result<S, TracedUnion<(F,)>>;

}

impl<S, F1, F2> IntoUnionSingle<S, F2> for Result<S, TracedUnion<(F1,)>>
where
    F1: Into<F2> + 'static,
    F2: 'static,
{
    fn into_union(self) -> Result<S, TracedUnion<(F2,)>>
    {
        self.map_err(|e| e.map(|e| e.into()))
    }
}

// Commented out since it conflicts with the above and cant merge into one trait since the return type is attached to the method generic
// pub trait IntoUnion<F>
// where
//     F: AnyError,
// {
//     fn union<Index, Other>(self) -> ErrorUnion<Other>
//     where
//         Other: TypeSet,
//         // Other::Variants: SupersetOf<Cons<F, End>, Index>,
//         Other::Variants: Contains<F, Index>;
// }

// impl<F> IntoUnion<F> for F
// where
//     F: AnyError,
// {
//     fn union<Index, Other>(self) -> ErrorUnion<Other>
//     where
//         Other: TypeSet,
//         // Other::Variants: SupersetOf<Cons<F, End>, Index>,
//         Other::Variants: Contains<F, Index>,
//     {
//         ErrorUnion::new(self)
//     }
// }
