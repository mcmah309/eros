use core::any::Any;
use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;
use std::any::TypeId;
use std::{backtrace, mem, ptr};

use crate::type_set::{
    write_debug, write_display, Contains, DebugFold, DisplayFold, ErrorFold, IsFold, Narrow,
    SupersetOf, TupleForm, TypeSet,
};

use crate::{AnyError, Cons, End, StrContext};

/// Any error that satisfies this trait's bounds can be used in a `TracedError`
pub trait SendSyncError: std::any::Any + std::error::Error + Send + Sync + 'static {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T> SendSyncError for T
where
    T: std::error::Error + Send + Sync + 'static,
{
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl std::error::Error for Box<dyn SendSyncError> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&**self)
    }
}

pub(crate) struct TracedUnionInner<T: ?Sized> {
    #[cfg(feature = "backtrace")]
    pub(crate) backtrace: std::backtrace::Backtrace,
    #[cfg(feature = "context")]
    pub(crate) context: Vec<StrContext>,
    #[cfg(feature = "location")]
    pub(crate) location: &'static std::panic::Location<'static>,
    pub(crate) error: T,
}

impl TracedUnionInner<dyn SendSyncError> {
    #[cfg_attr(feature = "location", track_caller)]
    pub(crate) fn new<T>(t: T) -> Box<TracedUnionInner<dyn SendSyncError>>
    where
        T: SendSyncError,
    {
        Box::new(TracedUnionInner {
            #[cfg(feature = "backtrace")]
            backtrace: std::backtrace::Backtrace::capture(),
            #[cfg(feature = "context")]
            context: Vec::new(),
            #[cfg(feature = "location")]
            location: std::panic::Location::caller(),
            error: t,
        })
    }

    #[cfg_attr(feature = "location", track_caller)]
    pub(crate) fn new_from_parts<T>(
        t: T,
        #[cfg(feature = "backtrace")] backtrace: std::backtrace::Backtrace,
        #[cfg(feature = "context")] context: Vec<StrContext>,
        #[cfg(feature = "location")] location: &'static std::panic::Location<'static>,
    ) -> Box<TracedUnionInner<dyn SendSyncError>>
    where
        T: SendSyncError,
    {
        Box::new(TracedUnionInner {
            #[cfg(feature = "backtrace")]
            backtrace,
            #[cfg(feature = "context")]
            context,
            #[cfg(feature = "location")]
            location,
            error: t,
        })
    }

    #[allow(unstable_name_collisions)]
    pub(crate) fn is_error<T: 'static>(&self) -> bool {
        self.error.type_id() == TypeId::of::<T>()
    }

    pub(crate) unsafe fn downcast_error_unchecked<T: 'static>(self: Box<Self>) -> T {
        debug_assert!(self.is_error::<T>());

        // Note: this prevents the Box from automatically dropping at the end of the function.
        let raw_container: *mut Self = Box::into_raw(self);

        // Thin the fat pointer directly — no intermediate dyn Any cast needed.
        // addr_of! gives *const dyn SendSyncError (fat), casting to *const T thins it.
        let thin_ptr = ptr::addr_of!((*raw_container).error) as *const T;
        // Copy to the stack
        let downcasted_value: T = ptr::read(thin_ptr);

        // Destructively drop the remaining fields inside the container
        #[cfg(feature = "backtrace")]
        ptr::drop_in_place(ptr::addr_of_mut!((*raw_container).backtrace));
        #[cfg(feature = "context")]
        ptr::drop_in_place(ptr::addr_of_mut!((*raw_container).context));
        #[cfg(feature = "location")]
        ptr::drop_in_place(ptr::addr_of_mut!((*raw_container).location));

        // Deallocate the Box allocation itself.
        // We reconstruct a Box containing uninitialized/dead data, but wrapped in
        // ManuallyDrop so its fields aren't dropped. When this `dead_box` goes out of scope,
        // it frees the underlying heap memory without touching the fields.
        let _dead_box: Box<mem::ManuallyDrop<Self>> =
            Box::from_raw(raw_container as *mut mem::ManuallyDrop<Self>);

        downcasted_value
    }

    pub(crate) unsafe fn downcast_error_unchecked_with_parts<T: 'static>(
        self: Box<Self>,
    ) -> TracedUnionInner<T> {
        debug_assert!(self.is_error::<T>());

        // Note: this prevents the Box from automatically dropping at the end of the function.
        let raw_container: *mut Self = Box::into_raw(self);

        // Thin the fat pointer directly — no intermediate dyn Any cast needed.
        // addr_of! gives *const dyn SendSyncError (fat), casting to *const T thins it.
        let thin_ptr = ptr::addr_of!((*raw_container).error) as *const T;
        // Copy to the stack
        let downcasted_value: T = ptr::read(thin_ptr);

        // Read the additional parts before dropping
        #[cfg(feature = "backtrace")]
        let backtrace = ptr::read(ptr::addr_of!((*raw_container).backtrace));
        #[cfg(feature = "context")]
        let context = ptr::read(ptr::addr_of!((*raw_container).context));
        #[cfg(feature = "location")]
        let location = ptr::read(ptr::addr_of!((*raw_container).location));

        // Deallocate the Box allocation itself.
        // We reconstruct a Box containing uninitialized/dead data, but wrapped in
        // ManuallyDrop so its fields aren't dropped. When this `dead_box` goes out of scope,
        // it frees the underlying heap memory without touching the fields.
        let _dead_box: Box<mem::ManuallyDrop<Self>> =
            Box::from_raw(raw_container as *mut mem::ManuallyDrop<Self>);

        TracedUnionInner {
            #[cfg(feature = "backtrace")]
            backtrace,
            #[cfg(feature = "context")]
            context,
            #[cfg(feature = "location")]
            location,
            error: downcasted_value,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn downcast_error<T: 'static>(self: Box<Self>) -> T {
        if self.is_error::<T>() {
            unsafe { self.downcast_error_unchecked() }
        } else {
            panic!(
                "Attempted to downcast to {}, but actual type was different",
                std::any::type_name::<T>()
            );
        }
    }

    pub(crate) fn downcast_error_ref<T: 'static>(&self) -> &T {
        (&self.error as &dyn Any).downcast_ref::<T>().unwrap()
    }

    pub(crate) fn downcast_error_mut<T: 'static>(&mut self) -> &mut T {
        (&mut self.error as &mut dyn Any)
            .downcast_mut::<T>()
            .unwrap()
    }
}

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
pub struct TracedUnion<E: TypeSet = AnyError> {
    pub(crate) inner: Box<TracedUnionInner<dyn SendSyncError>>,
    pub(crate) _pd: PhantomData<E>,
}

impl<T> Deref for TracedUnion<(T,)>
where
    T: 'static,
{
    type Target = T;

    fn deref(&self) -> &T {
        (&self.inner.error as &dyn Any).downcast_ref::<T>().unwrap()
    }
}

impl fmt::Debug for TracedUnion<AnyError> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_debug(
            &self.inner.error,
            formatter,
            #[cfg(feature = "context")]
            &self.inner.context,
            #[cfg(feature = "backtrace")]
            &self.inner.backtrace,
            #[cfg(feature = "location")]
            self.inner.location,
        )
    }
}

impl<E> fmt::Debug for TracedUnion<E>
where
    E: TypeSet,
    E::Variants: fmt::Debug + DebugFold,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        E::Variants::debug_fold(
            &self.inner.error,
            formatter,
            #[cfg(feature = "context")]
            &self.inner.context,
            #[cfg(feature = "backtrace")]
            &self.inner.backtrace,
            #[cfg(feature = "location")]
            self.inner.location,
        )?;
        Ok(())
    }
}

impl fmt::Display for TracedUnion<AnyError> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_display(&self.inner.error, formatter)
    }
}

impl<E> fmt::Display for TracedUnion<E>
where
    E: TypeSet,
    E::Variants: fmt::Display + DisplayFold,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        E::Variants::display_fold(&self.inner.error as &dyn Any, formatter)?;
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
        TracedUnion::new(io::Error::other("yooo"));
    is_send(&error_union);
    is_sync(&error_union);
    // is_error(&error_union); //todo
}

unsafe impl<T> Send for TracedUnion<T> where T: TypeSet + Send {}
unsafe impl<T> Sync for TracedUnion<T> where T: TypeSet + Sync {}

//************************************************************************//

impl TracedUnion {
    /// Create a new `ErrorUnion`.
    #[cfg_attr(feature = "location", track_caller)]
    pub fn new<T, OutSet, Index>(t: T) -> TracedUnion<OutSet>
    where
        T: SendSyncError,
        OutSet: TypeSet,
        OutSet::Variants: Contains<T, Index>,
    {
        TracedUnion {
            inner: TracedUnionInner::new(t),
            _pd: PhantomData,
        }
    }

    pub fn new_from_parts<T, OutSet, Index>(
        t: T,
        #[cfg(feature = "backtrace")] backtrace: std::backtrace::Backtrace,
        #[cfg(feature = "context")] context: Vec<StrContext>,
        #[cfg(feature = "location")] location: &'static std::panic::Location<'static>,
    ) -> TracedUnion<OutSet>
    where
        T: SendSyncError,
        OutSet: TypeSet,
        OutSet::Variants: Contains<T, Index>,
    {
        TracedUnion {
            inner: TracedUnionInner::new_from_parts(
                t,
                #[cfg(feature = "backtrace")]
                backtrace,
                #[cfg(feature = "context")]
                context,
                #[cfg(feature = "location")]
                location,
            ),
            _pd: PhantomData,
        }
    }

    pub(crate) fn erase<E>(t: TracedUnion<E>) -> TracedUnion
    where
        E: TypeSet,
    {
        TracedUnion {
            inner: t.inner,
            _pd: PhantomData,
        }
    }
}

//************************************************************************//

struct TracedUnionErrorWrapper<E>(TracedUnion<E>)
where
    E: TypeSet;

impl<E> std::error::Error for TracedUnionErrorWrapper<E>
where
    E: TypeSet,
    E::Variants: std::error::Error + DebugFold + DisplayFold + ErrorFold,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        E::Variants::source_fold(&self.0.inner.error as &dyn Any)
    }
}

impl<E> fmt::Display for TracedUnionErrorWrapper<E>
where
    E: TypeSet,
    E::Variants: fmt::Display + DisplayFold,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

impl<E> fmt::Debug for TracedUnionErrorWrapper<E>
where
    E: TypeSet,
    E::Variants: fmt::Debug + DebugFold,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

impl<E> TracedUnion<E>
where
    E: TypeSet + Send + Sync + 'static,
    E::Variants: std::error::Error + DebugFold + DisplayFold + ErrorFold,
{
    /// Creates a `Box<dyn SendSyncError>` error from this [`crate::TracedUnion`]. This is used since
    /// [`crate::TracedUnion`] cannot implement [`std::error::Error`] directly, otherwise trait implementations
    /// that require this bounds would conflict. To convert back into a [`crate::TracedUnion`],
    /// [`crate::TracedUnion::from_dyn_error`] must be used.
    pub fn into_dyn_error(self) -> Box<dyn SendSyncError> {
        Box::new(TracedUnionErrorWrapper(self)) as Box<dyn SendSyncError>
    }

    /// See [`crate::TracedUnion::into_dyn_error`].
    pub fn from_dyn_error(error: Box<dyn SendSyncError>) -> Result<Self, Box<dyn SendSyncError>> {
        let error_ref = &*error as &dyn Any;
        if !error_ref.is::<TracedUnionErrorWrapper<E>>() {
            return Err(error);
        }
        let error = error as Box<dyn Any>;
        let traced_union_wrapper = error.downcast::<TracedUnionErrorWrapper<E>>().unwrap();
        Ok(traced_union_wrapper.0)
    }
}

//************************************************************************//

impl<E> TracedUnion<E>
where
    E: TypeSet,
{
    /// Attempt to downcast the `ErrorUnion` into a specific type, and
    /// if that fails, return a `ErrorUnion` which does not contain that
    /// type as one of its possible variants.
    #[allow(clippy::type_complexity)]
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
        if self.inner.is_error::<Target>() {
            Ok(unsafe { self.inner.downcast_error_unchecked::<Target>() })
        } else {
            Err(TracedUnion {
                inner: self.inner,
                _pd: PhantomData,
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
        }
    }

    /// Attempt to split a subset of variants out of the `ErrorUnion`,
    /// returning the remainder of possible variants if the value
    /// does not have one of the `TargetList` types.
    #[allow(clippy::type_complexity)]
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
        if E::Variants::is_fold(&self.inner.error as &dyn Any) {
            Ok(TracedUnion {
                inner: self.inner,
                _pd: PhantomData,
            })
        } else {
            Err(TracedUnion {
                inner: self.inner,
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
        unsafe { self.inner.downcast_error_unchecked::<Target>() }
    }

    pub fn error_ref(&self) -> &dyn SendSyncError {
        &self.inner.error
    }

    pub fn error_ref_any(&self) -> &dyn Any {
        &self.inner.error
    }

    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.error.source()
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
        self.inner.context.push(context.into());
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
        self.inner.context.push(f().into());
        self
    }
}

impl<A: 'static> TracedUnion<(A,)> {
    /// Convert to the inner type of an ErrorUnion with a single possible type.
    pub fn into_inner(self) -> A {
        unsafe { self.inner.downcast_error_unchecked() }
    }

    /// Gets a reference to the inner type
    pub fn inner(&self) -> &A {
        self.inner.downcast_error_ref()
    }

    /// Gets a mutable reference to the inner type
    pub fn inner_mut(&mut self) -> &mut A {
        self.inner.downcast_error_mut()
    }

    pub fn map<U, F>(self, f: F) -> TracedUnion<(U,)>
    where
        U: SendSyncError,
        F: FnOnce(A) -> U,
    {
        // SAFETY: We know that the inner error is only of type A, so we can safely downcast it
        let inner = unsafe { self.inner.downcast_error_unchecked_with_parts::<A>() };
        TracedUnion {
            inner: TracedUnionInner::new_from_parts(
                f(inner.error),
                #[cfg(feature = "backtrace")]
                inner.backtrace,
                #[cfg(feature = "context")]
                inner.context,
                #[cfg(feature = "location")]
                inner.location,
            ),
            _pd: PhantomData,
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
    #[allow(clippy::type_complexity)]
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
                Ok(value) => Ok(value),
                Err(err) => Err(Err(err)),
            },
        }
    }
}

//************************************************************************//

pub trait IntoUnion<S, F> {
    /// Creates an `ErrorUnion` for this type.
    fn into_union<Index, Other>(self) -> Result<S, TracedUnion<Other>>
    where
        Other: TypeSet,
        Other::Variants: Contains<F, Index>;
}

impl<S, F: SendSyncError> IntoUnion<S, F> for Result<S, F> {
    #[cfg_attr(feature = "location", track_caller)]
    fn into_union<Index, Other>(self) -> Result<S, TracedUnion<Other>>
    where
        Other: TypeSet,
        Other::Variants: Contains<F, Index>,
    {
        self.map_err(TracedUnion::new)
    }
}

pub trait IntoDynUnion<S> {
    /// Creates an `ErrorUnion` for this type.
    fn into_dyn_union(self) -> Result<S, TracedUnion>;
}

impl<S, F: SendSyncError> IntoDynUnion<S> for Result<S, F> {
    fn into_dyn_union(self) -> Result<S, TracedUnion> {
        self.map_err(|e| e.into())
    }
}

impl<S, E: TypeSet> IntoDynUnion<S> for Result<S, TracedUnion<E>> {
    fn into_dyn_union(self) -> Result<S, TracedUnion> {
        self.map_err(|e| TracedUnion::erase(e))
    }
}

// pub trait IntoUnion<S, F> {
//     /// Con `Err` to i
//     fn into_union<Index, Other>(self) -> Result<S, TracedUnion<Other>>
//     where
//         Other: TypeSet,
//         Other::Variants: Contains<F, Index>;
// }

// impl<S, F1, F2> IntoUnion<S, F2> for Result<S, F1>
// where
//     F1: Into<F2> + SendSyncError, // `SendSyncError` is used to ensure it does not overlap with below
//     F2: SendSyncError,
// {
//     fn into_union<Index, Other>(self) -> Result<S, TracedUnion<Other>>
//     where
//         Other: TypeSet,
//         Other::Variants: Contains<F2, Index>,
//     {
//         self.map_err(|e| TracedUnion::new(e.into()))
//     }
// }

// pub trait InnerInto<S, F> {
//     /// Converts the inner type of an `TracedUnion` into another type
//     fn inner_into(self) -> Result<S, TracedUnion<(F,)>>;
// }

// impl<S, F1, F2> InnerInto<S, F2> for Result<S, TracedUnion<(F1,)>>
// where
//     F1: Into<F2> + SendSyncError,
//     F2: SendSyncError,
// {
//     fn inner_into(self) -> Result<S, TracedUnion<(F2,)>> {
//         self.map_err(|e| e.map(|e| e.into()))
//     }
// }

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

//************************************************************************//

// anyhow::Error does not implement `std::error::Error` so we need to wrap it
#[derive(Debug)]
pub(crate) struct AnyhowError(pub(crate) anyhow::Error);

impl fmt::Display for AnyhowError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

impl std::error::Error for AnyhowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

#[cfg(feature = "anyhow")]
impl TracedUnion {
    // todo improve this: Change to at display time and debug time instead of here
    #[cfg_attr(feature = "location", track_caller)]
    pub fn anyhow(error: anyhow::Error) -> TracedUnion {
        TracedUnion::new_from_parts(
            AnyhowError(error),
            #[cfg(feature = "backtrace")]
            std::backtrace::Backtrace::disabled(),
            #[cfg(feature = "context")]
            Vec::new(),
            #[cfg(feature = "location")]
            std::panic::Location::caller(),
        )
    }
}

// Needs specialization
// impl From<anyhow::Error> for TracedUnion {
//     fn from(value: anyhow::Error) -> Self {
//         TracedUnion::anyhow(value)
//     }
// }


#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    // ── helpers ──────────────────────────────────────────────────────────────

    #[derive(Debug, PartialEq)]
    struct FooError(String);

    impl fmt::Display for FooError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "FooError({})", self.0)
        }
    }
    impl std::error::Error for FooError {}

    #[derive(Debug, PartialEq)]
    struct BarError(u32);

    impl fmt::Display for BarError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "BarError({})", self.0)
        }
    }
    impl std::error::Error for BarError {}

    #[test]
    fn downcast_error_unchecked_correct_type_recovers_value() {
        let inner = TracedUnionInner::new(FooError("hello".into()));
        assert!(inner.is_error::<FooError>());
        let recovered: FooError = unsafe { inner.downcast_error_unchecked() };
        assert_eq!(recovered, FooError("hello".into()));
    }

    #[test]
    fn downcast_error_unchecked_does_not_leak_or_double_drop() {
        let payload = vec![1u8, 2, 3, 4, 5];

        #[derive(Debug, PartialEq)]
        struct VecError(Vec<u8>);
        impl fmt::Display for VecError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }
        impl std::error::Error for VecError {}

        let inner = TracedUnionInner::new(VecError(payload.clone()));
        let recovered: VecError = unsafe { inner.downcast_error_unchecked() };
        assert_eq!(recovered.0, payload);
    }

    #[test]
    #[should_panic]
    fn downcast_error_panics_on_wrong_type() {
        let inner = TracedUnionInner::new(FooError("oops".into()));
        inner.downcast_error::<BarError>(); // safe wrapper should panic
    }

    #[test]
    fn downcast_error_unchecked_with_parts_preserves_context() {
        let inner = TracedUnionInner::new(FooError("ctx".into()));

        let mut union: TracedUnion<(FooError,)> = TracedUnion {
            inner,
            _pd: PhantomData,
        };

        #[cfg(feature = "context")]
        {
            union.inner.context.push("step one".into());
            union.inner.context.push("step two".into());
        }

        let parts: TracedUnionInner<FooError> =
            unsafe { union.inner.downcast_error_unchecked_with_parts() };

        assert_eq!(parts.error, FooError("ctx".into()));

        #[cfg(feature = "context")]
        assert_eq!(parts.context.len(), 2);
    }

    #[test]
    fn downcast_error_unchecked_with_parts_correct_error_value() {
        let inner = TracedUnionInner::new(BarError(42));
        let parts: TracedUnionInner<BarError> =
            unsafe { inner.downcast_error_unchecked_with_parts() };
        assert_eq!(parts.error, BarError(42));
    }

    #[test]
    fn into_dyn_error_and_back_roundtrips() {
        let union: TracedUnion<(FooError,)> = TracedUnion::new(FooError("roundtrip".into()));
        let dyn_err: Box<dyn SendSyncError> = union.into_dyn_error();

        let recovered: TracedUnion<(FooError,)> =
            TracedUnion::from_dyn_error(dyn_err).expect("round-trip should succeed");

        assert_eq!(recovered.inner(), &FooError("roundtrip".into()));
    }

    #[test]
    fn from_dyn_error_wrong_type_returns_err() {
        let union: TracedUnion<(FooError,)> = TracedUnion::new(FooError("mismatch".into()));
        let dyn_err: Box<dyn SendSyncError> = union.into_dyn_error();

        let result: Result<TracedUnion<(BarError,)>, _> = TracedUnion::from_dyn_error(dyn_err);
        assert!(result.is_err(), "mismatched type should be returned as Err");
    }

    #[test]
    fn into_dyn_error_display_delegates_to_inner() {
        let union: TracedUnion<(FooError,)> = TracedUnion::new(FooError("display".into()));
        let dyn_err = union.into_dyn_error();
        assert!(dyn_err.to_string().contains("FooError(display)"));
    }

    #[test]
    fn into_dyn_error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>(_: T) {}
        let union: TracedUnion<(FooError,)> = TracedUnion::new(FooError("traits".into()));
        assert_send_sync(union.into_dyn_error());
    }

    #[test]
    fn from_dyn_error_preserves_context() {
        let mut union: TracedUnion<(FooError,)> = TracedUnion::new(FooError("ctx".into()));
        #[cfg(feature = "context")]
        {
            union = union.context("some context");
        }
        let dyn_err = union.into_dyn_error();
        let recovered: TracedUnion<(FooError,)> =
            TracedUnion::from_dyn_error(dyn_err).unwrap();

        #[cfg(feature = "context")]
        assert_eq!(recovered.inner.context.len(), 1);

        assert_eq!(recovered.inner(), &FooError("ctx".into()));
    }

    #[test]
    fn multi_variant_union_into_dyn_error_roundtrips() {
        let union: TracedUnion<(FooError, BarError)> =
            TracedUnion::new(BarError(99));
        let dyn_err = union.into_dyn_error();

        let recovered: TracedUnion<(FooError, BarError)> =
            TracedUnion::from_dyn_error(dyn_err).unwrap();

        let bar: BarError = recovered.narrow().unwrap();
        assert_eq!(bar, BarError(99));
    }
}