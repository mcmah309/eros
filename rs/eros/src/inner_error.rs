use crate::{SendSyncError, storage::ErrorOwner};
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::{
    error::Error,
    fmt,
    ops::{Deref, DerefMut},
};

/// Owns a concrete error extracted by [`crate::ErrorUnion::into_inner`] or
/// passed to a mapping callback.
///
/// Inspection, mutation, and concrete extraction work without allocation. The
/// representation is private: enabling `alloc` does not change the callback
/// type or the concrete type observed by downcasting. With `alloc`,
/// `into_boxed` or conversion into `Box<dyn SendSyncError>` can transfer the
/// original error into a source wrapper.
///
/// Mapping keeps saved union diagnostics with the mapped result;
/// `ErrorUnion::into_inner` discards them. Dropping this owner
/// drops the original error exactly once. Retaining this owner inside a new
/// error requires enough payload space; extracting a concrete error can keep
/// a replacement within the no-alloc inline limit.
pub struct InnerError {
    pub(crate) inner: ErrorOwner,
}

impl InnerError {
    /// Returns whether the original concrete error has type `T`.
    #[inline]
    pub fn is<T: 'static>(&self) -> bool {
        self.inner.is_error_type::<T>()
    }

    /// Borrows the original error if its concrete type is `T`.
    #[inline]
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.inner.downcast_error_ref::<T>()
    }

    /// Mutably borrows the original error if its concrete type is `T`.
    #[inline]
    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.inner.downcast_error_mut::<T>()
    }

    /// Extracts the original concrete error, returning this owner on mismatch.
    ///
    /// The result is always `T`, with every feature configuration. Extraction
    /// transfers ownership without allocating or dropping the extracted value.
    #[inline]
    pub fn downcast<T: 'static>(self) -> Result<T, Self> {
        if self.is::<T>() {
            // SAFETY: the exact concrete type was checked above.
            Ok(unsafe { self.inner.downcast_error_unchecked::<T>() })
        } else {
            Err(self)
        }
    }

    /// Borrows the original error through its error trait implementations.
    #[inline]
    pub fn as_error(&self) -> &dyn SendSyncError {
        self.inner.error()
    }

    /// Mutably borrows the original error through its error trait implementations.
    #[inline]
    pub fn as_error_mut(&mut self) -> &mut dyn SendSyncError {
        self.inner.error_mut()
    }

    /// Transfers the original concrete error into a box. Requires `alloc`.
    ///
    /// An already boxed root reuses its allocation when root diagnostics are
    /// disabled. Inline nonzero roots and roots sharing an allocation with
    /// diagnostics move into a new box; boxing a ZST needs no allocation.
    /// The box contains the original concrete error, so its downcasts and
    /// source chain continue to refer to that error. Conversion into
    /// `Box<dyn SendSyncError>` using `.into()` is equivalent.
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn into_boxed(self) -> Box<dyn SendSyncError> {
        self.inner.into_boxed()
    }
}

#[cfg(feature = "alloc")]
impl From<InnerError> for Box<dyn SendSyncError> {
    #[inline]
    fn from(error: InnerError) -> Self {
        error.into_boxed()
    }
}

impl Deref for InnerError {
    type Target = dyn SendSyncError;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_error()
    }
}
impl DerefMut for InnerError {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_error_mut()
    }
}
impl AsRef<dyn SendSyncError> for InnerError {
    #[inline]
    fn as_ref(&self) -> &(dyn SendSyncError + 'static) {
        self.as_error()
    }
}
impl AsMut<dyn SendSyncError> for InnerError {
    #[inline]
    fn as_mut(&mut self) -> &mut (dyn SendSyncError + 'static) {
        self.as_error_mut()
    }
}
impl fmt::Display for InnerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_error(), f)
    }
}
impl fmt::Debug for InnerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_error(), f)
    }
}
impl Error for InnerError {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.as_error().source()
    }
}
