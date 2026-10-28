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
pub(super) struct Storage {
    data: UnsafeCell<MaybeUninit<usize>>,
    ops: &'static Ops,
    diagnostics: ErrorDiagnostics,
}
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
/// data belongs to a live Storage constructed for exactly T.
#[inline]
unsafe fn concrete_pointer<T>(data: *mut MaybeUninit<usize>) -> *mut T {
    if size_of::<T>() == 0 {
        NonNull::<T>::dangling().as_ptr()
    } else if fits_inline::<T>() {
        data.cast()
    } else {
        #[cfg(feature = "alloc")]
        {
            unsafe { data.cast::<*mut T>().read() }
        }
        #[cfg(not(feature = "alloc"))]
        unreachable!("construction rejects errors that do not fit inline")
    }
}
unsafe fn error_pointer<T: SendSyncError>(data: *mut MaybeUninit<usize>) -> *mut dyn SendSyncError {
    unsafe { concrete_pointer::<T>(data) }
}
unsafe fn drop_error<T: SendSyncError>(data: *mut MaybeUninit<usize>) {
    let pointer = unsafe { concrete_pointer::<T>(data) };
    if fits_inline::<T>() {
        unsafe {
            pointer.drop_in_place();
        }
    } else {
        #[cfg(feature = "alloc")]
        unsafe {
            drop(Box::from_raw(pointer));
        }
        #[cfg(not(feature = "alloc"))]
        unreachable!("construction rejects errors that do not fit inline");
    }
}
#[cfg(feature = "alloc")]
unsafe fn box_error<T: SendSyncError>(data: *mut MaybeUninit<usize>) -> Box<dyn SendSyncError> {
    let pointer = unsafe { concrete_pointer::<T>(data) };
    if fits_inline::<T>() {
        Box::new(unsafe { pointer.read() })
    } else {
        unsafe { Box::from_raw(pointer) }
    }
}

impl Storage {
    #[inline]
    pub(super) fn new<T: SendSyncError>(value: T, diagnostics: ErrorDiagnostics) -> Self {
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
            // Preserve padding and pointer provenance: this word is never read
            // as usize. UnsafeCell permits the stored T's interior mutability.
            unsafe {
                data.get().cast::<T>().write(value);
            }
        } else {
            // Ownership transfers to Storage; drop_error recovers this Box<T>.
            #[cfg(feature = "alloc")]
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
            diagnostics,
        }
    }
    #[cfg(feature = "error_location")]
    pub(super) const fn diagnostics(&self) -> &ErrorDiagnostics {
        &self.diagnostics
    }
    pub(super) fn take_diagnostics(&mut self) -> ErrorDiagnostics {
        self.diagnostics.take()
    }
    #[inline]
    pub(super) fn error(&self) -> &dyn SendSyncError {
        unsafe { &*(self.ops.pointer)(self.data.get()) }
    }
    #[inline]
    pub(super) fn error_mut(&mut self) -> &mut dyn SendSyncError {
        unsafe { &mut *(self.ops.pointer)(self.data.get()) }
    }
    #[inline]
    pub(super) fn is_error_type<T: 'static>(&self) -> bool {
        self.ops.type_id == TypeId::of::<T>()
    }
    /// # Safety
    /// T must be the exact concrete type of the stored error.
    #[inline]
    pub(super) unsafe fn concrete_ref<T>(&self) -> &T {
        unsafe { &*concrete_pointer::<T>(self.data.get()) }
    }
    /// # Safety
    /// T must be the exact concrete type of the stored error.
    #[inline]
    pub(super) unsafe fn concrete_mut<T>(&mut self) -> &mut T {
        unsafe { &mut *concrete_pointer::<T>(self.data.get()) }
    }
    /// # Safety
    /// T must be the exact concrete type of the stored error.
    #[inline]
    pub(super) unsafe fn into_parts<T>(self) -> ErrorParts<T> {
        let owner = ManuallyDrop::new(self);
        let pointer = unsafe { concrete_pointer::<T>(owner.data.get()) };
        let error = if fits_inline::<T>() {
            unsafe { pointer.read() }
        } else {
            #[cfg(feature = "alloc")]
            {
                *unsafe { Box::from_raw(pointer) }
            }
            #[cfg(not(feature = "alloc"))]
            unreachable!("construction rejects errors that do not fit inline")
        };
        ErrorParts {
            error,
            diagnostics: unsafe { core::ptr::read(&owner.diagnostics) },
        }
    }
    #[cfg(feature = "alloc")]
    pub(super) fn into_boxed_parts(self) -> ErrorParts<Box<dyn SendSyncError>> {
        let owner = ManuallyDrop::new(self);
        ErrorParts {
            error: unsafe { (owner.ops.into_box)(owner.data.get()) },
            diagnostics: unsafe { core::ptr::read(&owner.diagnostics) },
        }
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
            // The guard frees the allocation on unwind without dropping the T
            // taken by f. MaybeUninit has exactly T's size and alignment.
            let allocation = unsafe {
                Box::from_raw(concrete_pointer::<T>(owner.data.get()).cast::<MaybeUninit<T>>())
            };
            let value = f(unsafe { allocation.as_ptr().read() });
            let raw = Box::into_raw(allocation).cast::<U>();
            unsafe {
                raw.write(value);
            }
            let data = UnsafeCell::new(MaybeUninit::<usize>::uninit());
            unsafe {
                data.get().cast::<*mut U>().write(raw);
            }
            return Self {
                data,
                ops: &OpsFor::<U>::OPS,
                diagnostics: unsafe { core::ptr::read(&owner.diagnostics) },
            };
        }
        let parts = unsafe { self.into_parts::<T>() };
        Self::new(f(parts.error), parts.diagnostics)
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
impl Drop for Storage {
    fn drop(&mut self) {
        unsafe {
            (self.ops.drop)(self.data.get());
        }
    }
}
// Only Send + Sync errors enter storage; borrows preserve T's aliasing rules.
unsafe impl Send for Storage {}
unsafe impl Sync for Storage {}
