use core::any::Any;
use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;
use std::error::Error;

use crate::context::Contextable;
use crate::generic_error::AnyError;
use crate::type_set::{
    Contains, DebugFold, DisplayFold, ErrorFold, IsFold, Narrow, SupersetOf, TupleForm, TypeSet,
};

use crate::{Cons, End, TracedError};

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
/// a `String` or a `u32`. The value over a simple `Result`
/// or other traditional enum starts to become apparent in larger
/// codebases where error handling needs to occur in
/// different places for different errors. `ErrorUnion` allows
/// you to quickly specify a function's return value as
/// involving a precise subset of errors that the caller
/// can clearly reason about.
///
/// `ErrorUnion` also holds the the root backtrace and context provided
/// throughout the call chain.
pub struct ErrorUnion<E: TypeSet> {
    pub(crate) value: Box<dyn Contextable>,
    _pd: PhantomData<E>,
}

fn _send_sync_error_assert() {
    use std::io;

    fn is_send<T: Send>(_: &T) {}
    fn is_sync<T: Sync>(_: &T) {}
    fn is_error<T: Error>(_: &T) {}

    let o: ErrorUnion<(io::Error,)> = ErrorUnion::new(io::Error::new(io::ErrorKind::Other, "yooo"));
    is_send(&o);
    is_sync(&o);
    is_error(&o);
}

unsafe impl<T> Send for ErrorUnion<T> where T: TypeSet + Send {}
unsafe impl<T> Sync for ErrorUnion<T> where T: TypeSet + Sync {}

impl<T> Deref for ErrorUnion<(T,)>
where
    T: 'static,
{
    type Target = T;

    fn deref(&self) -> &T {
        (self.value.as_ref() as &dyn Any)
            .downcast_ref::<T>()
            .unwrap()
    }
}

impl<T: Contextable> From<T> for ErrorUnion<(T,)>
where
    T: 'static,
{
    fn from(t: T) -> ErrorUnion<(T,)> {
        ErrorUnion::new(t)
    }
}

impl<E> fmt::Debug for ErrorUnion<E>
where
    E: TypeSet,
    E::Variants: fmt::Debug + DebugFold,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        E::Variants::debug_fold(self.value.as_ref() as &dyn Any, formatter)?;
        Ok(())
    }
}

impl<E> fmt::Display for ErrorUnion<E>
where
    E: TypeSet,
    E::Variants: fmt::Display + DisplayFold,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        E::Variants::display_fold(self.value.as_ref() as &dyn Any, formatter)?;
        Ok(())
    }
}

impl<E> Error for ErrorUnion<E>
where
    E: TypeSet,
    E::Variants: Error + DebugFold + DisplayFold + ErrorFold,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        E::Variants::source_fold(self.value.as_ref() as &dyn Any)
    }
}

impl<E> ErrorUnion<E>
where
    E: TypeSet,
{
    /// Create a new `ErrorUnion`.
    pub fn new<T, Index>(t: T) -> ErrorUnion<E>
    where
        T: Contextable,
        E::Variants: Contains<T, Index>,
    {
        ErrorUnion {
            value: Box::new(t),
            _pd: PhantomData,
        }
    }

    /// Attempt to downcast the `ErrorUnion` into a specific type, and
    /// if that fails, return a `ErrorUnion` which does not contain that
    /// type as one of its possible variants.
    pub fn narrow<Target, Index>(
        self,
    ) -> Result<
        Target,
        ErrorUnion<<<E::Variants as Narrow<Target, Index>>::Remainder as TupleForm>::Tuple>,
    >
    where
        Target: 'static,
        E::Variants: Narrow<Target, Index>,
    {
        if (self.value.as_ref() as &dyn Any).is::<Target>() {
            Ok(*(self.value as Box<dyn Any>).downcast::<Target>().unwrap())
        } else {
            Err(ErrorUnion {
                value: self.value,
                _pd: PhantomData,
            })
        }
    }

    /// Turns the `ErrorUnion` into a `ErrorUnion` with a set of variants
    /// which is a superset of the current one. This may also be
    /// the same set of variants, but in a different order.
    pub fn widen<Other, Index>(self) -> ErrorUnion<Other>
    where
        Other: TypeSet,
        Other::Variants: SupersetOf<E::Variants, Index>,
    {
        ErrorUnion {
            value: self.value,
            _pd: PhantomData,
        }
    }

    /// Attempt to split a subset of variants out of the `ErrorUnion`,
    /// returning the remainder of possible variants if the value
    /// does not have one of the `TargetList` types.
    pub fn subset<TargetList, Index>(
        self,
    ) -> Result<
        ErrorUnion<TargetList>,
        ErrorUnion<<<E::Variants as SupersetOf<TargetList::Variants, Index>>::Remainder as TupleForm>::Tuple>,
    >
    where
        TargetList: TypeSet,
        E::Variants: IsFold + SupersetOf<TargetList::Variants, Index>,
    {
        if E::Variants::is_fold(self.value.as_ref() as &dyn Any) {
            Ok(ErrorUnion {
                value: self.value,
                _pd: PhantomData,
            })
        } else {
            Err(ErrorUnion {
                value: self.value,
                _pd: PhantomData,
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
        *(self.value as Box<dyn Any>).downcast::<Target>().unwrap()
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
    pub fn as_enum<'a>(&'a self) -> E::EnumRef<'a>
    where
        E::EnumRef<'a>: From<&'a Self>,
    {
        E::EnumRef::from(&self)
    }
}

impl<T: 'static> ErrorUnion<(T,)> {
    pub fn into_inner(self) -> T {
        match self.to_enum() {
            crate::E1::A(inner) => inner,
        }
    }
}

impl<T: AnyError> ErrorUnion<(TracedError<T>,)> {
    // Note: overrides the trait so we don't just wrap in another TracedError
    pub fn traced(self) -> TracedError<T> {
        self.into_inner()
    }
}

impl ErrorUnion<(TracedError,)> {
    // Note: overrides the trait so we don't just wrap in another TracedError
    pub fn traced_dyn(self) -> TracedError {
        self.into_inner()
    }
}

//************************************************************************//

/// Run widen and narrow directly on Results with ErrorUnions
pub trait ReshapeUnionResult<S, E>
where
    E: TypeSet,
{
    /// Turns the `ErrorUnion` into a `ErrorUnion` with a set of variants
    /// which is a superset of the current one. This may also be
    /// the same set of variants, but in a different order.
    fn widen<Other, Index>(self) -> Result<S, ErrorUnion<Other>>
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
            ErrorUnion<<<E::Variants as Narrow<Target, Index>>::Remainder as TupleForm>::Tuple>,
        >,
    >
    where
        Target: 'static,
        E::Variants: Narrow<Target, Index>;
}

impl<S, E> ReshapeUnionResult<S, E> for Result<S, ErrorUnion<E>>
where
    E: TypeSet,
{
    fn widen<Other, Index>(self) -> Result<S, ErrorUnion<Other>>
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
            ErrorUnion<<<E::Variants as Narrow<Target, Index>>::Remainder as TupleForm>::Tuple>,
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

pub trait IntoUnionResult<S, F> {
    fn union<Index, Other>(self) -> Result<S, ErrorUnion<Other>>
    where
        Other: TypeSet,
        // Other::Variants: SupersetOf<Cons<F, End>, Index>,
        Other::Variants: Contains<F, Index>;
}

impl<S, F: 'static> IntoUnionResult<S, F> for Result<S, F> {
    fn union<Index, Other>(self) -> Result<S, ErrorUnion<Other>>
    where
        Other: TypeSet,
        // Other::Variants: SupersetOf<Cons<F, End>, Index>,
        Other::Variants: Contains<F, Index>,
    {
        self.map_err(ErrorUnion::new)
    }
}

// Commented out since it conflicts with the above and cant merge into one trait since the return type is attached to the method generic
// pub trait IntoUnion<F>
// where
//     F: Contextable,
// {
//     fn union<Index, Other>(self) -> ErrorUnion<Other>
//     where
//         Other: TypeSet,
//         // Other::Variants: SupersetOf<Cons<F, End>, Index>,
//         Other::Variants: Contains<F, Index>;
// }

// impl<F> IntoUnion<F> for F
// where
//     F: Contextable,
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
