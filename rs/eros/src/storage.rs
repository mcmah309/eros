#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "context")]
use alloc::vec::Vec;
use core::{
    any::Any,
    cell::UnsafeCell,
    mem::{ManuallyDrop, MaybeUninit},
    ptr::NonNull,
};

#[cfg(feature = "context")]
use crate::ContextFrame;
use crate::SendSyncError;

#[inline]
const fn fits_inline<T>() -> bool {
    size_of::<T>() == 0
        || (size_of::<T>() <= size_of::<usize>() && align_of::<T>() <= align_of::<usize>())
}

/// Owns an inline error or a thin pointer to a boxed concrete error. The word
/// is never read as an integer: MaybeUninit preserves padding and provenance
/// when the owner moves. UnsafeCell permits the stored T's interior mutability.
struct StoredError {
    data: UnsafeCell<MaybeUninit<usize>>,
    ops: &'static ErrorOps,
}

struct ErrorOps {
    pointer: unsafe fn(*mut MaybeUninit<usize>) -> *mut dyn SendSyncError,
    drop: unsafe fn(*mut MaybeUninit<usize>),
    #[cfg(feature = "alloc")]
    into_box: unsafe fn(*mut MaybeUninit<usize>) -> Box<dyn SendSyncError>,
}

struct OpsFor<T>(core::marker::PhantomData<T>);
impl<T: SendSyncError> OpsFor<T> {
    const OPS: ErrorOps = ErrorOps {
        pointer: error_pointer::<T>,
        drop: drop_error::<T>,
        #[cfg(feature = "alloc")]
        into_box: box_error::<T>,
    };
}

/// # Safety
/// The word must belong to a live StoredError constructed for exactly T.
#[inline]
unsafe fn concrete_pointer<T>(data: *mut MaybeUninit<usize>) -> *mut T {
    if size_of::<T>() == 0 {
        // ZSTs may be more aligned than our word. They have no payload bytes,
        // so their logical owned value uses a freshly aligned dangling pointer.
        NonNull::<T>::dangling().as_ptr()
    } else if fits_inline::<T>() {
        data.cast::<T>()
    } else {
        #[cfg(feature = "alloc")]
        // SAFETY: construction stored the thin Box<T> pointer in this word.
        unsafe {
            data.cast::<*mut T>().read()
        }
        #[cfg(not(feature = "alloc"))]
        unreachable!("a non-inline error cannot be constructed without alloc")
    }
}

/// # Safety
/// The word must contain a live error of exactly T; access to the returned
/// pointer must obey the borrow of the owner from which data was obtained.
#[inline]
unsafe fn error_pointer<T: SendSyncError>(data: *mut MaybeUninit<usize>) -> *mut dyn SendSyncError {
    // The coercion supplies Rust's real trait-object metadata without relying
    // on a vtable layout or storing a pointer into our movable inline word.
    unsafe { concrete_pointer::<T>(data) }
}

/// # Safety
/// The caller must own the live stored T and never access it again afterwards.
#[inline]
unsafe fn drop_error<T: SendSyncError>(data: *mut MaybeUninit<usize>) {
    let pointer = unsafe { concrete_pointer::<T>(data) };
    if fits_inline::<T>() {
        // SAFETY: the supplied value established validity, even for ZSTs.
        unsafe { pointer.drop_in_place() };
    } else {
        #[cfg(feature = "alloc")]
        // SAFETY: recover the original Box<T>, dropping T and freeing its heap.
        unsafe {
            drop(Box::from_raw(pointer));
        }
    }
}

/// # Safety
/// The caller must exclusively own the live T and suppress StoredError's drop.
#[cfg(feature = "alloc")]
#[inline]
unsafe fn box_error<T: SendSyncError>(data: *mut MaybeUninit<usize>) -> Box<dyn SendSyncError> {
    let pointer = unsafe { concrete_pointer::<T>(data) };
    if fits_inline::<T>() {
        // SAFETY: transfer the inline value into a new box. Box::new does not
        // allocate for ZSTs, including those with nontrivial alignment.
        Box::new(unsafe { pointer.read() })
    } else {
        // SAFETY: transfer the original allocation without moving its payload.
        unsafe { Box::from_raw(pointer) }
    }
}

impl StoredError {
    #[inline]
    fn new<T: SendSyncError>(value: T) -> Self {
        #[cfg(not(feature = "alloc"))]
        const {
            assert!(
                size_of::<T>() <= size_of::<usize>(),
                "T must fit in one pointer-sized word when the alloc feature is disabled"
            );
            assert!(
                size_of::<T>() == 0 || align_of::<T>() <= align_of::<usize>(),
                "T's alignment must fit the inline word when the alloc feature is disabled"
            );
        }
        let data = UnsafeCell::new(MaybeUninit::<usize>::uninit());
        if size_of::<T>() == 0 {
            // Transfer logical ownership. No bytes need to be written, and
            // later operations obtain a pointer with T's alignment.
            core::mem::forget(value);
        } else if fits_inline::<T>() {
            // SAFETY: T fits the word's size and alignment. The word is not
            // initialized/read as usize, so T may have padding or references.
            unsafe { data.get().cast::<T>().write(value) };
        } else {
            #[cfg(feature = "alloc")]
            // SAFETY: a thin pointer fits our word. It is always accessed as a
            // pointer, preserving its provenance through moves of this owner.
            unsafe {
                data.get()
                    .cast::<*mut T>()
                    .write(Box::into_raw(Box::new(value)));
            }
            #[cfg(not(feature = "alloc"))]
            unreachable!("the inline size/alignment assertion rejects T")
        }
        Self {
            data,
            ops: &OpsFor::<T>::OPS,
        }
    }

    #[inline]
    fn error(&self) -> &dyn SendSyncError {
        // SAFETY: ops matches the live T. The temporary trait-object pointer
        // points to its current address and the borrow is bounded by self.
        unsafe { &*(self.ops.pointer)(self.data.get()) }
    }

    #[inline]
    fn error_mut(&mut self) -> &mut dyn SendSyncError {
        // SAFETY: exclusive access to the owned error, at its current address.
        unsafe { &mut *(self.ops.pointer)(self.data.get()) }
    }

    /// # Safety
    /// T must be the exact concrete type of the owned error.
    #[inline]
    unsafe fn take<T: 'static>(self) -> T {
        debug_assert!((self.error() as &dyn Any).is::<T>());
        let owner = ManuallyDrop::new(self);
        let pointer = unsafe { (owner.ops.pointer)(owner.data.get()).cast::<T>() };
        if fits_inline::<T>() {
            // SAFETY: reading transfers ownership; ManuallyDrop suppresses the
            // old root destructor. No references into the owner remain live.
            unsafe { pointer.read() }
        } else {
            #[cfg(feature = "alloc")]
            // SAFETY: moving out of the original Box deallocates it without
            // dropping the extracted T. The caller guarantees the exact type.
            unsafe {
                *Box::from_raw(pointer)
            }
            #[cfg(not(feature = "alloc"))]
            unreachable!("a non-inline error cannot be constructed without alloc")
        }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn into_box(self) -> Box<dyn SendSyncError> {
        let owner = ManuallyDrop::new(self);
        // SAFETY: we transfer exclusive ownership and suppress the old drop.
        unsafe { (owner.ops.into_box)(owner.data.get()) }
    }
}

impl Drop for StoredError {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: construction selected these operations for our one live T.
        unsafe { (self.ops.drop)(self.data.get()) };
    }
}

// SAFETY: construction only accepts Send + Sync errors. UnsafeCell hides their
// storage representation, but every borrow still enforces T's aliasing rules.
unsafe impl Send for StoredError {}
unsafe impl Sync for StoredError {}

/// Diagnostic fields live alongside the root so an inline root needs no heap
/// container. Context and backtrace may still allocate independently.
pub(crate) struct ErrorUnionInner {
    error: StoredError,
    #[cfg(feature = "backtrace")]
    pub(crate) backtrace: std::backtrace::Backtrace,
    #[cfg(feature = "context")]
    pub(crate) context: Vec<ContextFrame>,
    #[cfg(feature = "location")]
    pub(crate) location: &'static core::panic::Location<'static>,
}

pub(crate) struct ErrorParts<T> {
    pub(crate) error: T,
    #[cfg(feature = "backtrace")]
    pub(crate) backtrace: std::backtrace::Backtrace,
    #[cfg(feature = "context")]
    pub(crate) context: Vec<ContextFrame>,
    #[cfg(feature = "location")]
    pub(crate) location: &'static core::panic::Location<'static>,
}

impl ErrorUnionInner {
    #[cfg_attr(feature = "location", track_caller)]
    pub(crate) fn new<T: SendSyncError>(value: T) -> Self {
        Self::new_from_parts(
            value,
            #[cfg(feature = "backtrace")]
            std::backtrace::Backtrace::capture(),
            #[cfg(feature = "context")]
            Vec::new(),
            #[cfg(feature = "location")]
            core::panic::Location::caller(),
        )
    }

    #[inline]
    pub(crate) fn new_from_parts<T: SendSyncError>(
        value: T,
        #[cfg(feature = "backtrace")] backtrace: std::backtrace::Backtrace,
        #[cfg(feature = "context")] context: Vec<ContextFrame>,
        #[cfg(feature = "location")] location: &'static core::panic::Location<'static>,
    ) -> Self {
        Self {
            error: StoredError::new(value),
            #[cfg(feature = "backtrace")]
            backtrace,
            #[cfg(feature = "context")]
            context,
            #[cfg(feature = "location")]
            location,
        }
    }

    #[inline]
    pub(crate) fn error(&self) -> &dyn SendSyncError {
        self.error.error()
    }
    #[inline]
    pub(crate) fn error_mut(&mut self) -> &mut dyn SendSyncError {
        self.error.error_mut()
    }

    #[inline]
    pub(crate) fn is_error_type<T: 'static>(&self) -> bool {
        (self.error() as &dyn Any).is::<T>()
    }

    /// # Safety
    /// T must be the exact concrete type of the owned error.
    #[inline]
    pub(crate) unsafe fn downcast_error_unchecked<T: 'static>(self) -> T {
        let error;
        {
            let parts = unsafe { self.downcast_error_unchecked_with_parts::<T>() };
            // Drop metadata while the root still has a local owner, before
            // moving it into the return place. This also handles a panic in a
            // context destructor without leaking the extracted root.
            error = parts.error;
        }
        error
    }

    /// # Safety
    /// T must be the exact concrete type of the owned error.
    #[inline]
    pub(crate) unsafe fn downcast_error_unchecked_with_parts<T: 'static>(self) -> ErrorParts<T> {
        ErrorParts {
            error: unsafe { self.error.take::<T>() },
            #[cfg(feature = "backtrace")]
            backtrace: self.backtrace,
            #[cfg(feature = "context")]
            context: self.context,
            #[cfg(feature = "location")]
            location: self.location,
        }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    pub(crate) fn into_boxed_parts(self) -> ErrorParts<Box<dyn SendSyncError>> {
        ErrorParts {
            error: self.error.into_box(),
            #[cfg(feature = "backtrace")]
            backtrace: self.backtrace,
            #[cfg(feature = "context")]
            context: self.context,
            #[cfg(feature = "location")]
            location: self.location,
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
        // SAFETY: exact type, and the reference lifetime follows self.
        unsafe { &*(self.error() as *const dyn SendSyncError).cast::<T>() }
    }

    /// # Safety
    /// T must be the exact concrete type of the owned error.
    #[inline]
    pub(crate) unsafe fn downcast_error_mut_unchecked<T: 'static>(&mut self) -> &mut T {
        debug_assert!(self.is_error_type::<T>());
        // SAFETY: exact type and exclusive access, with lifetime following self.
        unsafe { &mut *(self.error_mut() as *mut dyn SendSyncError).cast::<T>() }
    }
}
