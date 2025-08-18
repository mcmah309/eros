use core::any::Any;
use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;
use std::backtrace::Backtrace;
use std::error::Error;

use crate::string_kind::StringKind;
use crate::type_set::{
    Contains, DisplayFold, ErrorFold, IsFold, Narrow, SupersetOf, TupleForm, TypeSet,
};

use crate::{Cons, End};

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
    pub(crate) value: Box<dyn Any>,
    pub(crate) backtrace: Backtrace,
    pub(crate) context: Vec<StringKind>,
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
        self.value.downcast_ref::<T>().unwrap()
    }
}

impl<T> From<T> for ErrorUnion<(T,)>
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
    E::Variants: fmt::Display + DisplayFold,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)?;
        write!(formatter, "\n\nBacktrace:\n")?;
        fmt::Display::fmt(&self.backtrace, formatter)?;
        Ok(())
    }
}

impl<E> fmt::Display for ErrorUnion<E>
where
    E: TypeSet,
    E::Variants: fmt::Display + DisplayFold,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        E::Variants::display_fold(&self.value, formatter)?;
        if !self.context.is_empty() {
            write!(formatter, "\n\nContext:")?;
            for context_item in self.context.iter() {
                write!(formatter, "\n\t- {}", context_item)?;
            }
        }
        Ok(())
    }
}

impl<E> Error for ErrorUnion<E>
where
    E: TypeSet,
    E::Variants: Error + DisplayFold + ErrorFold,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        E::Variants::source_fold(&self.value)
    }
}

impl<E> ErrorUnion<E>
where
    E: TypeSet,
{
    /// Create a new `ErrorUnion`.
    pub fn new<T, Index>(t: T) -> ErrorUnion<E>
    where
        T: Any,
        E::Variants: Contains<T, Index>,
    {
        ErrorUnion {
            value: Box::new(t),
            context: Vec::new(),
            backtrace: Backtrace::capture(),
            _pd: PhantomData,
        }
    }

    pub(crate) fn new_internal<T, Index>(t: T, context: Vec<StringKind>, backtrace: Backtrace) -> ErrorUnion<E>
    where
        T: Any,
        E::Variants: Contains<T, Index>,
    {
        ErrorUnion {
            value: Box::new(t),
            context,
            backtrace,
            _pd: PhantomData,
        }
    }

    /// Attempt to downcast the `ErrorUnion` into a specific type, and
    /// if that fails, return a `ErrorUnion` which does not contain that
    /// type as one of its possible variants.
    pub fn deflate<Target, Index>(
        self,
    ) -> Result<
        Target,
        ErrorUnion<<<E::Variants as Narrow<Target, Index>>::Remainder as TupleForm>::Tuple>,
    >
    where
        Target: 'static,
        E::Variants: Narrow<Target, Index>,
    {
        if self.value.is::<Target>() {
            Ok(*self.value.downcast::<Target>().unwrap())
        } else {
            Err(ErrorUnion {
                value: self.value,
                context: self.context,
                backtrace: self.backtrace,
                _pd: PhantomData,
            })
        }
    }

    /// Turns the `ErrorUnion` into a `ErrorUnion` with a set of variants
    /// which is a superset of the current one. This may also be
    /// the same set of variants, but in a different order.
    pub fn inflate<Other, Index>(self) -> ErrorUnion<Other>
    where
        Other: TypeSet,
        Other::Variants: SupersetOf<E::Variants, Index>,
    {
        ErrorUnion {
            value: self.value,
            context: self.context,
            backtrace: self.backtrace,
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
        if E::Variants::is_fold(&self.value) {
            Ok(ErrorUnion {
                value: self.value,
                context: self.context,
                backtrace: self.backtrace,
                _pd: PhantomData,
            })
        } else {
            Err(ErrorUnion {
                value: self.value,
                context: self.context,
                backtrace: self.backtrace,
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
        *self.value.downcast::<Target>().unwrap()
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