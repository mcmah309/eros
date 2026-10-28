use super::{ChildVisitor, ErrorDiagnostics, ErrorParts, visit_batch};
use crate::SendSyncError;
use alloc::boxed::Box;
use core::{
    any::Any,
    mem::{ManuallyDrop, MaybeUninit},
    ptr,
};

// Implementations require sole ownership of a HeapError<T> allocation whose
// payload matches T and whose diagnostics have already been transferred out.
type IntoBox = unsafe fn(*mut ()) -> Box<dyn SendSyncError>;

// Rust's native fat pointer supplies the destructor, size, and alignment.
// Diagnostics and the trailing error always share one allocation.
// Invariant: diagnostics remain initialized until Storage is dropped or consumed.
pub(crate) struct Storage {
    owner: Owner,
}

// Diagnostics have moved out before this payload-only owner is exposed.
// Invariant: the box owns a live payload, and into_box matches its concrete type.
pub(crate) struct Owner {
    allocation: Box<HeapError<dyn SendSyncError>>,
}
// Casts recover the original HeapError<T>; allocation reuse initializes every
// field at the replacement type's own offsets.
struct HeapError<T: ?Sized> {
    diagnostics: MaybeUninit<ErrorDiagnostics>,
    into_box: IntoBox,
    children: Option<ChildVisitor>,
    value: T,
}

/// # Safety
/// `data` must come from Box::into_raw for a HeapError<T>, with exclusive
/// ownership transferred to this call. Its payload must still be initialized,
/// and ownership of its diagnostics must already have been transferred out.
unsafe fn take_value<T>(data: *mut ()) -> T {
    // SAFETY: the caller transfers the original Box allocation for HeapError<T>.
    // Moving its payload out frees the allocation; MaybeUninit skips diagnostics.
    let HeapError { value, .. } = *unsafe { Box::from_raw(data.cast::<HeapError<T>>()) };
    value
}
/// # Safety
/// `data` must satisfy take_value::<T>'s ownership and initialization requirements.
unsafe fn box_error<T: SendSyncError>(data: *mut ()) -> Box<dyn SendSyncError> {
    // SAFETY: the caller supplies exactly the allocation required by take_value.
    Box::new(unsafe { take_value::<T>(data) })
}
impl Storage {
    #[inline]
    pub(super) fn new<T: SendSyncError>(value: T, diagnostics: ErrorDiagnostics) -> Self {
        Self {
            owner: Owner {
                allocation: Box::new(HeapError {
                    diagnostics: MaybeUninit::new(diagnostics),
                    into_box: box_error::<T>,
                    children: None,
                    value,
                }),
            },
        }
    }
    pub(super) const fn diagnostics(&self) -> &ErrorDiagnostics {
        // SAFETY: Storage keeps diagnostics initialized throughout this borrow.
        unsafe { self.owner.allocation.diagnostics.assume_init_ref() }
    }
    #[cfg(any(feature = "context", feature = "backtrace"))]
    pub(super) fn diagnostics_mut(&mut self) -> &mut ErrorDiagnostics {
        // SAFETY: Storage keeps diagnostics initialized, and &mut self excludes
        // other access to them for the lifetime of the returned reference.
        unsafe { self.owner.allocation.diagnostics.assume_init_mut() }
    }
    pub(super) fn into_owner_parts(self) -> ErrorParts<Owner> {
        let storage = ManuallyDrop::new(self);
        // SAFETY: Storage guarantees initialized diagnostics, and consuming self
        // gives exclusive ownership. ManuallyDrop prevents either value from
        // being dropped here, so both can transfer exactly once into ErrorParts.
        // Owner's box skips the MaybeUninit diagnostics when dropped. Callers
        // need not establish any additional invariants, so this function is safe.
        unsafe {
            let diagnostics = storage.owner.allocation.diagnostics.assume_init_read();
            ErrorParts {
                error: ptr::read(&storage.owner),
                diagnostics,
            }
        }
    }
    pub(super) fn error(&self) -> &dyn SendSyncError {
        self.owner.error()
    }
    pub(super) fn error_mut(&mut self) -> &mut dyn SendSyncError {
        self.owner.error_mut()
    }
    pub(super) fn is_error_type<T: 'static>(&self) -> bool {
        self.owner.is_error_type::<T>()
    }
    /// # Safety
    /// T must be the exact concrete type of the stored error.
    pub(super) unsafe fn concrete_ref<T>(&self) -> &T {
        // SAFETY: the caller guarantees the concrete type required by Owner.
        unsafe { self.owner.concrete_ref::<T>() }
    }
    /// # Safety
    /// T must be the exact concrete type of the stored error.
    pub(super) unsafe fn concrete_mut<T>(&mut self) -> &mut T {
        // SAFETY: the caller guarantees the concrete type required by Owner;
        // &mut self gives the owner exclusive access to the payload.
        unsafe { self.owner.concrete_mut::<T>() }
    }
    /// # Safety
    /// T must be the exact concrete type of the stored error.
    #[inline]
    pub(super) unsafe fn into_parts<T>(self) -> ErrorParts<T> {
        let parts = self.into_owner_parts();
        ErrorParts {
            // SAFETY: the caller guarantees T, and into_owner_parts transfers
            // the payload's sole owner without changing its concrete type.
            error: unsafe { parts.error.into_error::<T>() },
            diagnostics: parts.diagnostics,
        }
    }
    /// # Safety
    /// T must be the exact concrete type of the stored error.
    #[inline]
    pub(super) unsafe fn map_single<T: SendSyncError, U: SendSyncError>(
        self,
        f: impl FnOnce(T) -> U,
    ) -> Self {
        if const {
            size_of::<HeapError<T>>() == size_of::<HeapError<U>>()
                && align_of::<HeapError<T>>() == align_of::<HeapError<U>>()
        } {
            // Until f succeeds, parts owns diagnostics and this box guards the
            // allocation. The moved-out payload cannot be dropped twice.
            let parts = self.into_owner_parts();
            // SAFETY: the caller guarantees T. Box::into_raw transfers the sole
            // allocation. Wrapping the entire HeapError<T> in MaybeUninit keeps
            // its exact size and alignment, allowing the guard to free storage
            // after the payload moves out without relying on field ordering.
            let allocation = unsafe {
                Box::from_raw(
                    Box::into_raw(parts.error.allocation).cast::<MaybeUninit<HeapError<T>>>(),
                )
            };
            // SAFETY: the original HeapError<T> layout locates the initialized,
            // exclusively owned payload. Raw field access avoids borrowing a
            // partially initialized struct, and the guard skips its destructor.
            let value = f(unsafe { ptr::addr_of!((*allocation.as_ptr()).value).read() });
            let raw = Box::into_raw(allocation).cast::<HeapError<U>>();
            // SAFETY: the branch checks the complete allocation's size and
            // alignment for U. We initialize every HeapError<U> field before
            // rebuilding its sole Box with U's trait-object metadata.
            unsafe {
                ptr::addr_of_mut!((*raw).value).write(value);
                ptr::addr_of_mut!((*raw).diagnostics).write(MaybeUninit::new(parts.diagnostics));
                ptr::addr_of_mut!((*raw).into_box).write(box_error::<U>);
                ptr::addr_of_mut!((*raw).children).write(None);
                // Unsizing supplies U's real trait metadata for ordinary Box
                // borrowing and destruction; no custom ownership header.
                Self {
                    owner: Owner {
                        allocation: Box::from_raw(raw),
                    },
                }
            }
        } else {
            // SAFETY: the caller guarantees the concrete type is T.
            let parts = unsafe { self.into_parts::<T>() };
            Self::new(f(parts.error), parts.diagnostics)
        }
    }
    pub(super) fn children(&self) -> Option<ChildVisitor> {
        self.owner.allocation.children
    }
    /// # Safety
    /// The stored concrete type must be ErrorUnionVec<E>.
    pub(super) unsafe fn enable_children<E: crate::TypeSet>(&mut self) {
        self.owner.allocation.children = Some(visit_batch::<E>);
    }
}

impl Drop for Storage {
    fn drop(&mut self) {
        // SAFETY: only Storage owns initialized diagnostics, and its destructor
        // runs once. Owner's box skips diagnostics but still drops the payload
        // and frees the allocation if diagnostics cleanup panics.
        unsafe {
            self.owner.allocation.diagnostics.assume_init_drop();
        }
    }
}

impl Owner {
    #[inline]
    pub(crate) fn error(&self) -> &dyn SendSyncError {
        &self.allocation.value
    }
    #[inline]
    pub(crate) fn error_mut(&mut self) -> &mut dyn SendSyncError {
        &mut self.allocation.value
    }
    #[inline]
    pub(crate) fn is_error_type<T: 'static>(&self) -> bool {
        (self.error() as &dyn Any).is::<T>()
    }
    /// # Safety
    /// T must be the exact concrete type of the stored error.
    #[inline]
    pub(crate) unsafe fn concrete_ref<T>(&self) -> &T {
        let raw = ptr::from_ref(&*self.allocation).cast::<HeapError<T>>();
        // SAFETY: the caller guarantees T, so the cast recovers the allocation's
        // original HeapError<T> layout and payload offset. The reference cannot
        // outlive the shared borrow.
        unsafe { &*ptr::addr_of!((*raw).value) }
    }
    /// # Safety
    /// T must be the exact concrete type of the stored error.
    #[inline]
    pub(crate) unsafe fn concrete_mut<T>(&mut self) -> &mut T {
        let raw = ptr::from_mut(&mut *self.allocation).cast::<HeapError<T>>();
        // SAFETY: the caller guarantees T, so the cast recovers the allocation's
        // original HeapError<T> layout and payload offset. &mut self excludes
        // access through other borrows.
        unsafe { &mut *ptr::addr_of_mut!((*raw).value) }
    }

    /// # Safety
    /// T must be the exact concrete type of the stored error.
    pub(super) unsafe fn into_error<T>(self) -> T {
        // SAFETY: the caller guarantees T; consuming Owner transfers its sole
        // Box allocation, whose diagnostics have already been moved out.
        unsafe { take_value::<T>(Box::into_raw(self.allocation).cast()) }
    }
    pub(crate) fn into_boxed(self) -> Box<dyn SendSyncError> {
        let into_box = self.allocation.into_box;
        // SAFETY: into_box matches the live payload's type. Consuming Owner
        // transfers its sole allocation after diagnostics have been moved out.
        unsafe { into_box(Box::into_raw(self.allocation).cast()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn equal_size_with_different_alignment_requires_replacement() {
        #[derive(Debug)]
        #[repr(align(256))]
        struct Aligned(u8);
        impl core::fmt::Display for Aligned {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
        impl core::error::Error for Aligned {}
        const BYTES: usize = size_of::<HeapError<Aligned>>() - size_of::<HeapError<()>>();
        #[derive(Debug)]
        struct Unaligned([u8; BYTES]);
        impl core::fmt::Display for Unaligned {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0[0])
            }
        }
        impl core::error::Error for Unaligned {}
        assert_eq!(
            size_of::<HeapError<Unaligned>>(),
            size_of::<HeapError<Aligned>>()
        );
        assert_ne!(
            align_of::<HeapError<Unaligned>>(),
            align_of::<HeapError<Aligned>>()
        );
        let error = crate::storage::ErrorUnionInner::new(Unaligned([73; BYTES]));
        // SAFETY: construction established exactly Unaligned.
        let mapped = unsafe { error.map_single::<Unaligned, Aligned>(|old| Aligned(old.0[0])) };
        let value = mapped.downcast_error_ref::<Aligned>().unwrap();
        assert_eq!((value as *const Aligned).addr() % 256, 0);
        assert_eq!(value.0, 73);
        drop(mapped); // Miri verifies the deallocation alignment as well.
    }
}
