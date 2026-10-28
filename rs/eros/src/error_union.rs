#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "context")]
use alloc::vec::Vec;
#[cfg(all(test, feature = "alloc"))]
use core::any::Any;
use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;

#[cfg(feature = "context")]
use crate::context::ContextFrame;
use crate::context::ContextValue;
use crate::formatting::Report;
use crate::type_set::{
    Contains, NarrowTarget, RecoveryHandler, RecoveryTarget, SupersetOf, TupleForm, TypeSet,
};

use crate::storage::ErrorUnionInner;
use crate::{AnyError, SendSyncDebugDisplay};

/// Any error that satisfies this trait's bounds can be used in a `ErrorUnion`
pub trait SendSyncError: core::error::Error + SendSyncDebugDisplay {}

impl<T> SendSyncError for T where T: core::error::Error + core::any::Any + Send + Sync + 'static {}

#[cfg(feature = "alloc")]
impl core::error::Error for Box<dyn SendSyncError> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&**self)
    }
}

/// `ErrorUnion` is an open sum type of errors. It differs from an enum
/// in that you do not need to define any actual new type
/// in order to hold some specific combination of variants,
/// but rather you simply describe the ErrorUnion as holding
/// one value out of several specific possibilities,
/// defined by using a tuple of those possible variants
/// as the generic parameter for the `ErrorUnion`.
/// Every tuple member must implement [`SendSyncError`].
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
    pub(crate) inner: ErrorUnionInner,
    // TypeId checks require exact variant types. A subtype coercion of E could
    // change a variant's TypeId and select the wrong unchecked enum downcast.
    pub(crate) _pd: PhantomData<fn(E) -> E>,
}

impl<T> Deref for ErrorUnion<(T,)>
where
    T: SendSyncError,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        // SAFETY: a singleton's invariant type set guarantees the concrete type.
        unsafe { self.inner.downcast_error_ref_unchecked::<T>() }
    }
}

impl<E: TypeSet> fmt::Debug for ErrorUnion<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Report::new(self).debug(formatter)
    }
}

impl<E: TypeSet> fmt::Display for ErrorUnion<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Report::new(self).display(formatter)
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
    // is_error(&error_union); // todo when specialization is stabilized
    is_error(&ErrorUnionAdapter::from(error_union));
}

// SAFETY: ErrorUnionInner owns only Send payloads and diagnostics. The invariant
// PhantomData carries no value, and T is additionally required to be Send.
unsafe impl<T> Send for ErrorUnion<T> where T: TypeSet + Send {}
// SAFETY: ErrorUnionInner permits shared access only to Sync payloads and
// diagnostics. PhantomData carries no value, and T is also required to be Sync.
unsafe impl<T> Sync for ErrorUnion<T> where T: TypeSet + Sync {}

//************************************************************************//

impl ErrorUnion {
    /// Create a new `ErrorUnion`.
    ///
    /// The union occupies two pointer-sized words, or three with `error_location`
    /// enabled without `alloc`. With `alloc` and root diagnostics enabled,
    /// diagnostics and payload share one allocation behind a boxed fat pointer.
    /// Otherwise, errors fitting one word in size and alignment are stored
    /// inline, with larger errors boxed when `alloc` is available. Without
    /// `alloc`, oversized or over-aligned nonzero errors are rejected at compile
    /// time. Zero-sized errors support arbitrary alignment.
    #[cfg_attr(feature = "error_location", track_caller)]
    pub fn new<T, OutSet, Index>(t: T) -> ErrorUnion<OutSet>
    where
        T: SendSyncError,
        OutSet: TypeSet,
        OutSet::Variants: Contains<T, Index>,
    {
        // todo when https://github.com/rust-lang/rust/issues/63084 is stabilized we should retstrict `T` from being `ErrorUnionVec` here and maybe elsewhere so it uses the correct construction function such that formatting is how we want it. Today if it uses `.union()` then it becomes how we want it.
        ErrorUnion {
            inner: ErrorUnionInner::new(t),
            _pd: PhantomData,
        }
    }

    #[allow(unused)] // Used with anyhow feature
    pub(crate) fn new_from_parts<T, OutSet, Index>(
        t: T,
        #[cfg(feature = "backtrace")] backtrace: std::backtrace::Backtrace,
        #[cfg(feature = "error_span_trace")] span_trace: tracing_error::SpanTrace,
        #[cfg(feature = "context")] context: Vec<ContextFrame>,
        #[cfg(feature = "error_location")] location: &'static core::panic::Location<'static>,
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
                #[cfg(feature = "error_span_trace")]
                span_trace,
                #[cfg(feature = "context")]
                context,
                #[cfg(feature = "error_location")]
                location,
            ),
            _pd: PhantomData,
        }
    }
}

//************************************************************************//

/// An [`ErrorUnion`] adapter implementing [`core::error::Error`].
///
/// Created using [`ErrorUnion::adapter`] or [`From`] without allocating or
/// discarding saved diagnostics.
///
/// Use [`AsRef`] or [`AsMut`] to borrow the union, and
/// [`union`](Self::union) to recover it by value.
/// To adapt a result's error directly, use [`ReshapeUnion::adapter`].
/// For plain errors, call [`IntoUnion::union`] first.
///
/// For a boxed trait object, use its native `downcast` method with the adapter's
/// exact error set, then call [`union`](Self::union).
/// A failed downcast returns the original box.
///
/// ```
/// # #[cfg(feature = "alloc")]
/// # {
/// # extern crate alloc;
/// use alloc::boxed::Box;
/// use eros::ErrorUnionAdapter;
///
/// let error = eros::error!("missing configuration");
/// let boxed: Box<dyn core::error::Error + Send + Sync> = Box::new(error.adapter());
/// let error = boxed.downcast::<ErrorUnionAdapter>()
///     .map(|adapter| adapter.union())
///     .unwrap();
/// assert_eq!(error.to_string(), "missing configuration");
/// # }
/// ```
///
/// `ErrorUnion` cannot implement `Error` directly because its blanket
/// conversion implementations would conflict.
///
/// Unions with the same error set also convert automatically through `?`.
/// Use [`.union()`](IntoUnion::union) for plain errors or
/// [`.widen()`](ReshapeUnion::widen) to adapt an existing union's error set.
///
/// ```
/// use eros::IntoUnion;
///
/// fn load() -> Result<(), eros::ErrorUnionAdapter> {
///     Err::<(), _>(core::fmt::Error).union()?;
///     Ok(())
/// }
///
/// fn accepts_error(_: impl core::error::Error + Send + Sync + 'static) {}
/// accepts_error(load().unwrap_err());
/// ```
pub struct ErrorUnionAdapter<E = AnyError>(ErrorUnion<E>)
where
    E: TypeSet;

impl<E: TypeSet> From<ErrorUnion<E>> for ErrorUnionAdapter<E> {
    fn from(error: ErrorUnion<E>) -> Self {
        Self(error)
    }
}

impl<E: TypeSet> AsRef<ErrorUnion<E>> for ErrorUnionAdapter<E> {
    fn as_ref(&self) -> &ErrorUnion<E> {
        &self.0
    }
}

impl<E: TypeSet> AsMut<ErrorUnion<E>> for ErrorUnionAdapter<E> {
    fn as_mut(&mut self) -> &mut ErrorUnion<E> {
        &mut self.0
    }
}

impl<E: TypeSet> ErrorUnionAdapter<E> {
    /// Removes the adapter, returning the original union.
    ///
    /// This operation is infallible and does not allocate or discard diagnostics.
    ///
    /// ```
    /// use eros::{ErrorUnion, ErrorUnionAdapter};
    ///
    /// let error: ErrorUnion<(core::fmt::Error,)> = ErrorUnion::new(core::fmt::Error);
    /// let adapter = ErrorUnionAdapter::from(error);
    /// assert!(adapter.as_ref().is_inner::<core::fmt::Error>());
    /// let error: ErrorUnion<(core::fmt::Error,)> = adapter.union();
    /// let _: core::fmt::Error = error.into_single();
    /// ```
    pub fn union(self) -> ErrorUnion<E> {
        self.0
    }
}

impl<E> core::error::Error for ErrorUnionAdapter<E>
where
    E: TypeSet,
{
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        self.0.source()
    }
}

impl<E> fmt::Display for ErrorUnionAdapter<E>
where
    E: TypeSet,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Native Error reporters traverse source() separately.
        write!(formatter, "{}", self.0.inner())
    }
}

impl<E> fmt::Debug for ErrorUnionAdapter<E>
where
    E: TypeSet,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

impl<E> ErrorUnion<E>
where
    E: TypeSet,
{
    /// Adapts this union to [`core::error::Error`] without allocating.
    ///
    /// Preserves the error set and all saved diagnostics. Use
    /// [`ErrorUnionAdapter::union`] to recover the original union.
    ///
    /// ```
    /// use eros::ErrorUnion;
    ///
    /// let error: ErrorUnion<(core::fmt::Error,)> = ErrorUnion::new(core::fmt::Error);
    /// let adapter = error.adapter();
    /// let error: ErrorUnion<(core::fmt::Error,)> = adapter.union();
    /// let _: core::fmt::Error = error.into_single();
    /// ```
    pub fn adapter(self) -> ErrorUnionAdapter<E> {
        self.into()
    }
}

//************************************************************************//

impl<E> ErrorUnion<E>
where
    E: TypeSet,
{
    /// Erases the static error set, preserving the concrete root and diagnostics.
    ///
    /// Reuses the existing storage without allocating or capturing new
    /// diagnostics. Boxed errors and domain wrappers remain intact, so later
    /// downcasts and recovery still see the same concrete root type.
    ///
    /// ```
    /// use eros::ErrorUnion;
    ///
    /// let typed: ErrorUnion<(std::fmt::Error,)> = ErrorUnion::new(std::fmt::Error);
    /// let erased: ErrorUnion = typed.erase();
    /// assert!(erased.is_inner::<std::fmt::Error>());
    /// ```
    pub fn erase(self) -> ErrorUnion {
        ErrorUnion {
            inner: self.inner,
            _pd: PhantomData,
        }
    }

    /// Selects one error type or a group, returning the remaining error union
    /// when the stored error does not match.
    ///
    /// A bare target, `narrow::<T, _>()`, extracts `T`, discarding saved
    /// diagnostics on a match. A tuple target, `narrow::<(T,), _>()`
    /// or `narrow::<(A, B), _>()`, returns an `ErrorUnion` of the selected types
    /// and preserves all diagnostics. The remainder always retains its
    /// diagnostics and original type order; the selected union uses the target order.
    ///
    /// Specify the target explicitly; the return type alone cannot infer it.
    /// Tuple targets support 0–26 types. An empty target `()` always returns
    /// the original union in `Err`. On an erased union, `AnyError` selects the
    /// original union without extracting the marker as a concrete error.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// use eros::{ErrorUnion, MsgError};
    /// use std::{fmt, io};
    ///
    /// let error: ErrorUnion<(MsgError, fmt::Error, io::Error)> =
    ///     ErrorUnion::new(MsgError::from("permission denied"));
    /// let error = error.context("read configuration");
    /// let selected = error.narrow::<(io::Error, MsgError), _>().unwrap();
    /// // Select a single type while retaining its diagnostics.
    /// let selected = selected.narrow::<(MsgError,), _>().unwrap();
    /// // Extract the concrete error when its diagnostics are no longer needed.
    /// let message = selected.narrow::<MsgError, _>().unwrap();
    /// assert_eq!(message.as_str(), "permission denied");
    /// # }
    /// ```
    #[allow(clippy::type_complexity)]
    pub fn narrow<Target, Index>(
        self,
    ) -> Result<Target::Output, ErrorUnion<<Target::Remainder as TupleForm>::Tuple>>
    where
        Target: NarrowTarget<E, Index>,
    {
        Target::split(self)
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

    /// Extracts the inner error if its concrete type is `T`.
    ///
    /// On success, saved diagnostics are discarded. On failure,
    /// returns the original union unchanged, including its error set and metadata.
    /// This checks only the inner error, not its sources or contexts.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// let error = eros::error!("failure").context("read configuration");
    /// let error = error.downcast_inner::<std::fmt::Error>().unwrap_err();
    /// let message = error.downcast_inner::<eros::MsgError>().unwrap();
    /// assert_eq!(message.as_str(), "failure");
    /// # }
    /// ```
    #[inline]
    pub fn downcast_inner<T: 'static>(self) -> Result<T, Self> {
        if self.inner.is_error_type::<T>() {
            // SAFETY: The concrete inner error type was checked above.
            Ok(unsafe { self.inner.downcast_error_unchecked::<T>() })
        } else {
            Err(self)
        }
    }

    #[inline]
    pub fn downcast_inner_ref<T: 'static>(&self) -> Option<&T> {
        self.inner.downcast_error_ref()
    }

    #[inline]
    pub fn downcast_inner_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.inner.downcast_error_mut()
    }

    /// Returns true if the inner error is of type `T`
    #[inline]
    pub fn is_inner<T: 'static>(&self) -> bool {
        self.inner.is_error_type::<T>()
    }

    #[cfg(feature = "backtrace")]
    pub const fn backtrace(&self) -> &std::backtrace::Backtrace {
        &self.inner.diagnostics().backtrace
    }

    /// Borrows the span trace captured when this error first entered Eros.
    ///
    /// Requires `error_span_trace` (also enabled by `span_trace`). The application must install
    /// [`tracing_error::ErrorLayer`] and instrument its operations for useful
    /// traces. Reshaping, recovery, and mapping preserve the original trace;
    /// borrowing or formatting never captures a new one.
    #[cfg(feature = "error_span_trace")]
    pub const fn span_trace(&self) -> &tracing_error::SpanTrace {
        &self.inner.diagnostics().span_trace
    }

    /// Returns where the error was first wrapped in an Eros union.
    ///
    /// Reshaping and mapping retain this original location.
    /// Requires `error_location` (also enabled by `location`). With
    /// `context` and `context_location`, each context frame also exposes
    /// its attachment location.
    #[cfg(feature = "error_location")]
    pub const fn location(&self) -> &'static core::panic::Location<'static> {
        self.inner.diagnostics().location
    }

    /// Borrows all context frames in attachment order (innermost first).
    ///
    /// Includes both ordinary and user-facing contexts, with their available
    /// metadata. Does not allocate or format values. Requires the `context` feature.
    /// Use [`ContextFrame::value`] to borrow each frame's message or boxed value.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// let error = eros::error!("permission denied").context("read configuration");
    /// for frame in error.contexts() {
    ///     println!("{}", frame.value());
    ///     #[cfg(feature = "context_location")]
    ///     println!("  at {}", frame.location());
    /// }
    /// # }
    /// ```
    ///
    /// With the `user_context` feature, filter for user-facing frames:
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// # #[cfg(feature = "user_context")]
    /// # {
    /// let error = eros::error!("permission denied")
    ///     .context("read configuration")
    ///     .user_context("Could not load your settings.");
    /// for frame in error.contexts().filter(|frame| frame.is_user_context()) {
    ///     println!("{frame}");
    /// }
    /// # }
    /// # }
    /// ```
    #[cfg(feature = "context")]
    pub fn contexts(
        &self,
    ) -> impl DoubleEndedIterator<Item = &crate::ContextFrame> + ExactSizeIterator {
        self.inner.diagnostics().context.iter()
    }

    pub fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        self.inner.error().source()
    }

    /// Returns a reference to the stored inner error.
    pub fn inner(&self) -> &dyn SendSyncError {
        self.inner.error()
    }

    /// Returns a mutable reference to the stored inner error.
    pub fn inner_mut(&mut self) -> &mut dyn SendSyncError {
        self.inner.error_mut()
    }

    /// Extracts the owned [`crate::InnerError`], discarding saved diagnostics.
    ///
    /// Preserves the original concrete error and its storage without allocating.
    /// Available with every feature configuration. The owner supports inspection,
    /// mutation, and owned downcasting. With `alloc`, call `.into_boxed()` on the
    /// returned owner when a boxed trait object is needed.
    ///
    /// ```
    /// use eros::{ErrorUnion, InnerError};
    /// use core::fmt;
    /// let error: ErrorUnion = ErrorUnion::new(fmt::Error);
    /// let inner: InnerError = error.into_inner();
    /// assert!(inner.is::<fmt::Error>());
    /// assert_eq!(inner.downcast::<fmt::Error>().unwrap(), fmt::Error);
    /// ```
    pub fn into_inner(self) -> crate::InnerError {
        self.inner.into_owner()
    }

    /// Adds additional context. This becomes a no-op if the `context` feature is disabled.
    #[allow(unused_mut)]
    #[allow(unused_variables)]
    #[cfg_attr(feature = "context_location", track_caller)]
    pub fn context<C: Into<ContextValue>>(mut self, context: C) -> Self {
        #[cfg(feature = "context")]
        self.inner
            .diagnostics_mut()
            .context
            .push(crate::context::ContextFrame::new(context.into()));
        self
    }

    /// Adds additional context that is user facing. This becomes a no-op if the `context` feature is disabled.
    #[cfg(feature = "user_context")]
    #[allow(unused_mut)]
    #[allow(unused_variables)]
    #[cfg_attr(feature = "context_location", track_caller)]
    pub fn user_context<C: Into<ContextValue>>(mut self, context: C) -> Self {
        #[cfg(feature = "context")]
        self.inner
            .diagnostics_mut()
            .context
            .push(crate::context::ContextFrame::new_user_context(
                context.into(),
            ));
        self
    }

    /// Lazily adds additional context. This becomes a no-op if the `context` feature is disabled.
    #[allow(unused_mut)]
    #[allow(unused_variables)]
    #[cfg_attr(feature = "context_location", track_caller)]
    pub fn with_context<F, C: Into<ContextValue>>(mut self, f: F) -> Self
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "context")]
        self.inner
            .diagnostics_mut()
            .context
            .push(crate::context::ContextFrame::new(f().into()));
        self
    }

    /// Lazily adds additional user facing context. This becomes a no-op if the `context` feature is disabled.
    #[cfg(feature = "user_context")]
    #[allow(unused_mut)]
    #[allow(unused_variables)]
    #[cfg_attr(feature = "context_location", track_caller)]
    pub fn with_user_context<F, C: Into<ContextValue>>(mut self, f: F) -> Self
    where
        F: FnOnce() -> C,
    {
        #[cfg(feature = "context")]
        self.inner
            .diagnostics_mut()
            .context
            .push(crate::context::ContextFrame::new_user_context(f().into()));
        self
    }
}

impl<A: SendSyncError> AsRef<A> for ErrorUnion<(A,)> {
    #[inline]
    fn as_ref(&self) -> &A {
        // SAFETY: a singleton's invariant type set guarantees the concrete type.
        unsafe { self.inner.downcast_error_ref_unchecked::<A>() }
    }
}

impl<A: SendSyncError> AsMut<A> for ErrorUnion<(A,)> {
    #[inline]
    fn as_mut(&mut self) -> &mut A {
        // SAFETY: a singleton's invariant type set guarantees the concrete type.
        unsafe { self.inner.downcast_error_mut_unchecked::<A>() }
    }
}

impl<A: SendSyncError> ErrorUnion<(A,)> {
    /// Borrows the single concrete inner error, preserving saved diagnostics.
    #[inline]
    pub fn as_single(&self) -> &A {
        self.as_ref()
    }

    /// Mutably borrows the single concrete inner error, preserving saved diagnostics.
    #[inline]
    pub fn as_single_mut(&mut self) -> &mut A {
        self.as_mut()
    }

    /// Convert the inner type of an `ErrorUnion` with a single possible type to that type.
    ///
    /// Use [`Self::as_single`] or [`Self::as_single_mut`] to borrow the inner type
    /// instead of consuming the `ErrorUnion`.
    #[inline]
    pub fn into_single(self) -> A {
        // SAFETY: the invariant singleton set guarantees the exact type A,
        // and consuming self transfers sole ownership to the extraction.
        unsafe { self.inner.downcast_error_unchecked() }
    }

    /// Maps the single concrete inner error, preserving saved diagnostics.
    ///
    /// With `alloc`, inline results need no allocation. Boxed roots reuse their
    /// allocation when the old and new complete storage layouts have the same
    /// size and alignment. Other boxed results allocate replacement storage.
    /// The storage and layout choices are known at compile time.
    ///
    /// [`Self::map_inner`] passes an owned [`crate::InnerError`] for unions
    /// with any number of variants. Without `alloc`, `U` must fit in one
    /// word in size and alignment, or be zero-sized, as with [`ErrorUnion::new`].
    pub fn map_single<U, F>(self, f: F) -> ErrorUnion<(U,)>
    where
        U: SendSyncError,
        F: FnOnce(A) -> U,
    {
        ErrorUnion {
            // SAFETY: the invariant singleton set guarantees the exact type A.
            inner: unsafe { self.inner.map_single::<A, U>(f) },
            _pd: PhantomData,
        }
    }
}

//************************************************************************//

/// Reshapes error sets and handles specific error types directly on results.
///
/// Implemented for [`crate::Result<T, E>`]. Reshaping an existing union retains
/// its saved diagnostics; extracting a concrete error discards them.
/// Import this trait directly or through [`crate::prelude`].
pub trait ReshapeUnion<S, E>
where
    E: TypeSet,
{
    /// Adapts the result's union to [`core::error::Error`] without allocating.
    ///
    /// Preserves the error set and all saved diagnostics. Successful values
    /// pass through unchanged. Equivalent to `.map_err(ErrorUnion::adapter)`.
    ///
    /// ```
    /// use eros::{ErrorUnion, ErrorUnionAdapter, ReshapeUnion};
    ///
    /// fn load() -> Result<(), ErrorUnionAdapter> {
    ///     let result: eros::Result<()> = Err(ErrorUnion::new(core::fmt::Error));
    ///     result.adapter()
    /// }
    ///
    /// assert!(load().unwrap_err().union().is_inner::<core::fmt::Error>());
    /// ```
    fn adapter(self) -> Result<S, ErrorUnionAdapter<E>>;

    /// Maps every inner error, preserving saved diagnostics.
    ///
    /// The callback always runs for `Err`, even if the original already has the
    /// replacement type. Successful values pass through without calling it.
    /// Receives the owned [`crate::InnerError`], just like [`ErrorUnion::map_inner`],
    /// and returns a union with the replacement as its single error type.
    /// Available with every feature configuration. For selective transformations,
    /// use [`Self::narrow`] to select errors before mapping them.
    ///
    /// ```
    /// use eros::{ErrorUnion, ReshapeUnion};
    /// use core::fmt;
    /// let result: eros::Result<u16> = Err(ErrorUnion::new(fmt::Error));
    /// let result: eros::Result<u16, (fmt::Error,)> = result.map_inner(|inner| {
    ///     assert!(inner.is::<fmt::Error>());
    ///     fmt::Error
    /// });
    /// assert!(result.unwrap_err().is_inner::<fmt::Error>());
    /// ```
    fn map_inner<T>(self, f: impl FnOnce(crate::InnerError) -> T) -> crate::Result<S, (T,)>
    where
        T: SendSyncError;

    /// Maps every inner error and widens to the destination error set.
    ///
    /// Equivalent to `self.map_inner(f).widen()`. The destination is inferred
    /// from the surrounding type and must contain the replacement type.
    /// Successful values pass through unchanged without calling `f`. Every
    /// error reaches the callback, including an existing replacement error.
    /// Saved diagnostics are preserved. See [`ErrorUnion::collapse`].
    ///
    /// ```
    /// use eros::{ErrorUnion, ReshapeUnion};
    /// use core::{fmt, num::ParseIntError};
    /// fn load() -> eros::Result<u16, (ParseIntError, fmt::Error)> {
    ///     let input: eros::Result<u16> = Err(ErrorUnion::new(fmt::Error));
    ///     let port = input.collapse(|_| fmt::Error)?;
    ///     Ok(port)
    /// }
    /// assert!(load().unwrap_err().is_inner::<fmt::Error>());
    /// ```
    fn collapse<Other, T, Index>(
        self,
        f: impl FnOnce(crate::InnerError) -> T,
    ) -> crate::Result<S, Other>
    where
        Other: TypeSet,
        T: SendSyncError,
        Other::Variants: SupersetOf<<(T,) as TypeSet>::Variants, Index>;

    /// Expands or reorders the result's error set.
    ///
    /// The destination must contain every type in the original set, or use
    /// [`AnyError`] to erase the set. `Other` is inferred from the destination;
    /// the proof parameter `Index` is inferred by the compiler.
    /// Successful values pass through unchanged. Errors retain their concrete
    /// type, storage, and saved diagnostics without allocating or capturing new ones.
    ///
    /// ```
    /// use eros::{ErrorUnion, ReshapeUnion};
    /// use core::{fmt, num::ParseIntError};
    ///
    /// let result: eros::Result<(), (fmt::Error,)> = Err(ErrorUnion::new(fmt::Error));
    /// let result: eros::Result<(), (ParseIntError, fmt::Error)> = result.widen();
    /// let erased: eros::Result<()> = result.widen();
    /// assert!(erased.unwrap_err().is_inner::<fmt::Error>());
    /// ```
    fn widen<Other, Index>(self) -> Result<S, ErrorUnion<Other>>
    where
        Other: TypeSet,
        Other::Variants: SupersetOf<E::Variants, Index>;

    /// Selects a matching error or group, returning the original success value
    /// or the remaining error union in the outer `Err` branch otherwise.
    ///
    /// As with [`ErrorUnion::narrow`], `narrow::<T, _>()` extracts a concrete
    /// error and discards its diagnostics. Tuple targets such as `(T,)` or
    /// `(A, B)` return an `ErrorUnion` with its diagnostics. The remainder
    /// always retains its diagnostics. Specify the target explicitly.
    /// Successful values pass through as `Err(Ok(value))`. The proof parameter
    /// `Index` is inferred by the compiler.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// use eros::{ErrorUnion, MsgError, ReshapeUnion};
    ///
    /// let result: eros::Result<u8, (MsgError, std::fmt::Error)> =
    ///     Err(ErrorUnion::new(MsgError::from("failed")));
    /// match result.narrow::<(MsgError,), _>() {
    ///     Ok(error) => assert_eq!(error.as_str(), "failed"),
    ///     Err(Ok(value)) => println!("success: {value}"),
    ///     Err(Err(remainder)) => eprintln!("unhandled: {remainder}"),
    /// }
    /// # }
    /// ```
    #[allow(clippy::type_complexity)]
    fn narrow<Target, Index>(
        self,
    ) -> Result<Target::Output, Result<S, ErrorUnion<<Target::Remainder as TupleForm>::Tuple>>>
    where
        Target: NarrowTarget<E, Index>;

    /// Handles one error type or a group, removing the handled types from the result.
    ///
    /// Calls `f` once on a matching error and returns its value as `Ok`. The
    /// handler receives a union of the selected types with its original
    /// diagnostics. Unhandled types retain their original order.
    /// Successful values and unhandled errors pass through without calling `f`.
    ///
    /// Use `recover::<ErrorType, _>(...)` for one type or
    /// `recover::<(FirstError, SecondError), _>(...)` for a group of 2–26 types.
    /// Alternatively, infer the target from an annotated handler argument such as
    /// `|error: ErrorUnion<(FirstError, SecondError)>| ...`.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// use eros::{ErrorUnion, ReshapeUnion};
    /// use std::{io, num::ParseIntError};
    ///
    /// let result: eros::Result<u16, (io::Error, ParseIntError)> =
    ///     Err(ErrorUnion::new(io::Error::other("cannot read port")));
    /// let result: eros::Result<u16, (ParseIntError,)> =
    ///     result.recover(|error: ErrorUnion<(io::Error,)>| {
    ///         eprintln!("{error:?}; using port 8080");
    ///         8080
    ///     });
    /// assert_eq!(result.unwrap(), 8080);
    /// # }
    /// ```
    ///
    /// Handle all errors in a group with one callback:
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// use eros::{ErrorUnion, ReshapeUnion};
    /// use std::{io, num::ParseIntError};
    ///
    /// let result: eros::Result<u16, (io::Error, ParseIntError)> =
    ///     Err(ErrorUnion::new(io::Error::other("cannot read port")));
    /// let port = result
    ///     .recover(|error: ErrorUnion<(io::Error, ParseIntError)>| {
    ///         eprintln!("{error:?}; using port 8080");
    ///         8080
    ///     })
    ///     .into_value();
    /// assert_eq!(port, 8080);
    /// # }
    /// ```
    #[allow(clippy::type_complexity)]
    fn recover<Target, Index>(
        self,
        f: impl RecoveryHandler<Target, E, Index, S>,
    ) -> Result<S, ErrorUnion<<Target::Remainder as TupleForm>::Tuple>>
    where
        Target: RecoveryTarget<E, Index>;

    /// Handles one error type or a group with a fallible fallback.
    ///
    /// The handler receives the matching error with its original diagnostics.
    /// Its result is returned unchanged, including any
    /// fallback error's diagnostics. The original error's diagnostics are not
    /// automatically attached to a new fallback error. Successful input values
    /// and unhandled errors pass through without calling the handler.
    ///
    /// `Other` is inferred from the destination or the handler's return type.
    /// It must contain every unhandled error type as well as the errors returned
    /// by the handler. Use `.union()` or `.widen()` inside the handler to convert
    /// a fallback result into that set. The handler may reintroduce handled types.
    /// Select a group with `|error: ErrorUnion<(FirstError, SecondError)>| ...`
    /// or `try_recover::<(FirstError, SecondError), Other, _, _>(...)`.
    /// As with [`Self::recover`], explicit single-type targets use the concrete
    /// error type, and groups use tuples of 2–26 types. The index parameters are inferred.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// use eros::{ErrorUnion, IntoUnion, ReshapeUnion};
    /// use std::{io, num::ParseIntError};
    ///
    /// let result: eros::Result<u16, (io::Error, ParseIntError)> =
    ///     Err(ErrorUnion::new(io::Error::other("cannot read port")));
    /// let result: eros::Result<u16, (ParseIntError,)> =
    ///     result.try_recover(|error: ErrorUnion<(io::Error,)>| {
    ///         eprintln!("{error:?}; parsing fallback port");
    ///         "8080".parse::<u16>().union()
    ///     });
    /// assert_eq!(result.unwrap(), 8080);
    /// # }
    /// ```
    fn try_recover<Target, Other, Index, OtherIndex>(
        self,
        f: impl RecoveryHandler<Target, E, Index, Result<S, ErrorUnion<Other>>>,
    ) -> Result<S, ErrorUnion<Other>>
    where
        Target: RecoveryTarget<E, Index>,
        Other: TypeSet,
        Other::Variants: SupersetOf<Target::Remainder, OtherIndex>;

    /// Extracts the success value once no possible error types remain.
    ///
    /// Only available for `eros::Result<S, ()>`. Calling this method does not
    /// compile if an unhandled error type remains in the result's set.
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// use eros::{ErrorUnion, MsgError, ReshapeUnion};
    ///
    /// let result: eros::Result<u16, (MsgError,)> =
    ///     Err(ErrorUnion::new(MsgError::from("missing port")));
    /// let port = result.recover::<MsgError, _>(|_| 8080).into_value();
    /// assert_eq!(port, 8080);
    /// # }
    /// ```
    fn into_value(self) -> S
    where
        E: TypeSet<Variants = crate::type_set::End>;
}

impl<S, E> ReshapeUnion<S, E> for Result<S, ErrorUnion<E>>
where
    E: TypeSet,
{
    fn adapter(self) -> Result<S, ErrorUnionAdapter<E>> {
        self.map_err(ErrorUnion::adapter)
    }

    fn map_inner<T>(self, f: impl FnOnce(crate::InnerError) -> T) -> crate::Result<S, (T,)>
    where
        T: SendSyncError,
    {
        self.map_err(|error| error.map_inner(f))
    }

    fn collapse<Other, T, Index>(
        self,
        f: impl FnOnce(crate::InnerError) -> T,
    ) -> crate::Result<S, Other>
    where
        Other: TypeSet,
        T: SendSyncError,
        Other::Variants: SupersetOf<<(T,) as TypeSet>::Variants, Index>,
    {
        self.map_inner(f).widen()
    }

    fn widen<Other, Index>(self) -> Result<S, ErrorUnion<Other>>
    where
        Other: TypeSet,
        Other::Variants: SupersetOf<E::Variants, Index>,
    {
        self.map_err(|e| e.widen())
    }

    fn narrow<Target, Index>(
        self,
    ) -> Result<Target::Output, Result<S, ErrorUnion<<Target::Remainder as TupleForm>::Tuple>>>
    where
        Target: NarrowTarget<E, Index>,
    {
        match self {
            Ok(value) => Err(Ok(value)),
            Err(err) => match err.narrow::<Target, Index>() {
                Ok(value) => Ok(value),
                Err(err) => Err(Err(err)),
            },
        }
    }

    fn recover<Target, Index>(
        self,
        f: impl RecoveryHandler<Target, E, Index, S>,
    ) -> Result<S, ErrorUnion<<Target::Remainder as TupleForm>::Tuple>>
    where
        Target: RecoveryTarget<E, Index>,
    {
        match self {
            Ok(value) => Ok(value),
            Err(error) => match Target::split(error) {
                Ok(selected) => Ok(f(selected)),
                Err(remainder) => Err(remainder),
            },
        }
    }

    fn try_recover<Target, Other, Index, OtherIndex>(
        self,
        f: impl RecoveryHandler<Target, E, Index, Result<S, ErrorUnion<Other>>>,
    ) -> Result<S, ErrorUnion<Other>>
    where
        Target: RecoveryTarget<E, Index>,
        Other: TypeSet,
        Other::Variants: SupersetOf<Target::Remainder, OtherIndex>,
    {
        match self {
            Ok(value) => Ok(value),
            Err(error) => match Target::split(error) {
                Ok(selected) => f(selected),
                // Other contains every unhandled type. Retain the remainder's
                // storage and diagnostics, changing only its type marker.
                Err(remainder) => Err(ErrorUnion {
                    inner: remainder.inner,
                    _pd: PhantomData,
                }),
            },
        }
    }

    fn into_value(self) -> S
    where
        E: TypeSet<Variants = crate::type_set::End>,
    {
        match self {
            Ok(value) => value,
            // TypeSet is sealed, and only () has an empty variant list. A union
            // with that empty set cannot be constructed through the safe API.
            Err(_) => unreachable!("an empty error set cannot contain an error"),
        }
    }
}

//************************************************************************//

/// Converts plain result errors into unions.
///
/// Implemented for `Result<S, F>` when `F` implements [`SendSyncError`]. The
/// destination determines whether errors are tracked in a typed set or erased
/// with [`AnyError`]. Successful values pass through unchanged.
/// Import this trait directly or through [`crate::prelude`].
pub trait IntoUnion<S, F> {
    /// Wraps the result's error in a union whose error set is inferred from the destination.
    ///
    /// A typed destination must include `F`; [`AnyError`] accepts any supported
    /// error type. The proof parameter `Index` is inferred by the compiler.
    /// Errors enter Eros through [`ErrorUnion::new`], capturing enabled diagnostics
    /// and following its storage requirements. Successful values pass through unchanged.
    ///
    /// ```
    /// use eros::IntoUnion;
    /// use core::{fmt, num::ParseIntError};
    ///
    /// let typed: eros::Result<(), (fmt::Error, ParseIntError)> = Err(fmt::Error).union();
    /// assert!(typed.unwrap_err().is_inner::<fmt::Error>());
    ///
    /// let erased: eros::Result<()> = Err(fmt::Error).union();
    /// assert!(erased.unwrap_err().is_inner::<fmt::Error>());
    /// ```
    fn union<Other, Index>(self) -> Result<S, ErrorUnion<Other>>
    where
        Other: TypeSet,
        Other::Variants: Contains<F, Index>;
}

impl<S, F: SendSyncError> IntoUnion<S, F> for Result<S, F> {
    #[cfg_attr(feature = "error_location", track_caller)]
    fn union<Other, Index>(self) -> Result<S, ErrorUnion<Other>>
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

/// Erases a result's static error type into an [`ErrorUnion<AnyError>`].
pub trait Erase<S> {
    /// Preserves the concrete error, including boxes and domain wrappers.
    ///
    /// Existing unions retain their storage and saved diagnostics.
    /// Plain errors are wrapped in a new union. Successful values pass through
    /// unchanged. This works with or without `alloc`.
    ///
    /// ```
    /// use eros::Erase;
    ///
    /// let result = Err::<(), _>(std::fmt::Error);
    /// let erased: eros::Result<()> = result.erase();
    /// assert!(erased.unwrap_err().is_inner::<std::fmt::Error>());
    /// ```
    fn erase(self) -> Result<S, ErrorUnion>;
}

impl<S, F: SendSyncError> Erase<S> for Result<S, F> {
    #[cfg_attr(feature = "error_location", track_caller)]
    fn erase(self) -> Result<S, ErrorUnion> {
        match self {
            Ok(value) => Ok(value),
            Err(err) => Err(ErrorUnion::new(err)),
        }
    }
}

impl<S, E: TypeSet> Erase<S> for Result<S, ErrorUnion<E>> {
    fn erase(self) -> Result<S, ErrorUnion> {
        self.map_err(ErrorUnion::erase)
    }
}

// pub trait IntoUnion<S, F> {
//     /// Con `Err` to i
//     fn union<Other, Index>(self) -> Result<S, ErrorUnion<Other>>
//     where
//         Other: TypeSet,
//         Other::Variants: Contains<F, Index>;
// }

// impl<S, F1, F2> IntoUnion<S, F2> for Result<S, F1>
// where
//     F1: Into<F2> + SendSyncError, // `SendSyncError` is used to ensure it does not overlap with below
//     F2: SendSyncError,
// {
//     fn union<Other, Index>(self) -> Result<S, ErrorUnion<Other>>
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
//         self.map_err(|e| e.map_single(|e| e.into()))
//     }
// }

// Commented out since it conflicts with the above and cant merge into one trait since the return type is attached to the method generic
// pub trait IntoUnion<F>
// where
//     F: AnyError,
// {
//     fn union<Other, Index>(self) -> ErrorUnion<Other>
//     where
//         Other: TypeSet,
//         // Other::Variants: SupersetOf<Cons<F, End>, Index>,
//         Other::Variants: Contains<F, Index>;
// }

// impl<F> IntoUnion<F> for F
// where
//     F: AnyError,
// {
//     fn union<Other, Index>(self) -> ErrorUnion<Other>
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
    #[cfg_attr(feature = "error_location", track_caller)]
    pub fn from_anyhow(error: anyhow::Error) -> ErrorUnion {
        ErrorUnion::new_from_parts(
            AnyhowError(error),
            #[cfg(feature = "backtrace")]
            std::backtrace::Backtrace::disabled(),
            #[cfg(feature = "error_span_trace")]
            tracing_error::SpanTrace::capture(),
            #[cfg(feature = "context")]
            Vec::new(),
            #[cfg(feature = "error_location")]
            core::panic::Location::caller(),
        )
    }

    #[cfg_attr(feature = "error_location", track_caller)]
    pub fn from_anyhow_arc(error: alloc::sync::Arc<anyhow::Error>) -> ErrorUnion {
        ErrorUnion::new_from_parts(
            AnyhowErrorArc(error),
            #[cfg(feature = "backtrace")]
            std::backtrace::Backtrace::disabled(),
            #[cfg(feature = "error_span_trace")]
            tracing_error::SpanTrace::capture(),
            #[cfg(feature = "context")]
            Vec::new(),
            #[cfg(feature = "error_location")]
            core::panic::Location::caller(),
        )
    }
}

// Needs specialization
// impl From<anyhow::Error> for ErrorUnion {
//     fn from(value: anyhow::Error) -> Self {
//         ErrorUnion::from_anyhow(value)
//     }
// }

#[cfg(feature = "anyhow")]
impl From<ErrorUnion> for anyhow::Error {
    #[allow(unused_mut)]
    fn from(mut value: ErrorUnion) -> Self {
        #[cfg(feature = "context")]
        let context = core::mem::take(&mut value.inner.diagnostics_mut().context);
        let inner_error = value.into_inner().into_boxed();
        let mut error = anyhow::Error::new(inner_error);
        #[cfg(feature = "context")]
        for c in context {
            error = error.context(c.context);
        }
        error
    }
}

//************************************************************************//

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;
    use std::fmt;
    #[cfg(not(feature = "std"))]
    use std::{prelude::v1::*, vec};

    //************************************************************************//

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

    #[cfg(not(any(
        feature = "backtrace",
        feature = "context",
        feature = "error_location",
        feature = "error_span_trace"
    )))]
    #[test]
    fn inner_without_metadata_uses_two_words_for_every_error_type() {
        assert_eq!(size_of::<ErrorUnionInner>(), 2 * size_of::<usize>());
        assert_eq!(align_of::<ErrorUnionInner>(), align_of::<usize>());
        assert_eq!(
            ErrorUnionInner::new(BarError(42)).error().to_string(),
            "BarError(42)"
        );
    }

    #[test]
    fn downcast_error_unchecked_correct_type_recovers_value() {
        let inner = ErrorUnionInner::new(FooError("hello".into()));
        assert!(inner.is_error_type::<FooError>());
        // SAFETY: construction and the preceding check establish exactly FooError.
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
        // SAFETY: inner was constructed with exactly VecError.
        let recovered: VecError = unsafe { inner.downcast_error_unchecked() };
        assert_eq!(recovered.0, payload);
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
                .diagnostics_mut()
                .context
                .push(ContextFrame::new("step one".into()));
            union
                .inner
                .diagnostics_mut()
                .context
                .push(ContextFrame::new("step two".into()));
        }

        // SAFETY: the payload was constructed as FooError; adding context above
        // changes only diagnostics, so its concrete type remains unchanged.
        let parts: crate::storage::ErrorParts<FooError> =
            unsafe { union.inner.downcast_error_unchecked_with_parts() };

        assert_eq!(parts.error, FooError("ctx".into()));

        #[cfg(feature = "context")]
        assert_eq!(parts.diagnostics.context.len(), 2);
    }

    #[test]
    fn downcast_error_unchecked_with_parts_correct_error_value() {
        let inner = ErrorUnionInner::new(BarError(42));
        // SAFETY: inner was constructed with exactly BarError.
        let parts: crate::storage::ErrorParts<BarError> =
            unsafe { inner.downcast_error_unchecked_with_parts() };
        assert_eq!(parts.error, BarError(42));
    }

    #[test]
    fn boxed_std_error_and_back_roundtrips() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("roundtrip".into()));
        let dyn_err: Box<dyn core::error::Error + Send + Sync> =
            Box::new(ErrorUnionAdapter::from(union));
        let recovered = dyn_err
            .downcast::<ErrorUnionAdapter<(FooError,)>>()
            .expect("round-trip should succeed")
            .union();

        assert_eq!(recovered.as_ref(), &FooError("roundtrip".into()));
    }

    #[test]
    fn boxed_adapter_downcast_wrong_type_returns_err() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("mismatch".into()));
        let dyn_err: Box<dyn core::error::Error + Send + Sync> =
            Box::new(ErrorUnionAdapter::from(union));

        let result = dyn_err.downcast::<ErrorUnionAdapter<(BarError,)>>();
        assert!(result.is_err(), "mismatched type should be returned as Err");
    }

    #[test]
    fn std_error_display_delegates_to_inner() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("display".into()));
        let adapter = ErrorUnionAdapter::from(union);
        assert!(adapter.to_string().contains("FooError(display)"));
    }

    #[test]
    fn std_error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>(_: T) {}
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("traits".into()));
        assert_send_sync(ErrorUnionAdapter::from(union));
    }

    #[test]
    fn boxed_adapter_downcast_preserves_context() {
        let mut union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("ctx".into()));
        #[cfg(feature = "context")]
        {
            union = union.context("some context");
        }
        let dyn_err: Box<dyn core::error::Error + Send + Sync> =
            Box::new(ErrorUnionAdapter::from(union));
        let recovered = dyn_err
            .downcast::<ErrorUnionAdapter<(FooError,)>>()
            .unwrap()
            .union();

        #[cfg(feature = "context")]
        assert_eq!(recovered.inner.diagnostics().context.len(), 1);

        assert_eq!(recovered.as_ref(), &FooError("ctx".into()));
    }

    #[test]
    fn multi_variant_union_boxed_std_error_roundtrips() {
        let union: ErrorUnion<(FooError, BarError)> = ErrorUnion::new(BarError(99));
        let dyn_err: Box<dyn core::error::Error + Send + Sync> =
            Box::new(ErrorUnionAdapter::from(union));

        let recovered = dyn_err
            .downcast::<ErrorUnionAdapter<(FooError, BarError)>>()
            .unwrap()
            .union();

        let bar: BarError = recovered.narrow::<BarError, _>().unwrap();
        assert_eq!(bar, BarError(99));
    }

    #[test]
    fn into_inner_owner_returns_concrete_type_not_wrapper() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("concrete".into()));
        let dyn_err = union.into_inner();

        assert!(
            dyn_err.is::<FooError>(),
            "expected FooError, got a wrapper or wrong type"
        );
    }

    #[test]
    fn into_inner_owner_value_is_preserved() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("preserved".into()));
        let dyn_err = union.into_inner();

        let foo = dyn_err.downcast_ref::<FooError>().unwrap();
        assert_eq!(foo, &FooError("preserved".into()));
    }

    #[test]
    fn into_inner_owner_display_is_concrete_type() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("display".into()));
        let dyn_err = union.into_inner();

        assert_eq!(dyn_err.to_string(), "FooError(display)");
    }

    #[test]
    fn into_inner_owner_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>(_: T) {}
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("traits".into()));
        assert_send_sync(union.into_inner());
    }

    #[test]
    fn into_inner_differs_from_boxed_std_error() {
        let union_a: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("a".into()));
        let union_b: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("b".into()));

        let inner_dyn = union_a.into_inner();
        let wrapper_dyn = Box::new(ErrorUnionAdapter::from(union_b));

        assert!(inner_dyn.is::<FooError>());
        assert!(!(&*wrapper_dyn as &dyn Any).is::<FooError>());
        assert!((&*wrapper_dyn as &dyn Any).is::<ErrorUnionAdapter<(FooError,)>>());
    }

    #[test]
    fn into_inner_owner_multi_variant_foo() {
        let union: ErrorUnion<(FooError, BarError)> = ErrorUnion::new(FooError("multi".into()));
        let dyn_err = union.into_inner();

        assert!(dyn_err.is::<FooError>());
        assert!(!dyn_err.is::<BarError>());
        let foo = dyn_err.downcast_ref::<FooError>().unwrap();
        assert_eq!(foo, &FooError("multi".into()));
    }

    #[test]
    fn into_inner_owner_multi_variant_bar() {
        let union: ErrorUnion<(FooError, BarError)> = ErrorUnion::new(BarError(77));
        let dyn_err = union.into_inner();

        assert!(dyn_err.is::<BarError>());
        assert!(!dyn_err.is::<FooError>());
        let bar = dyn_err.downcast_ref::<BarError>().unwrap();
        assert_eq!(bar, &BarError(77));
    }

    #[test]
    fn into_inner_owner_does_not_leak_heap_allocation() {
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

        let recovered = dyn_err.downcast_ref::<VecError>().unwrap();
        assert_eq!(recovered.0, payload);
        // `dyn_err` drops here — Miri will catch any double-free or leak.
    }

    #[test]
    fn into_inner_owner_is_not_an_adapter() {
        let union_a: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("bare".into()));
        let bare_dyn: Box<dyn core::error::Error + Send + Sync> = union_a.into_inner().into_boxed();

        let result = bare_dyn.downcast::<ErrorUnionAdapter<(FooError,)>>();
        assert!(
            result.is_err(),
            "a bare inner error should not downcast to an ErrorUnionAdapter"
        );
    }
}

#[cfg(all(test, feature = "alloc"))]
mod downcast_inner_tests {
    use super::*;
    use std::fmt;
    #[cfg(not(feature = "std"))]
    use std::{prelude::v1::*, vec};

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
    fn downcast_inner_correct_type_returns_ok() {
        let union: ErrorUnion<(FooError,)> = ErrorUnion::new(FooError("hello".into()));
        let result = union.downcast_inner::<FooError>();
        assert_eq!(result.unwrap(), FooError("hello".into()));
    }

    #[test]
    fn downcast_inner_wrong_type_returns_original_union() {
        let union: ErrorUnion<(FooError, BarError)> = ErrorUnion::new(FooError("hello".into()));
        let result = union.downcast_inner::<BarError>();
        let union: ErrorUnion<(FooError, BarError)> = result.unwrap_err();
        assert_eq!(
            union.downcast_inner::<FooError>().unwrap(),
            FooError("hello".into())
        );
    }

    #[test]
    fn downcast_inner_multi_variant_correct_type() {
        let union: ErrorUnion<(FooError, BarError)> = ErrorUnion::new(BarError(7));
        let result = union.downcast_inner::<BarError>();
        assert_eq!(result.unwrap(), BarError(7));
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
        assert_eq!(result.unwrap(), VecError(payload));
    }

    #[test]
    fn downcast_inner_wrong_type_retains_value_without_leaking() {
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
        let union = result.unwrap_err();
        assert_eq!(
            union.downcast_inner::<VecError>().unwrap(),
            VecError(vec![9, 9, 9])
        );
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
