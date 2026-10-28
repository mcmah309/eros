#[cfg(feature = "alloc")]
use super::{ChildVisitor, visit_batch};
use super::{ErrorDiagnostics, ErrorParts};
use crate::SendSyncError;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::{
    any::TypeId,
    cell::UnsafeCell,
    mem::{ManuallyDrop, MaybeUninit},
    ptr::NonNull,
};

#[inline]
const fn fits_inline<T>() -> bool {
    size_of::<T>() == 0
        || (size_of::<T>() <= size_of::<usize>() && align_of::<T>() <= align_of::<usize>())
}

/// Without root diagnostics, the first word holds a value or Box<T> pointer;
/// the second word selects static operations. Without alloc, the optional
/// caller location occupies a third word. No discriminant is necessary.
pub(crate) struct Storage {
    owner: Owner,
    diagnostics: ErrorDiagnostics,
}
// Invariant: ops matches the initialized payload's concrete type. data holds
// that value inline, or the sole Box<T> pointer when it does not fit inline.
// ZST payloads use an aligned dangling pointer and occupy no bytes in data.
pub(crate) struct Owner {
    data: UnsafeCell<MaybeUninit<usize>>,
    ops: &'static Ops,
}
// Each operation requires data from a live Owner for its concrete T. Borrowing
// operations preserve the owner's lifetime and aliasing rules; drop and into_box
// require exclusive ownership and consume the payload exactly once.
struct Ops {
    type_id: TypeId,
    pointer: unsafe fn(*mut MaybeUninit<usize>) -> *mut dyn SendSyncError,
    drop: unsafe fn(*mut MaybeUninit<usize>),
    #[cfg(feature = "alloc")]
    into_box: unsafe fn(*mut MaybeUninit<usize>) -> Box<dyn SendSyncError>,
    #[cfg(feature = "alloc")]
    children: Option<ChildVisitor>,
}
struct OpsFor<T>(core::marker::PhantomData<T>);
impl<T: SendSyncError> OpsFor<T> {
    const OPS: Ops = Ops {
        type_id: TypeId::of::<T>(),
        pointer: error_pointer::<T>,
        drop: drop_error::<T>,
        #[cfg(feature = "alloc")]
        into_box: box_error::<T>,
        #[cfg(feature = "alloc")]
        children: None,
    };
}
#[cfg(feature = "alloc")]
impl<E: crate::TypeSet> OpsFor<crate::ErrorUnionVec<E>> {
    const WITH_CHILDREN: Ops = Ops {
        children: Some(visit_batch::<E>),
        ..Self::OPS
    };
}

/// # Safety
/// `data` must point to the data field of a live Owner constructed for exactly T,
/// whose payload has not been moved out or dropped. The returned pointer must
/// only be used within that owner's lifetime and under its borrowing rules.
#[inline]
unsafe fn concrete_pointer<T>(data: *mut MaybeUninit<usize>) -> *mut T {
    if size_of::<T>() == 0 {
        NonNull::<T>::dangling().as_ptr()
    } else if fits_inline::<T>() {
        data.cast()
    } else {
        #[cfg(feature = "alloc")]
        {
            // SAFETY: the caller guarantees an initialized Owner for T. This
            // branch stores a Box<T> pointer in a pointer-sized, aligned slot.
            unsafe { data.cast::<*mut T>().read() }
        }
        #[cfg(not(feature = "alloc"))]
        unreachable!("construction rejects errors that do not fit inline")
    }
}
/// # Safety
/// `data` and uses of the returned pointer must satisfy concrete_pointer::<T>'s
/// initialization, lifetime, and borrowing requirements.
unsafe fn error_pointer<T: SendSyncError>(data: *mut MaybeUninit<usize>) -> *mut dyn SendSyncError {
    // SAFETY: the caller provides a live Owner for T as concrete_pointer requires.
    unsafe { concrete_pointer::<T>(data) }
}
/// # Safety
/// `data` must satisfy concrete_pointer::<T>'s requirements, with exclusive
/// access to the payload. The payload must not be accessed or dropped afterward.
unsafe fn drop_error<T: SendSyncError>(data: *mut MaybeUninit<usize>) {
    // SAFETY: the caller guarantees a live, initialized Owner for T.
    let pointer = unsafe { concrete_pointer::<T>(data) };
    if fits_inline::<T>() {
        // SAFETY: the caller grants exclusive access and exactly one drop. The
        // pointer is aligned and initialized, including for an aligned ZST.
        unsafe {
            pointer.drop_in_place();
        }
    } else {
        #[cfg(feature = "alloc")]
        // SAFETY: this branch owns the original Box<T> allocation. The caller
        // relinquishes the payload, so reconstructing and dropping it is unique.
        unsafe {
            drop(Box::from_raw(pointer));
        }
        #[cfg(not(feature = "alloc"))]
        unreachable!("construction rejects errors that do not fit inline");
    }
}
#[cfg(feature = "alloc")]
/// # Safety
/// `data` must satisfy concrete_pointer::<T>'s requirements, with exclusive
/// ownership of the payload transferred here. It must not be accessed afterward.
unsafe fn box_error<T: SendSyncError>(data: *mut MaybeUninit<usize>) -> Box<dyn SendSyncError> {
    // SAFETY: the caller guarantees a live, initialized Owner for T.
    let pointer = unsafe { concrete_pointer::<T>(data) };
    if fits_inline::<T>() {
        // SAFETY: the caller transfers the initialized payload exactly once;
        // the old owner will not drop it after this read.
        Box::new(unsafe { pointer.read() })
    } else {
        // SAFETY: the pointer came from Box::into_raw for T, and the caller
        // transfers that allocation's sole ownership to the returned Box.
        unsafe { Box::from_raw(pointer) }
    }
}

impl Owner {
    #[inline]
    fn new<T: SendSyncError>(value: T) -> Self {
        #[cfg(not(feature = "alloc"))]
        const {
            assert!(
                fits_inline::<T>(),
                "T must fit in one usize in size and alignment when the alloc feature is disabled (zero-sized errors support any alignment)"
            );
        }
        let data = UnsafeCell::new(MaybeUninit::<usize>::uninit());
        if size_of::<T>() == 0 {
            core::mem::forget(value);
        } else if fits_inline::<T>() {
            // SAFETY: fits_inline guarantees enough space and alignment for T,
            // and data is exclusively owned. The word is never read as usize,
            // preserving padding and pointer provenance. UnsafeCell permits
            // the stored T's interior mutability.
            unsafe {
                data.get().cast::<T>().write(value);
            }
        } else {
            #[cfg(feature = "alloc")]
            // SAFETY: the slot has a pointer's size and alignment, and data is
            // exclusively owned. Box::into_raw transfers ownership to Owner;
            // ops for T will recover the same Box<T> when it is dropped.
            unsafe {
                data.get()
                    .cast::<*mut T>()
                    .write(Box::into_raw(Box::new(value)));
            }
            #[cfg(not(feature = "alloc"))]
            unreachable!("construction rejects errors that do not fit inline");
        }
        Self {
            data,
            ops: &OpsFor::<T>::OPS,
        }
    }
    #[inline]
    pub(crate) fn error(&self) -> &dyn SendSyncError {
        // SAFETY: ops matches the initialized payload, and the returned shared
        // reference is tied to self. UnsafeCell preserves interior mutability.
        unsafe { &*(self.ops.pointer)(self.data.get()) }
    }
    #[inline]
    pub(crate) fn error_mut(&mut self) -> &mut dyn SendSyncError {
        // SAFETY: ops matches the initialized payload, and &mut self gives
        // exclusive access for the lifetime of the returned reference.
        unsafe { &mut *(self.ops.pointer)(self.data.get()) }
    }
    #[inline]
    pub(crate) fn is_error_type<T: 'static>(&self) -> bool {
        self.ops.type_id == TypeId::of::<T>()
    }
    /// # Safety
    /// T must be the exact concrete type of the stored error.
    #[inline]
    pub(crate) unsafe fn concrete_ref<T>(&self) -> &T {
        // SAFETY: the caller guarantees T. Owner holds a live payload, and the
        // reference is limited to the shared borrow of self.
        unsafe { &*concrete_pointer::<T>(self.data.get()) }
    }
    /// # Safety
    /// T must be the exact concrete type of the stored error.
    #[inline]
    pub(crate) unsafe fn concrete_mut<T>(&mut self) -> &mut T {
        // SAFETY: the caller guarantees T. Owner holds a live payload, and
        // &mut self gives exclusive access for the returned reference.
        unsafe { &mut *concrete_pointer::<T>(self.data.get()) }
    }
    /// # Safety
    /// T must be the exact concrete type of the stored error.
    #[inline]
    pub(super) unsafe fn into_error<T>(self) -> T {
        let owner = ManuallyDrop::new(self);
        // SAFETY: the caller guarantees T, and the consumed owner still holds
        // its initialized payload until the following ownership transfer.
        let pointer = unsafe { concrete_pointer::<T>(owner.data.get()) };
        let error = if fits_inline::<T>() {
            // SAFETY: the payload is aligned and initialized. ManuallyDrop
            // prevents its old owner from dropping the value moved out here.
            unsafe { pointer.read() }
        } else {
            #[cfg(feature = "alloc")]
            {
                // SAFETY: this is the original Box<T> pointer. Consuming the
                // owner and suppressing its destructor transfers ownership once.
                *unsafe { Box::from_raw(pointer) }
            }
            #[cfg(not(feature = "alloc"))]
            unreachable!("construction rejects errors that do not fit inline")
        };
        error
    }
    #[cfg(feature = "alloc")]
    pub(crate) fn into_boxed(self) -> Box<dyn SendSyncError> {
        let owner = ManuallyDrop::new(self);
        // SAFETY: ops matches the live payload. Consuming self grants exclusive
        // ownership, and ManuallyDrop prevents a second drop after conversion.
        unsafe { (owner.ops.into_box)(owner.data.get()) }
    }
    /// # Safety
    /// T must be the exact concrete type of the stored error.
    #[inline]
    pub(super) unsafe fn map_single<T: SendSyncError, U: SendSyncError>(
        self,
        f: impl FnOnce(T) -> U,
    ) -> Self {
        #[cfg(feature = "alloc")]
        if const {
            !fits_inline::<T>()
                && !fits_inline::<U>()
                && size_of::<T>() == size_of::<U>()
                && align_of::<T>() == align_of::<U>()
        } {
            let owner = ManuallyDrop::new(self);
            // SAFETY: the caller guarantees T, and this branch stores a Box<T>.
            // ManuallyDrop transfers sole ownership to the guard. MaybeUninit<T>
            // has T's layout and lets the guard free the allocation on unwind
            // without dropping the payload taken by f.
            let allocation = unsafe {
                Box::from_raw(concrete_pointer::<T>(owner.data.get()).cast::<MaybeUninit<T>>())
            };
            // SAFETY: the initialized T is exclusively owned, and the
            // MaybeUninit guard will not drop it after it moves into f.
            let value = f(unsafe { allocation.as_ptr().read() });
            let raw = Box::into_raw(allocation).cast::<U>();
            // SAFETY: the branch verifies identical T and U sizes and
            // alignments, and the old payload has been moved out.
            unsafe {
                raw.write(value);
            }
            let data = UnsafeCell::new(MaybeUninit::<usize>::uninit());
            // SAFETY: the slot has a pointer's size and alignment. Ownership of
            // the initialized U allocation transfers to data with U's ops.
            unsafe {
                data.get().cast::<*mut U>().write(raw);
            }
            return Self {
                data,
                ops: &OpsFor::<U>::OPS,
            };
        }
        // SAFETY: the caller guarantees that the owned payload's type is T.
        Self::new(f(unsafe { self.into_error::<T>() }))
    }
    #[cfg(feature = "alloc")]
    pub(super) fn children(&self) -> Option<ChildVisitor> {
        self.ops.children
    }
    /// # Safety
    /// The stored concrete type must be ErrorUnionVec<E>.
    #[cfg(feature = "alloc")]
    pub(super) unsafe fn enable_children<E: crate::TypeSet>(&mut self) {
        self.ops = &OpsFor::<crate::ErrorUnionVec<E>>::WITH_CHILDREN;
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        // SAFETY: ops matches the live payload, and the destructor has exclusive
        // access. Ownership transfers to the drop operation exactly once.
        unsafe {
            (self.ops.drop)(self.data.get());
        }
    }
}
// SAFETY: constructors accept only Send errors. Moving Owner transfers its
// payload, and its static ops contains no thread-local state.
unsafe impl Send for Owner {}
// SAFETY: constructors accept only Sync errors. Shared access yields only &T;
// UnsafeCell permits T's interior mutability without granting mutable access.
unsafe impl Sync for Owner {}

impl Storage {
    #[inline]
    pub(super) fn new<T: SendSyncError>(value: T, diagnostics: ErrorDiagnostics) -> Self {
        Self {
            owner: Owner::new(value),
            diagnostics,
        }
    }
    #[cfg(feature = "error_location")]
    pub(super) const fn diagnostics(&self) -> &ErrorDiagnostics {
        &self.diagnostics
    }
    pub(super) fn into_owner_parts(self) -> ErrorParts<Owner> {
        ErrorParts {
            error: self.owner,
            diagnostics: self.diagnostics,
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
    pub(super) unsafe fn into_parts<T>(self) -> ErrorParts<T> {
        ErrorParts {
            // SAFETY: the caller guarantees T, and consuming Storage transfers
            // its payload's sole owner without changing the concrete type.
            error: unsafe { self.owner.into_error::<T>() },
            diagnostics: self.diagnostics,
        }
    }
    /// # Safety
    /// T must be the exact concrete type of the stored error.
    pub(super) unsafe fn map_single<T: SendSyncError, U: SendSyncError>(
        self,
        f: impl FnOnce(T) -> U,
    ) -> Self {
        Self {
            // SAFETY: the caller guarantees the concrete type required by
            // Owner::map_single, and self transfers sole payload ownership.
            owner: unsafe { self.owner.map_single::<T, U>(f) },
            diagnostics: self.diagnostics,
        }
    }
    #[cfg(feature = "alloc")]
    pub(super) fn children(&self) -> Option<ChildVisitor> {
        self.owner.children()
    }
    /// # Safety
    /// The stored concrete type must be ErrorUnionVec<E>.
    #[cfg(feature = "alloc")]
    pub(super) unsafe fn enable_children<E: crate::TypeSet>(&mut self) {
        // SAFETY: the caller guarantees the payload is exactly ErrorUnionVec<E>,
        // as required when replacing its operations with WITH_CHILDREN.
        unsafe {
            self.owner.enable_children::<E>();
        }
    }
}
