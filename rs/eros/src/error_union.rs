use core::any::Any;
use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;
use std::any::TypeId;
use std::{mem, ptr};

#[cfg(feature = "context")]
use crate::context::ErosContext;
use crate::type_set::{
    Contains, DebugFold, DisplayFold, ErrorFold, IsFold, Narrow, SupersetOf, TupleForm, TypeSet,
    write_debug, write_display,
};

use crate::{AnyError, Cons, End, StrError};

/// Any error that satisfies this trait's bounds can be used in a `ErrorUnion`
pub trait SendSyncError: std::any::Any + std::error::Error + Send + Sync + 'static {
    /// Converts this `SendSynError` to `Any`.
    ///
    /// Warning: Use carefully since `Any` is this type,
    /// not the underlying type. e.g. For `x: Box<dyn SendSyncError>` with `x.as_any()`,
    /// the type yielded is `Box<dyn SendSyncError>` so functions like `x.is::<T>()` will not work as expected.
    /// While one would probably want `(&*x as &dyn Any)` which is the underlying and `x.is::<T>()`
    /// will work as expected.
    ///
    /// The main use case for this function is when `x: &dyn SendSyncError`.
    /// In such case `x.is::<T>()` will likely work as intended.
    fn as_any(&self) -> &dyn Any;
}

impl<T> SendSyncError for T
where
    T: std::error::Error + Send + Sync + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::error::Error for Box<dyn SendSyncError> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&**self)
    }
}

pub(crate) struct ErrorUnionInner<T: ?Sized> {
    #[cfg(feature = "backtrace")]
    pub(crate) backtrace: std::backtrace::Backtrace,
    #[cfg(feature = "context")]
    pub(crate) context: Vec<ErosContext>,
    #[cfg(feature = "location")]
    pub(crate) location: &'static std::panic::Location<'static>,
    /// Re-boxes the error field into a fresh allocation.
    /// Stored at construction so the concrete type is still known.
    pub(crate) into_box_fn: fn(*mut dyn SendSyncError) -> Box<dyn SendSyncError>,
    pub(crate) error: T,
}

fn make_box<T: SendSyncError>(ptr: *mut dyn SendSyncError) -> Box<dyn SendSyncError> {
    // SAFETY: caller guarantees ptr points to a live T
    let value: T = unsafe { ptr::read(ptr as *const dyn SendSyncError as *const T) };
    Box::new(value)
}

impl ErrorUnionInner<dyn SendSyncError> {
    #[cfg_attr(feature = "location", track_caller)]
    pub(crate) fn new<T>(t: T) -> Box<ErrorUnionInner<dyn SendSyncError>>
    where
        T: SendSyncError,
    {
        Box::new(ErrorUnionInner {
            #[cfg(feature = "backtrace")]
            backtrace: std::backtrace::Backtrace::capture(),
            #[cfg(feature = "context")]
            context: Vec::new(),
            #[cfg(feature = "location")]
            location: std::panic::Location::caller(),
            into_box_fn: make_box::<T>,
            error: t,
        })
    }

    pub(crate) fn new_from_parts<T>(
        t: T,
        #[cfg(feature = "backtrace")] backtrace: std::backtrace::Backtrace,
        #[cfg(feature = "context")] context: Vec<ErosContext>,
        #[cfg(feature = "location")] location: &'static std::panic::Location<'static>,
    ) -> Box<ErrorUnionInner<dyn SendSyncError>>
    where
        T: SendSyncError,
    {
        Box::new(ErrorUnionInner {
            #[cfg(feature = "backtrace")]
            backtrace,
            #[cfg(feature = "context")]
            context,
            #[cfg(feature = "location")]
            location,
            into_box_fn: make_box::<T>,
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
    ) -> ErrorUnionInner<T> {
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

        let into_box_fn = ptr::read(ptr::addr_of!((*raw_container).into_box_fn));

        // Deallocate the Box allocation itself.
        // We reconstruct a Box containing uninitialized/dead data, but wrapped in
        // ManuallyDrop so its fields aren't dropped. When this `dead_box` goes out of scope,
        // it frees the underlying heap memory without touching the fields.
        let _dead_box: Box<mem::ManuallyDrop<Self>> =
            Box::from_raw(raw_container as *mut mem::ManuallyDrop<Self>);

        ErrorUnionInner {
            #[cfg(feature = "backtrace")]
            backtrace,
            #[cfg(feature = "context")]
            context,
            #[cfg(feature = "location")]
            location,
            into_box_fn,
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

/// `ErrorUnion` is an open sum type of errors. It differs from an enum
/// in that you do not need to define any actual new type
/// in order to hold some specific combination of variants,
/// but rather you simply describe the ErrorUnion as holding
/// one value out of several specific possibilities,
/// defined by using a tuple of those possible variants
/// as the generic parameter for the `ErrorUnion`.
///
/// For example, a `ErrorUnion<(io::Error, fmt::Error)>` contains either
/// a `io::Error` or a `fmt::Error`. The benefit of this over creating
/// specific enums for each function become apparent in larger
/// codebases where error handling needs to occur in
/// different places for different errors. As such, `ErrorUnion` allows
/// you to quickly specify a function's return value as
/// involving a precise subset of errors that the caller
/// can clearly reason about. Providing maximum composability with
/// no boilerplate.
///
/// When the exact error type does not matter, `ErrorUnion<AnyError>` represents
/// the set of all possible errors.
///
/// `ErrorUnion` also holds information surrounding the error depending on feature
/// flags enabled. This may include `Backtrace` and/or `Location`. Context can be added throughout
/// the call stack.
pub struct ErrorUnion<E: TypeSet = AnyError> {
    pub(crate) inner: Box<ErrorUnionInner<dyn SendSyncError>>,
    pub(crate) _pd: PhantomData<E>,
}

impl<T> Deref for ErrorUnion<(T,)>
where
    T: 'static,
{
    type Target = T;

    fn deref(&self) -> &T {
        (&self.inner.error as &dyn Any).downcast_ref::<T>().unwrap()
    }
}

impl fmt::Debug for ErrorUnion<AnyError> {
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

impl<E> fmt::Debug for ErrorUnion<E>
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

impl fmt::Display for ErrorUnion<AnyError> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_display(&self.inner.error, formatter)
    }
}

impl<E> fmt::Display for ErrorUnion<E>
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

    let error_union: ErrorUnion<(io::Error, fmt::Error)> =
        ErrorUnion::new(io::Error::other("yooo"));
    is_send(&error_union);
    is_sync(&error_union);
    // is_error(&error_union); //todo
}

unsafe impl<T> Send for ErrorUnion<T> where T: TypeSet + Send {}
unsafe impl<T> Sync for ErrorUnion<T> where T: TypeSet + Sync {}

//************************************************************************//

impl ErrorUnion {
    /// Create a new `ErrorUnion`.
    #[cfg_attr(feature = "location", track_caller)]
    pub fn new<T, OutSet, Index>(t: T) -> ErrorUnion<OutSet>
    where
        T: SendSyncError,
        OutSet: TypeSet,
        OutSet::Variants: Contains<T, Index>,
    {
        ErrorUnion {
            inner: ErrorUnionInner::new(t),
            _pd: PhantomData,
        }
    }

    pub fn new_from_parts<T, OutSet, Index>(
        t: T,
        #[cfg(feature = "backtrace")] backtrace: std::backtrace::Backtrace,
        #[cfg(feature = "context")] context: Vec<ErosContext>,
        #[cfg(feature = "location")] location: &'static std::panic::Location<'static>,
    ) -> ErrorUnion<OutSet>
    where
        T: SendSyncError,
        OutSet: TypeSet,
        OutSet::Variants: Contains<T, Index>,
    {
        ErrorUnion {
            inner: ErrorUnionInner::new_from_parts(
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

    pub(crate) fn erase<E>(t: ErrorUnion<E>) -> ErrorUnion
    where
        E: TypeSet,
    {
        ErrorUnion {
            inner: t.inner,
            _pd: PhantomData,
        }
    }
}

//************************************************************************//

struct ErrorUnionErrorWrapper<E>(ErrorUnion<E>)
where
    E: TypeSet;

impl<E> std::error::Error for ErrorUnionErrorWrapper<E>
where
    E: TypeSet,
    E::Variants: std::error::Error + DebugFold + DisplayFold + ErrorFold,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        E::Variants::source_fold(&self.0.inner.error as &dyn Any)
    }
}

impl<E> fmt::Display for ErrorUnionErrorWrapper<E>
where
    E: TypeSet,
    E::Variants: fmt::Display + DisplayFold,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

impl<E> fmt::Debug for ErrorUnionErrorWrapper<E>
where
    E: TypeSet,
    E::Variants: fmt::Debug + DebugFold,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

impl<E> ErrorUnion<E>
where
    E: TypeSet + Send + Sync + 'static,
    E::Variants: std::error::Error + DebugFold + DisplayFold + ErrorFold,
{
    /// Creates a `Box<dyn SendSyncError>` error from this [`crate::ErrorUnion`]. This is used since
    /// [`crate::ErrorUnion`] cannot implement [`std::error::Error`] directly, otherwise trait implementations
    /// that require this bounds would conflict. To convert back into a [`crate::ErrorUnion`],
    /// [`crate::ErrorUnion::from_dyn_error`] must be used.
    pub fn into_dyn_error(self) -> Box<dyn SendSyncError> {
        Box::new(ErrorUnionErrorWrapper(self)) as Box<dyn SendSyncError>
    }

    /// See [`crate::ErrorUnion::into_dyn_error`].
    pub fn from_dyn_error(error: Box<dyn SendSyncError>) -> Result<Self, Box<dyn SendSyncError>> {
        let error_ref = &*error as &dyn Any;
        if !error_ref.is::<ErrorUnionErrorWrapper<E>>() {
            return Err(error);
        }
        let error = error as Box<dyn Any>;
        let error_union_wrapper = error.downcast::<ErrorUnionErrorWrapper<E>>().unwrap();
        Ok(error_union_wrapper.0)
    }
}

//************************************************************************//

impl<E> ErrorUnion<E>
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
        ErrorUnion<<<E::Variants as Narrow<Target, Index>>::Remainder as TupleForm>::Tuple>,
    >
    where
        Target: 'static,
        E::Variants: Narrow<Target, Index>,
    {
        if self.inner.is_error::<Target>() {
            Ok(unsafe { self.inner.downcast_error_unchecked::<Target>() })
        } else {
            Err(ErrorUnion {
                inner: self.inner,
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
        ErrorUnion<TargetList>,
        ErrorUnion<<<E::Variants as SupersetOf<TargetList::Variants, Index>>::Remainder as TupleForm>::Tuple>,
    >
    where
        TargetList: TypeSet,
        E::Variants: IsFold + SupersetOf<TargetList::Variants, Index>,
    {
        if E::Variants::is_fold(&self.inner.error as &dyn Any) {
            Ok(ErrorUnion {
                inner: self.inner,
                _pd: PhantomData,
            })
        } else {
            Err(ErrorUnion {
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

    /// Gets a reference to the inner underlying error
    pub fn inner_ref(&self) -> &dyn SendSyncError {
        &self.inner.error
    }

    /// Gets a reference to the inner underlying error as `Any`
    pub fn inner_ref_any(&self) -> &dyn Any {
        &self.inner.error
    }

    #[cfg(feature = "backtrace")]
    pub fn backtrace(&self) -> &std::backtrace::Backtrace {
        &self.inner.backtrace
    }

    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.error.source()
    }

    /// Into the inner underlying error
    pub fn into_inner_dyn_error(self) -> Box<dyn SendSyncError> {
        let raw = Box::into_raw(self.inner);
        unsafe {
            let into_box_fn = (*raw).into_box_fn;
            let error_ptr = ptr::addr_of_mut!((*raw).error) as *mut dyn SendSyncError;

            let boxed = (into_box_fn)(error_ptr);

            // Drop remaining fields, free the allocation (same pattern as downcast_error_unchecked)
            #[cfg(feature = "backtrace")]
            ptr::drop_in_place(ptr::addr_of_mut!((*raw).backtrace));
            #[cfg(feature = "context")]
            ptr::drop_in_place(ptr::addr_of_mut!((*raw).context));
            #[cfg(feature = "location")]
            ptr::drop_in_place(ptr::addr_of_mut!((*raw).location));

            let _dead: Box<mem::ManuallyDrop<ErrorUnionInner<dyn SendSyncError>>> =
                Box::from_raw(raw as *mut _);

            boxed
        }
    }

    #[cfg(feature = "logging")]
    pub fn log_error(&self) {
        #[cfg(all(feature = "log_display", not(feature = "log_debug"), feature = "tracing"))]
        tracing::error!("{}", self);
        #[cfg(all(feature = "log_debug", feature = "tracing"))]
        tracing::error!("{:#?}", self);
    }

    #[cfg(feature = "logging")]
    pub fn log_warn(&self) {
        #[cfg(all(feature = "log_display", not(feature = "log_debug"), feature = "tracing"))]
        tracing::warn!("{}", self);
        #[cfg(all(feature = "log_debug", feature = "tracing"))]
        tracing::warn!("{:#?}", self);
    }

    #[cfg(feature = "logging")]
    pub fn log_info(&self) {
        #[cfg(all(feature = "log_display", not(feature = "log_debug"), feature = "tracing"))]
        tracing::info!("{}", self);
        #[cfg(all(feature = "log_debug", feature = "tracing"))]
        tracing::info!("{:#?}", self);
    }

    #[cfg(feature = "logging")]
    pub fn log_debug(&self) {
        #[cfg(all(feature = "log_display", not(feature = "log_debug"), feature = "tracing"))]
        tracing::debug!("{}", self);
        #[cfg(all(feature = "log_debug", feature = "tracing"))]
        tracing::debug!("{:#?}", self);
    }

    #[cfg(feature = "logging")]
    pub fn log_trace(&self) {
        #[cfg(all(feature = "log_display", not(feature = "log_debug"), feature = "tracing"))]
        tracing::trace!("{}", self);
        #[cfg(all(feature = "log_debug", feature = "tracing"))]
        tracing::trace!("{:#?}", self);
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
    #[cfg_attr(feature = "location", track_caller)]
    pub fn context<C: Into<StrError>>(mut self, context: C) -> Self {
        #[cfg(feature = "context")]
        self.inner
            .context
            .push(crate::context::ErosContext::new(context.into()));
        self
    }

    /// Adds additional context lazily. This becomes a no-op if the `traced` feature is disabled.
    #[allow(unused_mut)]
    #[allow(unused_variables)]
    #[cfg_attr(feature = "location", track_caller)]
    pub fn with_context<F, C: Into<StrError>>(mut self, f: F) -> Self
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "context")]
        self.inner
            .context
            .push(crate::context::ErosContext::new(f().into()));
        self
    }
}

impl<A: 'static> ErrorUnion<(A,)> {
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

    pub fn map<U, F>(self, f: F) -> ErrorUnion<(U,)>
    where
        U: SendSyncError,
        F: FnOnce(A) -> U,
    {
        // SAFETY: We know that the inner error is only of type A, so we can safely downcast it
        let inner = unsafe { self.inner.downcast_error_unchecked_with_parts::<A>() };
        ErrorUnion {
            inner: ErrorUnionInner::new_from_parts(
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
    fn widen<Other, Index>(self) -> Result<S, ErrorUnion<Other>>
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
            ErrorUnion<<<E::Variants as Narrow<Target, Index>>::Remainder as TupleForm>::Tuple>,
        >,
    >
    where
        Target: 'static,
        E::Variants: Narrow<Target, Index>;
}

impl<S, E> ReshapeUnion<S, E> for Result<S, ErrorUnion<E>>
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
                Ok(value) => Ok(value),
                Err(err) => Err(Err(err)),
            },
        }
    }
}

//************************************************************************//

pub trait IntoUnion<S, F> {
    /// Creates an `ErrorUnion` for this type.
    fn into_union<Index, Other>(self) -> Result<S, ErrorUnion<Other>>
    where
        Other: TypeSet,
        Other::Variants: Contains<F, Index>;
}

impl<S, F: SendSyncError> IntoUnion<S, F> for Result<S, F> {
    #[cfg_attr(feature = "location", track_caller)]
    fn into_union<Index, Other>(self) -> Result<S, ErrorUnion<Other>>
    where
        Other: TypeSet,
        Other::Variants: Contains<F, Index>,
    {
        // Note: We use match so the call location gets passed through
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(ErrorUnion::new(err)),
        }
    }
}

pub trait IntoDynUnion<S> {
    /// Creates an `ErrorUnion` for this type.
    fn into_dyn_union(self) -> Result<S, ErrorUnion>;
}

impl<S, F: SendSyncError> IntoDynUnion<S> for Result<S, F> {
    #[cfg_attr(feature = "location", track_caller)]
    fn into_dyn_union(self) -> Result<S, ErrorUnion> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(ErrorUnion::new(err)),
        }
    }
}

impl<S, E: TypeSet> IntoDynUnion<S> for Result<S, ErrorUnion<E>> {
    fn into_dyn_union(self) -> Result<S, ErrorUnion> {
        self.map_err(|e| ErrorUnion::erase(e))
    }
}

// pub trait IntoUnion<S, F> {
//     /// Con `Err` to i
//     fn into_union<Index, Other>(self) -> Result<S, ErrorUnion<Other>>
//     where
//         Other: TypeSet,
//         Other::Variants: Contains<F, Index>;
// }

// impl<S, F1, F2> IntoUnion<S, F2> for Result<S, F1>
// where
//     F1: Into<F2> + SendSyncError, // `SendSyncError` is used to ensure it does not overlap with below
//     F2: SendSyncError,
// {
//     fn into_union<Index, Other>(self) -> Result<S, ErrorUnion<Other>>
//     where
//         Other: TypeSet,
//         Other::Variants: Contains<F2, Index>,
//     {
//         self.map_err(|e| ErrorUnion::new(e.into()))
//     }
// }

// pub trait InnerInto<S, F> {
//     /// Converts the inner type of an `ErrorUnion` into another type
//     fn inner_into(self) -> Result<S, ErrorUnion<(F,)>>;
// }

// impl<S, F1, F2> InnerInto<S, F2> for Result<S, ErrorUnion<(F1,)>>
// where
//     F1: Into<F2> + SendSyncError,
//     F2: SendSyncError,
// {
//     fn inner_into(self) -> Result<S, ErrorUnion<(F2,)>> {
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
#[cfg(feature = "anyhow")]
#[derive(Debug)]
pub(crate) struct AnyhowError(pub(crate) anyhow::Error);

#[cfg(feature = "anyhow")]
impl fmt::Display for AnyhowError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

#[cfg(feature = "anyhow")]
impl std::error::Error for AnyhowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

#[cfg(feature = "anyhow")]
#[derive(Debug)]
pub(crate) struct AnyhowErrorArc(pub(crate) std::sync::Arc<anyhow::Error>);

#[cfg(feature = "anyhow")]
impl fmt::Display for AnyhowErrorArc {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

#[cfg(feature = "anyhow")]
impl std::error::Error for AnyhowErrorArc {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

#[cfg(feature = "anyhow")]
impl ErrorUnion {
    #[cfg_attr(feature = "location", track_caller)]
    pub fn anyhow(error: anyhow::Error) -> ErrorUnion {
        ErrorUnion::new_from_parts(
            AnyhowError(error),
            #[cfg(feature = "backtrace")]
            std::backtrace::Backtrace::disabled(),
            #[cfg(feature = "context")]
            Vec::new(),
            #[cfg(feature = "location")]
            std::panic::Location::caller(),
        )
    }

    #[cfg_attr(feature = "location", track_caller)]
    pub fn anyhow_arc(error: std::sync::Arc<anyhow::Error>) -> ErrorUnion {
        ErrorUnion::new_from_parts(
            AnyhowErrorArc(error),
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
// impl From<anyhow::Error> for ErrorUnion {
//     fn from(value: anyhow::Error) -> Self {
//         ErrorUnion::anyhow(value)
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
        let inner = ErrorUnionInner::new(FooError("hello".into()));
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

        let inner = ErrorUnionInner::new(VecError(payload.clone()));
        let recovered: VecError = unsafe { inner.downcast_error_unchecked() };
        assert_eq!(recovered.0, payload);
    }

    #[test]
    #[should_panic]
    fn downcast_error_panics_on_wrong_type() {
        let inner = ErrorUnionInner::new(FooError("oops".into()));
        inner.downcast_error::<BarError>(); // safe wrapper should panic
    }

    #[test]
    fn downcast_error_unchecked_with_parts_preserves_context() {
        let inner = ErrorUnionInner::new(FooError("ctx".into()));

        let mut union: ErrorUnion<(FooError,)> = ErrorUnion {
            inner,
            _pd: PhantomData,
        };

        #[cfg(feature = "context")]
        {
            union
                .inner
                .context
                .push(ErosContext::new("step one".into()));
            union
                .inner
                .context
                .push(ErosContext::new("step two".into()));
        }

        let parts: ErrorUnionInner<FooError> =
            unsafe { union.inner.downcast_error_unchecked_with_parts() };

        assert_eq!(parts.error, FooError("ctx".into()));

        #[cfg(feature = "context")]
        assert_eq!(parts.context.len(), 2);
    }

    #[test]
    fn downcast_error_unchecked_with_parts_correct_error_value() {
        let inner = ErrorUnionInner::new(BarError(42));
        let parts: ErrorUnionInner<BarError> =
            unsafe { inner.downcast_error_unchecked_with_parts() };
        assert_eq!(parts.error, BarError(42));
    }

    #[test]
    fn into_dyn_error_and_back_roundtrips() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("roundtrip".into()));
        let dyn_err: Box<dyn SendSyncError> = union.into_dyn_error();
        assert!((&*dyn_err as &dyn Any).is::<ErrorUnionErrorWrapper<(FooError,)>>());
        let recovered: ErrorUnion<(FooError,)> =
            ErrorUnion::from_dyn_error(dyn_err).expect("round-trip should succeed");

        assert_eq!(recovered.inner(), &FooError("roundtrip".into()));
    }

    #[test]
    fn from_dyn_error_wrong_type_returns_err() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("mismatch".into()));
        let dyn_err: Box<dyn SendSyncError> = union.into_dyn_error();

        let result: Result<ErrorUnion<(BarError,)>, _> = ErrorUnion::from_dyn_error(dyn_err);
        assert!(result.is_err(), "mismatched type should be returned as Err");
    }

    #[test]
    fn into_dyn_error_display_delegates_to_inner() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("display".into()));
        let dyn_err = union.into_dyn_error();
        assert!(dyn_err.to_string().contains("FooError(display)"));
    }

    #[test]
    fn into_dyn_error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>(_: T) {}
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("traits".into()));
        assert_send_sync(union.into_dyn_error());
    }

    #[test]
    fn from_dyn_error_preserves_context() {
        let mut union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("ctx".into()));
        #[cfg(feature = "context")]
        {
            union = union.context("some context");
        }
        let dyn_err = union.into_dyn_error();
        let recovered: ErrorUnion<(FooError,)> = ErrorUnion::from_dyn_error(dyn_err).unwrap();

        #[cfg(feature = "context")]
        assert_eq!(recovered.inner.context.len(), 1);

        assert_eq!(recovered.inner(), &FooError("ctx".into()));
    }

    #[test]
    fn multi_variant_union_into_dyn_error_roundtrips() {
        let union: ErrorUnion<(FooError, BarError)> = ErrorUnion::new(BarError(99));
        let dyn_err = union.into_dyn_error();

        let recovered: ErrorUnion<(FooError, BarError)> =
            ErrorUnion::from_dyn_error(dyn_err).unwrap();

        let bar: BarError = recovered.narrow().unwrap();
        assert_eq!(bar, BarError(99));
    }

    #[test]
    fn into_inner_dyn_error_returns_concrete_type_not_wrapper() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("concrete".into()));
        let dyn_err = union.into_inner_dyn_error();

        assert!(
            (&*dyn_err as &dyn Any).is::<FooError>(),
            "expected FooError, got a wrapper or wrong type"
        );
    }

    #[test]
    fn into_inner_dyn_error_value_is_preserved() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("preserved".into()));
        let dyn_err = union.into_inner_dyn_error();

        let foo = (&*dyn_err as &dyn Any).downcast_ref::<FooError>().unwrap();
        assert_eq!(foo, &FooError("preserved".into()));
    }

    #[test]
    fn into_inner_dyn_error_display_is_concrete_type() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("display".into()));
        let dyn_err = union.into_inner_dyn_error();

        assert_eq!(dyn_err.to_string(), "FooError(display)");
    }

    #[test]
    fn into_inner_dyn_error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>(_: T) {}
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("traits".into()));
        assert_send_sync(union.into_inner_dyn_error());
    }

    #[test]
    fn into_inner_dyn_error_differs_from_into_dyn_error() {
        let union_a: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("a".into()));
        let union_b: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("b".into()));

        let inner_dyn = union_a.into_inner_dyn_error();
        let wrapper_dyn = union_b.into_dyn_error();

        assert!((&*inner_dyn as &dyn Any).is::<FooError>());
        assert!(!(&*wrapper_dyn as &dyn Any).is::<FooError>());
        assert!((&*wrapper_dyn as &dyn Any).is::<ErrorUnionErrorWrapper<(FooError,)>>());
    }

    #[test]
    fn into_inner_dyn_error_multi_variant_foo() {
        let union: ErrorUnion<(FooError, BarError)> = ErrorUnion::new(FooError("multi".into()));
        let dyn_err = union.into_inner_dyn_error();

        assert!((&*dyn_err as &dyn Any).is::<FooError>());
        assert!(!(&*dyn_err as &dyn Any).is::<BarError>());
        let foo = (&*dyn_err as &dyn Any).downcast_ref::<FooError>().unwrap();
        assert_eq!(foo, &FooError("multi".into()));
    }

    #[test]
    fn into_inner_dyn_error_multi_variant_bar() {
        let union: ErrorUnion<(FooError, BarError)> = ErrorUnion::new(BarError(77));
        let dyn_err = union.into_inner_dyn_error();

        assert!((&*dyn_err as &dyn Any).is::<BarError>());
        assert!(!(&*dyn_err as &dyn Any).is::<FooError>());
        let bar = (&*dyn_err as &dyn Any).downcast_ref::<BarError>().unwrap();
        assert_eq!(bar, &BarError(77));
    }

    #[test]
    fn into_inner_dyn_error_does_not_leak_heap_allocation() {
        // Uses a Vec payload so Miri / address-sanitizer can catch leaks or double-drops.
        #[derive(Debug, PartialEq)]
        struct VecError(Vec<u8>);
        impl fmt::Display for VecError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }
        impl std::error::Error for VecError {}

        let payload = vec![1u8, 2, 3, 4, 5];
        let union: ErrorUnion<(VecError,)> = ErrorUnion::new(VecError(payload.clone()));
        let dyn_err = union.into_inner_dyn_error();

        let recovered = (&*dyn_err as &dyn Any).downcast_ref::<VecError>().unwrap();
        assert_eq!(recovered.0, payload);
        // `dyn_err` drops here — Miri will catch any double-free or leak.
    }

    #[test]
    fn into_inner_dyn_error_not_roundtrippable_via_from_dyn_error() {
        // Confirm that from_dyn_error correctly rejects a bare inner error
        // (since it's not wrapped in ErrorUnionErrorWrapper).
        let union_a: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("bare".into()));
        let bare_dyn = union_a.into_inner_dyn_error();

        let result: Result<ErrorUnion<(FooError,)>, _> = ErrorUnion::from_dyn_error(bare_dyn);
        assert!(
            result.is_err(),
            "from_dyn_error should reject a bare inner error, not an ErrorUnionErrorWrapper"
        );
    }
}
