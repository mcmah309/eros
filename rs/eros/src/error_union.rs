use alloc::boxed::Box;
#[cfg(feature = "context")]
use alloc::vec::Vec;
use core::any::Any;
#[cfg(not(feature = "std"))]
use core::any::TypeId;
use core::fmt;
use core::marker::PhantomData;
use core::mem;
use core::ops::Deref;
use core::ptr;
#[cfg(feature = "clone")]
use core::sync::atomic::{AtomicUsize, Ordering};
#[cfg(feature = "std")]
use std::any::TypeId;

use crate::context::ContextSource;
#[cfg(feature = "context")]
use crate::context::ErosContext;
use crate::type_set::{
    Contains, DebugFold, DisplayFold, ErrorFold, IsFold, Narrow, SupersetOf, TupleForm, TypeSet,
    write_debug, write_display,
};

use crate::{AnyError, Cons, End};

/// Any error that satisfies this trait's bounds can be used in a `ErrorUnion`
pub trait SendSyncError: core::any::Any + core::error::Error + Send + Sync + 'static {
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
    T: core::error::Error + core::any::Any + Send + Sync + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl core::error::Error for Box<dyn SendSyncError> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&**self)
    }
}

pub(crate) struct ErrorUnionInner<T: ?Sized> {
    #[cfg(feature = "backtrace")]
    pub(crate) backtrace: std::backtrace::Backtrace,
    #[cfg(feature = "context")]
    pub(crate) context: Vec<ErosContext>,
    #[cfg(feature = "location")]
    pub(crate) location: &'static core::panic::Location<'static>,
    /// Number of live `ErrorUnion` handles that currently share this allocation.
    /// Only present when the `clone` feature is enabled, in which case
    /// `ErrorUnion` behaves like a reference-counted (`Arc`-like) pointer instead
    /// of a uniquely-owned `Box`. Any operation that needs to move the `error`
    /// field out (and free this allocation) must first confirm the count is `1`,
    /// since doing so while other handles exist would leave them dangling.
    #[cfg(feature = "clone")]
    pub(crate) count: AtomicUsize,
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
    /// Allocates a new `ErrorUnionInner` and returns a raw, non-null pointer to it.
    #[cfg_attr(feature = "location", track_caller)]
    pub(crate) fn new<T>(t: T) -> ptr::NonNull<ErrorUnionInner<dyn SendSyncError>>
    where
        T: SendSyncError,
    {
        let boxed: Box<ErrorUnionInner<dyn SendSyncError>> = Box::new(ErrorUnionInner {
            #[cfg(feature = "backtrace")]
            backtrace: std::backtrace::Backtrace::capture(),
            #[cfg(feature = "context")]
            context: Vec::new(),
            #[cfg(feature = "location")]
            location: core::panic::Location::caller(),
            #[cfg(feature = "clone")]
            count: AtomicUsize::new(1),
            into_box_fn: make_box::<T>,
            error: t,
        });
        // SAFETY: Box::into_raw never returns a null pointer.
        unsafe { ptr::NonNull::new_unchecked(Box::into_raw(boxed)) }
    }

    pub(crate) fn new_from_parts<T>(
        t: T,
        #[cfg(feature = "backtrace")] backtrace: std::backtrace::Backtrace,
        #[cfg(feature = "context")] context: Vec<ErosContext>,
        #[cfg(feature = "location")] location: &'static core::panic::Location<'static>,
    ) -> ptr::NonNull<ErrorUnionInner<dyn SendSyncError>>
    where
        T: SendSyncError,
    {
        let boxed: Box<ErrorUnionInner<dyn SendSyncError>> = Box::new(ErrorUnionInner {
            #[cfg(feature = "backtrace")]
            backtrace,
            #[cfg(feature = "context")]
            context,
            #[cfg(feature = "location")]
            location,
            #[cfg(feature = "clone")]
            count: AtomicUsize::new(1),
            into_box_fn: make_box::<T>,
            error: t,
        });
        // SAFETY: Box::into_raw never returns a null pointer.
        unsafe { ptr::NonNull::new_unchecked(Box::into_raw(boxed)) }
    }

    #[allow(unstable_name_collisions)]
    pub(crate) fn is_error<T: 'static>(&self) -> bool {
        self.error.type_id() == TypeId::of::<T>()
    }

    /// Panics unless this allocation is uniquely owned (ref count of `1`).
    /// A no-op when the `clone` feature is disabled, since in that case
    /// `ErrorUnion` is always uniquely owned by construction.
    #[allow(dead_code)]
    #[inline]
    fn assert_unique(&self) {
        #[cfg(feature = "clone")]
        {
            let count = self.count.load(Ordering::Acquire);
            assert_eq!(
                count, 1,
                "cannot downcast an ErrorUnion's inner error while {count} clone(s) of it exist"
            );
        }
    }

    pub(crate) unsafe fn downcast_error_unchecked<T: 'static>(self: Box<Self>) -> T {
        debug_assert!(self.is_error::<T>());
        // Taking the error out of this allocation destroys the allocation itself,
        // which would leave any other clones dangling, so refuse unless we're the
        // sole owner.
        self.assert_unique();

        // Note: this prevents the Box from automatically dropping at the end of the function.
        let raw_container: *mut Self = Box::into_raw(self);

        unsafe {
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
    }

    pub(crate) unsafe fn downcast_error_unchecked_with_parts<T: 'static>(
        self: Box<Self>,
    ) -> ErrorUnionInner<T> {
        debug_assert!(self.is_error::<T>());
        // Taking the error out of this allocation destroys the allocation itself,
        // which would leave any other clones dangling, so refuse unless we're the
        // sole owner.
        self.assert_unique();

        // Note: this prevents the Box from automatically dropping at the end of the function.
        let raw_container: *mut Self = Box::into_raw(self);

        unsafe {
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
                // This is a brand new, uniquely-owned allocation once it gets
                // re-boxed by the caller (see `ErrorUnion::map`), so it starts
                // fresh at a count of one.
                #[cfg(feature = "clone")]
                count: AtomicUsize::new(1),
                into_box_fn,
                error: downcasted_value,
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn downcast_error<T: 'static>(self: Box<Self>) -> Option<T> {
        if self.is_error::<T>() {
            Some(unsafe { self.downcast_error_unchecked::<T>() })
        } else {
            None
        }
    }

    pub(crate) fn downcast_error_ref<T: 'static>(&self) -> Option<&T> {
        (&self.error as &dyn Any).downcast_ref::<T>()
    }

    // todo when https://github.com/rust-lang/rust/issues/90850 is stabilized
    // pub(crate) unsafe fn downcast_unchecked_error_ref<T: 'static>(&self) -> &T {
    //     debug_assert!(self.is_error::<T>());
    //     unsafe { (&self.error as &dyn Any).downcast_unchecked_ref::<T>() }
    // }

    pub(crate) fn downcast_error_mut<T: 'static>(&mut self) -> Option<&mut T> {
        (&mut self.error as &mut dyn Any).downcast_mut::<T>()
    }

    // todo when https://github.com/rust-lang/rust/issues/90850 is stabilized
    // pub(crate) unsafe fn downcast_unchecked_error_mut<T: 'static>(&mut self) -> &mut T {
    //     debug_assert!(self.is_error::<T>());
    //     unsafe { (&mut self.error as &mut dyn Any).downcast_unchecked_mut::<T>() }
    // }
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
///
/// When the `clone` feature is enabled, `ErrorUnion` is reference-counted internally
/// (similar to `Arc`) and implements [`Clone`]. Cloning is cheap: it bumps an atomic
/// counter rather than duplicating the underlying error. Because of this, operations
/// that need to take ownership of the underlying error (such as [`ErrorUnion::narrow`]
/// succeeding, [`ErrorUnion::take`], [`ErrorUnion::into_single`], [`ErrorUnion::into_inner`],
/// or [`ErrorUnion::map`]) will panic if any other clone of the same `ErrorUnion` is still
/// alive. Use [`ErrorUnion::ref_count`] or [`ErrorUnion::is_unique`] to check beforehand.
pub struct ErrorUnion<E: TypeSet = AnyError> {
    pub(crate) inner: ptr::NonNull<ErrorUnionInner<dyn SendSyncError>>,
    pub(crate) _pd: PhantomData<E>,
}

impl<T> Deref for ErrorUnion<(T,)>
where
    T: 'static,
{
    type Target = T;

    fn deref(&self) -> &T {
        (unsafe { &self.inner.as_ref().error } as &dyn Any)
            .downcast_ref::<T>()
            .unwrap()
    }
}

impl fmt::Debug for ErrorUnion<AnyError> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = unsafe { self.inner.as_ref() };
        write_debug(
            &inner.error,
            formatter,
            #[cfg(feature = "context")]
            &inner.context,
            #[cfg(feature = "backtrace")]
            &inner.backtrace,
            #[cfg(feature = "location")]
            inner.location,
        )
    }
}

impl<E> fmt::Debug for ErrorUnion<E>
where
    E: TypeSet,
    E::Variants: fmt::Debug + DebugFold,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = unsafe { self.inner.as_ref() };
        E::Variants::debug_fold(
            &inner.error,
            formatter,
            #[cfg(feature = "context")]
            &inner.context,
            #[cfg(feature = "backtrace")]
            &inner.backtrace,
            #[cfg(feature = "location")]
            inner.location,
        )?;
        Ok(())
    }
}

impl fmt::Display for ErrorUnion<AnyError> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_display(&unsafe { self.inner.as_ref() }.error, formatter)
    }
}

impl<E> fmt::Display for ErrorUnion<E>
where
    E: TypeSet,
    E::Variants: fmt::Display + DisplayFold,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        E::Variants::display_fold(&unsafe { self.inner.as_ref() }.error as &dyn Any, formatter)?;
        Ok(())
    }
}

//************************************************************************//

#[cfg(feature = "std")]
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

/// Drops the underlying allocation.
///
/// Without the `clone` feature, `ErrorUnion` is always the unique owner of its
/// allocation (mirroring the old `Box`-based representation), so this always frees it.
///
/// With the `clone` feature, `ErrorUnion` is reference-counted: dropping decrements
/// the shared counter, and the allocation is only actually freed once the last
/// handle is dropped.
impl<E> Drop for ErrorUnion<E>
where
    E: TypeSet,
{
    fn drop(&mut self) {
        #[cfg(feature = "clone")]
        {
            // SAFETY: `self.inner` is always a valid, live allocation for as long
            // as any `ErrorUnion` referencing it exists.
            let previous_count = unsafe { self.inner.as_ref() }
                .count
                .fetch_sub(1, Ordering::AcqRel);
            if previous_count != 1 {
                // Other clones are still alive; they remain responsible for the
                // allocation.
                return;
            }
        }

        // SAFETY: We are either the unique owner (no `clone` feature), or we just
        // observed the count drop to zero (with the `clone` feature), so it is
        // safe to reclaim the allocation.
        unsafe {
            drop(Box::from_raw(self.inner.as_ptr()));
        }
    }
}

/// Cloning an `ErrorUnion` is cheap: it simply bumps the shared reference count
/// rather than duplicating the underlying error. Operations that need to take
/// ownership of the underlying error will panic while more than one clone is alive
/// — see the type-level docs on [`ErrorUnion`] for details.
#[cfg(feature = "clone")]
impl<E> Clone for ErrorUnion<E>
where
    E: TypeSet,
{
    fn clone(&self) -> Self {
        // SAFETY: `self.inner` is always a valid, live allocation for as long
        // as this `ErrorUnion` exists.
        unsafe { self.inner.as_ref() }
            .count
            .fetch_add(1, Ordering::AcqRel);

        ErrorUnion {
            inner: self.inner,
            _pd: PhantomData,
        }
    }
}

#[cfg(feature = "clone")]
impl<E> ErrorUnion<E>
where
    E: TypeSet,
{
    /// Returns the number of `ErrorUnion` handles (including this one) that
    /// currently share the same underlying error allocation.
    ///
    /// Only available when the `clone` feature is enabled.
    pub fn ref_count(&self) -> usize {
        unsafe { self.inner.as_ref() }.count.load(Ordering::Acquire)
    }

    /// Returns `true` if this `ErrorUnion` is the sole owner of its underlying
    /// error, i.e. [`ErrorUnion::ref_count`] is `1`.
    ///
    /// Operations that take ownership of the underlying error (like
    /// [`ErrorUnion::take`], [`ErrorUnion::into_single`], [`ErrorUnion::into_inner`],
    /// [`ErrorUnion::map`], or a successful [`ErrorUnion::narrow`]) will panic
    /// unless this returns `true`. Check this first if you'd rather avoid the panic.
    ///
    /// Only available when the `clone` feature is enabled.
    pub fn is_unique(&self) -> bool {
        self.ref_count() == 1
    }
}

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

    #[allow(unused)] // Used with anyhow feature
    pub(crate) fn new_from_parts<T, OutSet, Index>(
        t: T,
        #[cfg(feature = "backtrace")] backtrace: std::backtrace::Backtrace,
        #[cfg(feature = "context")] context: Vec<ErosContext>,
        #[cfg(feature = "location")] location: &'static core::panic::Location<'static>,
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
            inner: t.into_raw_inner(),
            _pd: PhantomData,
        }
    }
}

//************************************************************************//

struct ErrorUnionErrorWrapper<E>(ErrorUnion<E>)
where
    E: TypeSet;

impl<E> core::error::Error for ErrorUnionErrorWrapper<E>
where
    E: TypeSet,
    E::Variants: core::error::Error + DebugFold + DisplayFold + ErrorFold,
{
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        E::Variants::source_fold(&unsafe { self.0.inner.as_ref() }.error as &dyn Any)
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
    E::Variants: core::error::Error + DebugFold + DisplayFold + ErrorFold,
{
    /// Creates a `Box<dyn SendSyncError>` error from this [`crate::ErrorUnion`]. This is used since
    /// [`crate::ErrorUnion`] cannot implement [`core::error::Error`] directly, otherwise trait implementations
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
    /// Consumes this `ErrorUnion`, returning the raw pointer to its (possibly
    /// shared) allocation without running `Drop` — i.e. without touching the
    /// reference count.
    ///
    /// This is the building block for every operation that either re-labels the
    /// same allocation with a different `TypeSet` (e.g. [`ErrorUnion::widen`]) or
    /// takes ownership of it to downcast (e.g. [`ErrorUnion::take`]). In both
    /// cases the logical "handle" is simply being moved, not duplicated or
    /// dropped, so the reference count must be left untouched.
    fn into_raw_inner(self) -> ptr::NonNull<ErrorUnionInner<dyn SendSyncError>> {
        let this = mem::ManuallyDrop::new(self);
        this.inner
    }

    /// Attempt to downcast the `ErrorUnion` into a specific type, and
    /// if that fails, return a `ErrorUnion` which does not contain that
    /// type as one of its possible variants.
    ///
    /// If the `clone` feature is enabled, this panics on a successful downcast
    /// unless this `ErrorUnion` is uniquely owned — see [`ErrorUnion::is_unique`].
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
        if unsafe { self.inner.as_ref() }.is_error::<Target>() {
            let ptr = self.into_raw_inner();
            Ok(unsafe { Box::from_raw(ptr.as_ptr()).downcast_error_unchecked::<Target>() })
        } else {
            Err(ErrorUnion {
                inner: self.into_raw_inner(),
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
            inner: self.into_raw_inner(),
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
        let is_match = E::Variants::is_fold(&unsafe { self.inner.as_ref() }.error as &dyn Any);
        let ptr = self.into_raw_inner();
        if is_match {
            Ok(ErrorUnion {
                inner: ptr,
                _pd: PhantomData,
            })
        } else {
            Err(ErrorUnion {
                inner: ptr,
                _pd: PhantomData,
            })
        }
    }

    /// For a `ErrorUnion` with a single variant, return
    /// the contained value.
    ///
    /// If the `clone` feature is enabled, this panics unless this `ErrorUnion`
    /// is uniquely owned — see [`ErrorUnion::is_unique`].
    pub fn take<Target>(self) -> Target
    where
        Target: 'static,
        E: TypeSet<Variants = Cons<Target, End>>,
    {
        let ptr = self.into_raw_inner();
        unsafe { Box::from_raw(ptr.as_ptr()).downcast_error_unchecked::<Target>() }
    }

    /// If the `clone` feature is enabled and the downcast succeeds, this panics
    /// unless this `ErrorUnion` is uniquely owned — see [`ErrorUnion::is_unique`].
    pub fn downcast_inner<T: 'static>(self) -> Option<T> {
        if unsafe { self.inner.as_ref() }.is_error::<T>() {
            let ptr = self.into_raw_inner();
            Some(unsafe { Box::from_raw(ptr.as_ptr()).downcast_error_unchecked::<T>() })
        } else {
            // Wrong type: just let `self` drop normally, which correctly
            // decrements the ref count (or frees the allocation) without
            // requiring unique ownership.
            None
        }
    }

    pub fn downcast_inner_ref<T: 'static>(&self) -> Option<&T> {
        unsafe { self.inner.as_ref() }.downcast_error_ref()
    }

    pub fn downcast_inner_mut<T: 'static>(&mut self) -> Option<&mut T> {
        unsafe { self.inner.as_mut() }.downcast_error_mut()
    }

    /// Returns true if the inner error is of type `T`
    pub fn is_inner<T: 'static>(&self) -> bool {
        unsafe { self.inner.as_ref() }.is_error::<T>()
    }

    #[cfg(feature = "backtrace")]
    pub fn backtrace(&self) -> &std::backtrace::Backtrace {
        unsafe { &self.inner.as_ref().backtrace }
    }

    pub fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        unsafe { self.inner.as_ref() }.error.source()
    }

    /// Gets a reference to the inner underlying error
    pub fn inner_ref(&self) -> &dyn SendSyncError {
        unsafe { &self.inner.as_ref().error }
    }

    /// Gets a mutable reference to the inner underlying error
    pub fn inner_mut(&mut self) -> &mut dyn SendSyncError {
        unsafe { &mut self.inner.as_mut().error }
    }

    /// Into the inner underlying error
    ///
    /// If the `clone` feature is enabled, this panics unless this `ErrorUnion`
    /// is uniquely owned — see [`ErrorUnion::is_unique`].
    pub fn into_inner(self) -> Box<dyn SendSyncError> {
        // Re-boxing via `into_box_fn` below destroys this allocation, which
        // would leave any other clones dangling, so this can't proceed unless
        // we're the sole owner. `downcast_error_unchecked{,_with_parts}` enforce
        // this same rule internally, but this method doesn't go through either
        // of them, so it needs its own check.
        #[cfg(feature = "clone")]
        assert_eq!(
            self.ref_count(),
            1,
            "cannot take ownership of an ErrorUnion's inner error while {} clone(s) of it exist",
            self.ref_count()
        );

        let ptr = self.into_raw_inner();
        let raw = ptr.as_ptr();
        unsafe {
            let into_box_fn = (*raw).into_box_fn;
            let error_ptr = ptr::addr_of_mut!((*raw).error);

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

    /// Returns the latest error.
    /// This takes into consideration errors added as context
    pub fn latest_error(&self) -> &dyn SendSyncError {
        #[cfg(feature = "context")]
        for context in unsafe { self.inner.as_ref() }.context.iter().rev() {
            if let crate::context::ContextSource::Error(err) = &context.context {
                return err.as_ref();
            }
        }
        self.inner_ref()
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

    /// Adds additional context. This becomes a no-op if the `context` feature is disabled.
    #[allow(unused_mut)]
    #[allow(unused_variables)]
    #[cfg_attr(feature = "location", track_caller)]
    pub fn context<C: Into<ContextSource>>(mut self, context: C) -> Self {
        #[cfg(feature = "context")]
        unsafe { self.inner.as_mut() }
            .context
            .push(crate::context::ErosContext::new(context.into()));
        self
    }

    /// Adds additional context that is user facing. This becomes a no-op if the `context` feature is disabled.
    #[cfg(feature = "user_context")]
    #[allow(unused_mut)]
    #[allow(unused_variables)]
    #[cfg_attr(feature = "location", track_caller)]
    pub fn user_context<C: Into<ContextSource>>(mut self, context: C) -> Self {
        #[cfg(feature = "context")]
        unsafe { self.inner.as_mut() }
            .context
            .push(crate::context::ErosContext::new_user_facing(context.into()));
        self
    }

    /// Lazily adds additional context. This becomes a no-op if the `context` feature is disabled.
    #[allow(unused_mut)]
    #[allow(unused_variables)]
    #[cfg_attr(feature = "location", track_caller)]
    pub fn with_context<F, C: Into<ContextSource>>(mut self, f: F) -> Self
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "context")]
        unsafe { self.inner.as_mut() }
            .context
            .push(crate::context::ErosContext::new(f().into()));
        self
    }

    /// Lazily adds additional user facing context. This becomes a no-op if the `context` feature is disabled.
    #[cfg(feature = "user_context")]
    #[allow(unused_mut)]
    #[allow(unused_variables)]
    #[cfg_attr(feature = "location", track_caller)]
    pub fn with_user_context<F, C: Into<ContextSource>>(mut self, f: F) -> Self
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "context")]
        unsafe { self.inner.as_mut() }
            .context
            .push(crate::context::ErosContext::new_user_facing(f().into()));
        self
    }
}

impl<A: 'static> AsRef<A> for ErrorUnion<(A,)> {
    fn as_ref(&self) -> &A {
        unsafe { self.inner.as_ref() }.downcast_error_ref().unwrap()
    }
}

impl<A: 'static> AsMut<A> for ErrorUnion<(A,)> {
    fn as_mut(&mut self) -> &mut A {
        unsafe { self.inner.as_mut() }.downcast_error_mut().unwrap()
    }
}

impl<A: 'static> ErrorUnion<(A,)> {
    /// Convert the inner type of an `ErrorUnion` with a single possible type to that type.
    ///
    /// Use `as_ref` or `as_mut` if you want to borrow the inner type instead of consuming the `ErrorUnion`.
    ///
    /// If the `clone` feature is enabled, this panics unless this `ErrorUnion`
    /// is uniquely owned — see [`ErrorUnion::is_unique`].
    pub fn into_single(self) -> A {
        let ptr = self.into_raw_inner();
        unsafe { Box::from_raw(ptr.as_ptr()).downcast_error_unchecked() }
    }

    /// If the `clone` feature is enabled, this panics unless this `ErrorUnion`
    /// is uniquely owned — see [`ErrorUnion::is_unique`].
    pub fn map<U, F>(self, f: F) -> ErrorUnion<(U,)>
    where
        U: SendSyncError,
        F: FnOnce(A) -> U,
    {
        let ptr = self.into_raw_inner();
        // SAFETY: We know that the inner error is only of type A, so we can safely downcast it
        let inner =
            unsafe { Box::from_raw(ptr.as_ptr()).downcast_error_unchecked_with_parts::<A>() };
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

// anyhow::Error does not implement `core::error::Error` so we need to wrap it
#[cfg(feature = "anyhow")]
#[derive(Debug)]
pub(crate) struct AnyhowError(pub(crate) anyhow::Error);

#[cfg(feature = "anyhow")]
impl fmt::Display for AnyhowError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

#[cfg(feature = "anyhow")]
impl core::error::Error for AnyhowError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        self.0.source()
    }
}

#[cfg(feature = "anyhow")]
#[derive(Debug)]
pub(crate) struct AnyhowErrorArc(pub(crate) alloc::sync::Arc<anyhow::Error>);

#[cfg(feature = "anyhow")]
impl fmt::Display for AnyhowErrorArc {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

#[cfg(feature = "anyhow")]
impl core::error::Error for AnyhowErrorArc {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
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
            core::panic::Location::caller(),
        )
    }

    #[cfg_attr(feature = "location", track_caller)]
    pub fn anyhow_arc(error: alloc::sync::Arc<anyhow::Error>) -> ErrorUnion {
        ErrorUnion::new_from_parts(
            AnyhowErrorArc(error),
            #[cfg(feature = "backtrace")]
            std::backtrace::Backtrace::disabled(),
            #[cfg(feature = "context")]
            Vec::new(),
            #[cfg(feature = "location")]
            core::panic::Location::caller(),
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
        assert!(unsafe { inner.as_ref() }.is_error::<FooError>());
        let recovered: FooError =
            unsafe { Box::from_raw(inner.as_ptr()).downcast_error_unchecked() };
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
        let recovered: VecError =
            unsafe { Box::from_raw(inner.as_ptr()).downcast_error_unchecked() };
        assert_eq!(recovered.0, payload);
    }

    #[test]
    #[should_panic]
    fn downcast_error_panics_on_wrong_type() {
        let inner = ErrorUnionInner::new(FooError("oops".into()));
        unsafe { Box::from_raw(inner.as_ptr()) }
            .downcast_error::<BarError>()
            .unwrap(); // should panic
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
            unsafe { union.inner.as_mut() }
                .context
                .push(ErosContext::new("step one".into()));
            unsafe { union.inner.as_mut() }
                .context
                .push(ErosContext::new("step two".into()));
        }

        let ptr = union.into_raw_inner();
        let parts: ErrorUnionInner<FooError> =
            unsafe { Box::from_raw(ptr.as_ptr()).downcast_error_unchecked_with_parts() };

        assert_eq!(parts.error, FooError("ctx".into()));

        #[cfg(feature = "context")]
        assert_eq!(parts.context.len(), 2);
    }

    #[test]
    fn downcast_error_unchecked_with_parts_correct_error_value() {
        let inner = ErrorUnionInner::new(BarError(42));
        let parts: ErrorUnionInner<BarError> =
            unsafe { Box::from_raw(inner.as_ptr()).downcast_error_unchecked_with_parts() };
        assert_eq!(parts.error, BarError(42));
    }

    #[test]
    fn into_dyn_error_and_back_roundtrips() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("roundtrip".into()));
        let dyn_err: Box<dyn SendSyncError> = union.into_dyn_error();
        assert!((&*dyn_err as &dyn Any).is::<ErrorUnionErrorWrapper<(FooError,)>>());
        let recovered: ErrorUnion<(FooError,)> =
            ErrorUnion::from_dyn_error(dyn_err).expect("round-trip should succeed");

        assert_eq!(recovered.as_ref(), &FooError("roundtrip".into()));
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
        assert_eq!(unsafe { recovered.inner.as_ref() }.context.len(), 1);

        assert_eq!(recovered.as_ref(), &FooError("ctx".into()));
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
        let dyn_err = union.into_inner();

        assert!(
            (&*dyn_err as &dyn Any).is::<FooError>(),
            "expected FooError, got a wrapper or wrong type"
        );
    }

    #[test]
    fn into_inner_dyn_error_value_is_preserved() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("preserved".into()));
        let dyn_err = union.into_inner();

        let foo = (&*dyn_err as &dyn Any).downcast_ref::<FooError>().unwrap();
        assert_eq!(foo, &FooError("preserved".into()));
    }

    #[test]
    fn into_inner_dyn_error_display_is_concrete_type() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("display".into()));
        let dyn_err = union.into_inner();

        assert_eq!(dyn_err.to_string(), "FooError(display)");
    }

    #[test]
    fn into_inner_dyn_error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>(_: T) {}
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("traits".into()));
        assert_send_sync(union.into_inner());
    }

    #[test]
    fn into_inner_dyn_error_differs_from_into_dyn_error() {
        let union_a: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("a".into()));
        let union_b: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("b".into()));

        let inner_dyn = union_a.into_inner();
        let wrapper_dyn = union_b.into_dyn_error();

        assert!((&*inner_dyn as &dyn Any).is::<FooError>());
        assert!(!(&*wrapper_dyn as &dyn Any).is::<FooError>());
        assert!((&*wrapper_dyn as &dyn Any).is::<ErrorUnionErrorWrapper<(FooError,)>>());
    }

    #[test]
    fn into_inner_dyn_error_multi_variant_foo() {
        let union: ErrorUnion<(FooError, BarError)> = ErrorUnion::new(FooError("multi".into()));
        let dyn_err = union.into_inner();

        assert!((&*dyn_err as &dyn Any).is::<FooError>());
        assert!(!(&*dyn_err as &dyn Any).is::<BarError>());
        let foo = (&*dyn_err as &dyn Any).downcast_ref::<FooError>().unwrap();
        assert_eq!(foo, &FooError("multi".into()));
    }

    #[test]
    fn into_inner_dyn_error_multi_variant_bar() {
        let union: ErrorUnion<(FooError, BarError)> = ErrorUnion::new(BarError(77));
        let dyn_err = union.into_inner();

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
        let dyn_err = union.into_inner();

        let recovered = (&*dyn_err as &dyn Any).downcast_ref::<VecError>().unwrap();
        assert_eq!(recovered.0, payload);
        // `dyn_err` drops here — Miri will catch any double-free or leak.
    }

    #[test]
    fn into_inner_dyn_error_not_roundtrippable_via_from_dyn_error() {
        // Confirm that from_dyn_error correctly rejects a bare inner error
        // (since it's not wrapped in ErrorUnionErrorWrapper).
        let union_a: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("bare".into()));
        let bare_dyn = union_a.into_inner();

        let result: Result<ErrorUnion<(FooError,)>, _> = ErrorUnion::from_dyn_error(bare_dyn);
        assert!(
            result.is_err(),
            "from_dyn_error should reject a bare inner error, not an ErrorUnionErrorWrapper"
        );
    }

    #[cfg(feature = "clone")]
    #[test]
    fn clone_bumps_ref_count_and_downcast_panics_while_shared() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("shared".into()));
        assert_eq!(union.ref_count(), 1);
        assert!(union.is_unique());

        let clone = union.clone();
        assert_eq!(union.ref_count(), 2);
        assert_eq!(clone.ref_count(), 2);
        assert!(!union.is_unique());

        drop(clone);
        assert_eq!(union.ref_count(), 1);
        assert!(union.is_unique());

        // Now that we're unique again, taking ownership succeeds.
        assert_eq!(union.into_single(), FooError("shared".into()));
    }

    #[cfg(feature = "clone")]
    #[test]
    #[should_panic]
    fn into_single_panics_while_shared() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("shared".into()));
        let _clone = union.clone();
        let _ = union.into_single(); // should panic: not uniquely owned
    }

    #[cfg(feature = "clone")]
    #[test]
    fn clones_see_same_error_value() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("same".into()));
        let clone = union.clone();
        assert_eq!(union.as_ref(), clone.as_ref());
    }
}

#[cfg(test)]
mod latest_error_tests {
    use super::*;
    use std::fmt;

    #[derive(Debug, PartialEq)]
    struct PrimaryError(String);
    impl fmt::Display for PrimaryError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "PrimaryError({})", self.0)
        }
    }
    impl std::error::Error for PrimaryError {}

    #[derive(Debug, PartialEq)]
    struct ContextError(String);
    impl fmt::Display for ContextError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "ContextError({})", self.0)
        }
    }
    impl std::error::Error for ContextError {}

    fn box_err(e: impl SendSyncError) -> Box<dyn SendSyncError> {
        Box::new(e)
    }

    #[test]
    fn latest_error_with_no_context_returns_inner() {
        let union: ErrorUnion<(PrimaryError,)> = ErrorUnion::new(PrimaryError("base".into()));

        assert_eq!(union.latest_error().to_string(), "PrimaryError(base)");
    }

    #[test]
    fn latest_error_with_no_context_returns_correct_type() {
        let union: ErrorUnion<(PrimaryError,)> = ErrorUnion::new(PrimaryError("base".into()));

        assert!(union.latest_error().as_any().is::<PrimaryError>());
    }

    #[cfg(feature = "context")]
    #[test]
    fn latest_error_with_string_context_only_returns_inner() {
        let union: ErrorUnion<(PrimaryError,)> = ErrorUnion::new(PrimaryError("base".into()));
        let union = union.context("just a string message");

        assert_eq!(union.latest_error().to_string(), "PrimaryError(base)");
    }

    #[cfg(feature = "context")]
    #[test]
    fn latest_error_with_error_context_returns_context_error() {
        let union: ErrorUnion<(PrimaryError,)> = ErrorUnion::new(PrimaryError("base".into()));
        let union = union.context(box_err(ContextError("ctx-1".into())));

        assert_eq!(union.latest_error().to_string(), "ContextError(ctx-1)");
    }

    #[cfg(feature = "context")]
    #[test]
    fn latest_error_returns_most_recently_added_error_context() {
        let union: ErrorUnion<(PrimaryError,)> = ErrorUnion::new(PrimaryError("base".into()));
        let union = union
            .context(box_err(ContextError("ctx-1".into())))
            .context(box_err(ContextError("ctx-2".into())))
            .context(box_err(ContextError("ctx-3".into())));

        assert_eq!(union.latest_error().to_string(), "ContextError(ctx-3)");
    }

    #[cfg(feature = "context")]
    #[test]
    fn latest_error_skips_trailing_string_contexts_to_find_error_context() {
        let union: ErrorUnion<(PrimaryError,)> = ErrorUnion::new(PrimaryError("base".into()));
        let union = union
            .context(box_err(ContextError("ctx-1".into())))
            .context("a string note added after");

        assert_eq!(union.latest_error().to_string(), "ContextError(ctx-1)");
    }

    #[cfg(feature = "context")]
    #[test]
    fn latest_error_with_only_string_contexts_falls_back_to_inner() {
        let union: ErrorUnion<(PrimaryError,)> = ErrorUnion::new(PrimaryError("base".into()));
        let union = union.context("note one").context("note two");

        assert_eq!(union.latest_error().to_string(), "PrimaryError(base)");
    }

    #[cfg(feature = "context")]
    #[test]
    fn latest_error_error_context_correct_concrete_type() {
        let union: ErrorUnion<(PrimaryError,)> = ErrorUnion::new(PrimaryError("base".into()));
        let union = union.context(box_err(ContextError("typed".into())));

        assert!(union.latest_error().as_any().is::<ContextError>());
    }

    #[cfg(feature = "context")]
    #[test]
    fn latest_error_multi_variant_union_with_error_context() {
        let union: ErrorUnion<(PrimaryError, ContextError)> =
            ErrorUnion::new(PrimaryError("base".into()));
        let union = union.context(box_err(ContextError("ctx-1".into())));

        assert_eq!(union.latest_error().to_string(), "ContextError(ctx-1)");
    }
}

#[cfg(test)]
mod downcast_inner_tests {
    use super::*;
    use std::fmt;

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
    fn downcast_inner_correct_type_returns_some() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("hello".into()));
        let result = union.downcast_inner::<FooError>();
        assert_eq!(result, Some(FooError("hello".into())));
    }

    #[test]
    fn downcast_inner_wrong_type_returns_none() {
        let union: ErrorUnion<(FooError, BarError)> = ErrorUnion::new(FooError("hello".into()));
        let result = union.downcast_inner::<BarError>();
        assert_eq!(result, None);
    }

    #[test]
    fn downcast_inner_multi_variant_correct_type() {
        let union: ErrorUnion<(FooError, BarError)> = ErrorUnion::new(BarError(7));
        let result = union.downcast_inner::<BarError>();
        assert_eq!(result, Some(BarError(7)));
    }

    #[test]
    fn downcast_inner_does_not_leak_or_double_drop() {
        // Vec payload lets Miri/ASan catch leaks or double-frees.
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
        let result = union.downcast_inner::<VecError>();
        assert_eq!(result, Some(VecError(payload)));
    }

    #[test]
    fn downcast_inner_wrong_type_drops_value_without_leaking() {
        // The Some(T) branch isn't taken; ensure the wrong-type path still
        // doesn't leak the original error (it lives on inside `self`/`union`
        // until `union` is dropped at the end of the test).
        #[derive(Debug, PartialEq)]
        struct VecError(Vec<u8>);
        impl fmt::Display for VecError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }
        impl std::error::Error for VecError {}

        let union: ErrorUnion<(VecError, FooError)> = ErrorUnion::new(VecError(vec![9, 9, 9]));
        let result = union.downcast_inner::<FooError>();
        assert_eq!(result, None);
        // `union`'s inner VecError is dropped normally here.
    }

    #[test]
    fn downcast_inner_ref_correct_type_returns_some() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("ref".into()));
        let result = union.downcast_inner_ref::<FooError>();
        assert_eq!(result, Some(&FooError("ref".into())));
    }

    #[test]
    fn downcast_inner_ref_wrong_type_returns_none() {
        let union: ErrorUnion<(FooError, BarError)> = ErrorUnion::new(FooError("ref".into()));
        let result = union.downcast_inner_ref::<BarError>();
        assert_eq!(result, None);
    }

    #[test]
    fn downcast_inner_ref_does_not_consume_union() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("still here".into()));
        let first = union.downcast_inner_ref::<FooError>();
        assert_eq!(first, Some(&FooError("still here".into())));
        // Union is still usable afterwards since this took `&self`.
        let second = union.downcast_inner_ref::<FooError>();
        assert_eq!(second, Some(&FooError("still here".into())));
    }

    #[test]
    fn downcast_inner_ref_multi_variant_bar() {
        let union: ErrorUnion<(FooError, BarError)> = ErrorUnion::new(BarError(123));
        assert_eq!(union.downcast_inner_ref::<BarError>(), Some(&BarError(123)));
        assert_eq!(union.downcast_inner_ref::<FooError>(), None);
    }

    #[test]
    fn downcast_inner_mut_correct_type_returns_some() {
        let mut union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("mut".into()));
        let result = union.downcast_inner_mut::<FooError>();
        assert_eq!(result, Some(&mut FooError("mut".into())));
    }

    #[test]
    fn downcast_inner_mut_wrong_type_returns_none() {
        let mut union: ErrorUnion<(FooError, BarError)> = ErrorUnion::new(FooError("mut".into()));
        let result = union.downcast_inner_mut::<BarError>();
        assert_eq!(result, None);
    }

    #[test]
    fn downcast_inner_mut_allows_mutation() {
        let mut union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("before".into()));
        {
            let inner = union.downcast_inner_mut::<FooError>().unwrap();
            inner.0 = "after".into();
        }
        assert_eq!(
            union.downcast_inner_ref::<FooError>(),
            Some(&FooError("after".into()))
        );
    }

    #[test]
    fn downcast_inner_mut_multi_variant_bar() {
        let mut union: ErrorUnion<(FooError, BarError)> = ErrorUnion::new(BarError(1));
        {
            let bar = union.downcast_inner_mut::<BarError>().unwrap();
            bar.0 = 99;
        }
        assert_eq!(union.downcast_inner_ref::<BarError>(), Some(&BarError(99)));
        assert_eq!(union.downcast_inner_mut::<FooError>(), None);
    }
}
