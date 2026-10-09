use core::{any::Any, mem::ManuallyDrop, ptr::NonNull};

use crate::SendSyncError;

/// Owns a zero-sized error through its trait-object metadata, without allocating.
/// The data pointer is non-null and aligned for the concrete type. Only `new`
/// and `new_from_parts` establish ownership, and both require a supplied ZST.
pub(crate) struct ZstError {
    error: NonNull<dyn SendSyncError>,
    #[cfg(feature = "location")]
    pub(crate) location: &'static core::panic::Location<'static>,
}

pub(crate) struct ErrorParts<T> {
    pub(crate) error: T,
    #[cfg(feature = "location")]
    pub(crate) location: &'static core::panic::Location<'static>,
}

impl ZstError {
    #[cfg_attr(feature = "location", track_caller)]
    pub(crate) fn new<T: SendSyncError>(value: T) -> Self {
        Self::new_from_parts(
            value,
            #[cfg(feature = "location")]
            core::panic::Location::caller(),
        )
    }

    pub(crate) fn new_from_parts<T: SendSyncError>(
        value: T,
        #[cfg(feature = "location")] location: &'static core::panic::Location<'static>,
    ) -> Self {
        const {
            assert!(
                size_of::<T>() == 0,
                "T must be zero-sized when the alloc feature is disabled"
            );
        }
        let error = NonNull::<T>::dangling();
        // SAFETY: T has no bytes and the pointer is non-null and aligned for T.
        // The supplied value establishes validity, including for ZSTs that
        // cannot be freely constructed. Writing transfers ownership to us.
        unsafe {
            error.as_ptr().write(value);
        }
        Self {
            error,
            #[cfg(feature = "location")]
            location,
        }
    }

    pub(crate) fn error(&self) -> &dyn SendSyncError {
        // SAFETY: the pointer describes the valid owned ZST established at
        // construction. The reference lifetime is bounded by this owner.
        unsafe { self.error.as_ref() }
    }

    pub(crate) fn error_mut(&mut self) -> &mut dyn SendSyncError {
        // SAFETY: exclusive access to the owned ZST. Independently owned ZSTs
        // have no overlapping bytes even if dangling addresses coincide.
        unsafe { self.error.as_mut() }
    }

    #[inline]
    pub(crate) fn is_error_type<T: 'static>(&self) -> bool {
        (self.error() as &dyn Any).is::<T>()
    }

    /// # Safety
    /// T must be the exact concrete type of the owned error.
    #[inline]
    pub(crate) unsafe fn downcast_error_unchecked<T: 'static>(self) -> T {
        debug_assert!(self.is_error_type::<T>());
        let owner = ManuallyDrop::new(self);
        // SAFETY: the caller guarantees the exact concrete type; reading
        // transfers ownership and ManuallyDrop prevents another destructor.
        unsafe { owner.error.as_ptr().cast::<T>().read() }
    }

    /// # Safety
    /// T must be the exact concrete type of the owned error.
    #[inline]
    pub(crate) unsafe fn downcast_error_unchecked_with_parts<T: 'static>(self) -> ErrorParts<T> {
        #[cfg(feature = "location")]
        let location = self.location;
        ErrorParts {
            // SAFETY: the caller guarantees the exact concrete type.
            error: unsafe { self.downcast_error_unchecked() },
            #[cfg(feature = "location")]
            location,
        }
    }

    #[inline]
    pub(crate) fn downcast_error_ref<T: 'static>(&self) -> Option<&T> {
        (self.error() as &dyn Any).downcast_ref()
    }

    #[inline]
    pub(crate) fn downcast_error_mut<T: 'static>(&mut self) -> Option<&mut T> {
        (self.error_mut() as &mut dyn Any).downcast_mut()
    }

    /// # Safety
    /// T must be the exact concrete type of the owned error.
    #[inline]
    pub(crate) unsafe fn downcast_error_ref_unchecked<T: 'static>(&self) -> &T {
        debug_assert!(self.is_error_type::<T>());
        // SAFETY: the caller guarantees the exact type; lifetime follows self.
        unsafe { &*self.error.as_ptr().cast::<T>() }
    }

    /// # Safety
    /// T must be the exact concrete type of the owned error.
    #[inline]
    pub(crate) unsafe fn downcast_error_mut_unchecked<T: 'static>(&mut self) -> &mut T {
        debug_assert!(self.is_error_type::<T>());
        // SAFETY: exact type and exclusive access, with lifetime following self.
        unsafe { &mut *self.error.as_ptr().cast::<T>() }
    }
}

impl Drop for ZstError {
    fn drop(&mut self) {
        // SAFETY: this owner holds one live ZST of the type in its metadata.
        // Extracting the value suppresses this drop; otherwise it runs once.
        unsafe {
            self.error.as_ptr().drop_in_place();
        }
    }
}

// SAFETY: constructors only accept Send + Sync errors and exclusively own the
// logical ZST value. Mutable references require exclusive access to the owner.
unsafe impl Send for ZstError {}
unsafe impl Sync for ZstError {}
