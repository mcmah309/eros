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

/* ------------------------- OneOf ----------------------- */

/// `OneOf` is an open sum type. It differs from an enum
/// in that you do not need to define any actual new type
/// in order to hold some specific combination of variants,
/// but rather you simply describe the OneOf as holding
/// one value out of several specific possibilities,
/// defined by using a tuple of those possible variants
/// as the generic parameter for the `OneOf`.
///
/// For example, a `OneOf<(String, u32)>` contains either
/// a `String` or a `u32`. The value over a simple `Result`
/// or other traditional enum starts to become apparent in larger
/// codebases where error handling needs to occur in
/// different places for different errors. `OneOf` allows
/// you to quickly specify a function's return value as
/// involving a precise subset of errors that the caller
/// can clearly reason about.
pub struct U<E: TypeSet> {
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

    let o: U<(io::Error,)> = U::new(io::Error::new(io::ErrorKind::Other, "yooo"));
    is_send(&o);
    is_sync(&o);
    is_error(&o);
}

unsafe impl<T> Send for U<T> where T: TypeSet + Send {}
unsafe impl<T> Sync for U<T> where T: TypeSet + Sync {}

impl<T> Deref for U<(T,)>
where
    T: 'static,
{
    type Target = T;

    fn deref(&self) -> &T {
        self.value.downcast_ref::<T>().unwrap()
    }
}

impl<T> From<T> for U<(T,)>
where
    T: 'static,
{
    fn from(t: T) -> U<(T,)> {
        U::new(t)
    }
}

impl<E> fmt::Debug for U<E>
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

impl<E> fmt::Display for U<E>
where
    E: TypeSet,
    E::Variants: fmt::Display + DisplayFold,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        E::Variants::display_fold(&self.value, formatter)?;
        if !self.context.is_empty() {
            if !self.value.is::<()>() {
                write!(formatter, "\n\n")?;
            }
            write!(formatter, "Context:")?;
            for context_item in self.context.iter() {
                write!(formatter, "\n    - {}", context_item)?;
            }
        }
        Ok(())
    }
}

impl<E> Error for U<E>
where
    E: TypeSet,
    E::Variants: Error + DisplayFold + ErrorFold,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        E::Variants::source_fold(&self.value)
    }
}

impl<E> U<E>
where
    E: TypeSet,
{
    /// Create a new `OneOf`.
    pub fn new<T, Index>(t: T) -> U<E>
    where
        T: Any,
        E::Variants: Contains<T, Index>,
    {
        U {
            value: Box::new(t),
            context: Vec::new(),
            backtrace: Backtrace::capture(),
            _pd: PhantomData,
        }
    }

    /// Attempt to downcast the `OneOf` into a specific type, and
    /// if that fails, return a `OneOf` which does not contain that
    /// type as one of its possible variants.
    pub fn narrow<Target, Index>(
        self,
    ) -> Result<
        Target,
        U<<<E::Variants as Narrow<Target, Index>>::Remainder as TupleForm>::Tuple>,
    >
    where
        Target: 'static,
        E::Variants: Narrow<Target, Index>,
    {
        if self.value.is::<Target>() {
            Ok(*self.value.downcast::<Target>().unwrap())
        } else {
            Err(U {
                value: self.value,
                context: self.context,
                backtrace: self.backtrace,
                _pd: PhantomData,
            })
        }
    }

    /// Turns the `OneOf` into a `OneOf` with a set of variants
    /// which is a superset of the current one. This may also be
    /// the same set of variants, but in a different order.
    pub fn broaden<Other, Index>(self) -> U<Other>
    where
        Other: TypeSet,
        Other::Variants: SupersetOf<E::Variants, Index>,
    {
        U {
            value: self.value,
            context: self.context,
            backtrace: self.backtrace,
            _pd: PhantomData,
        }
    }

    /// Attempt to split a subset of variants out of the `OneOf`,
    /// returning the remainder of possible variants if the value
    /// does not have one of the `TargetList` types.
    pub fn subset<TargetList, Index>(
        self,
    ) -> Result<
        U<TargetList>,
        U<<<E::Variants as SupersetOf<TargetList::Variants, Index>>::Remainder as TupleForm>::Tuple>,
    >
    where
        TargetList: TypeSet,
        E::Variants: IsFold + SupersetOf<TargetList::Variants, Index>,
    {
        if E::Variants::is_fold(&self.value) {
            Ok(U {
                value: self.value,
                context: self.context,
                backtrace: self.backtrace,
                _pd: PhantomData,
            })
        } else {
            Err(U {
                value: self.value,
                context: self.context,
                backtrace: self.backtrace,
                _pd: PhantomData,
            })
        }
    }

    /// For a `OneOf` with a single variant, return
    /// the contained value.
    pub fn take<Target>(self) -> Target
    where
        Target: 'static,
        E: TypeSet<Variants = Cons<Target, End>>,
    {
        *self.value.downcast::<Target>().unwrap()
    }

    /// Convert the `OneOf` to an owned enum for
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
