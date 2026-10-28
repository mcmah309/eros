use super::{ChildVisitor, ErrorDiagnostics, ErrorParts, visit_batch};
use crate::SendSyncError;
use alloc::boxed::Box;
use core::{any::Any, mem::MaybeUninit, ptr};

type IntoBox = unsafe fn(*mut ()) -> ErrorParts<Box<dyn SendSyncError>>;

// Rust's native fat pointer supplies the destructor, size, and alignment.
// Diagnostics and the trailing error always share one allocation.
pub(super) struct Storage {
    allocation: Box<HeapError<dyn SendSyncError>>,
}
#[repr(C)]
struct HeapError<T: ?Sized> {
    diagnostics: ErrorDiagnostics,
    into_box: IntoBox,
    children: Option<ChildVisitor>,
    value: T,
}

/// # Safety
/// data is the exclusively owned allocation for exactly T.
unsafe fn take_parts<T>(data: *mut ()) -> ErrorParts<T> {
    let HeapError {
        diagnostics, value, ..
    } = *unsafe { Box::from_raw(data.cast::<HeapError<T>>()) };
    ErrorParts {
        error: value,
        diagnostics,
    }
}
unsafe fn box_error<T: SendSyncError>(data: *mut ()) -> ErrorParts<Box<dyn SendSyncError>> {
    let parts = unsafe { take_parts::<T>(data) };
    ErrorParts {
        error: Box::new(parts.error),
        diagnostics: parts.diagnostics,
    }
}
impl Storage {
    #[inline]
    pub(super) fn new<T: SendSyncError>(value: T, diagnostics: ErrorDiagnostics) -> Self {
        Self {
            allocation: Box::new(HeapError {
                diagnostics,
                into_box: box_error::<T>,
                children: None,
                value,
            }),
        }
    }
    pub(super) const fn diagnostics(&self) -> &ErrorDiagnostics {
        &self.allocation.diagnostics
    }
    #[cfg(any(feature = "context", feature = "backtrace"))]
    pub(super) fn diagnostics_mut(&mut self) -> &mut ErrorDiagnostics {
        &mut self.allocation.diagnostics
    }
    pub(super) fn take_diagnostics(&mut self) -> ErrorDiagnostics {
        self.allocation.diagnostics.take()
    }
    #[inline]
    pub(super) fn error(&self) -> &dyn SendSyncError {
        &self.allocation.value
    }
    #[inline]
    pub(super) fn error_mut(&mut self) -> &mut dyn SendSyncError {
        &mut self.allocation.value
    }
    #[inline]
    pub(super) fn is_error_type<T: 'static>(&self) -> bool {
        (self.error() as &dyn Any).is::<T>()
    }
    /// # Safety
    /// T must be the exact concrete type of the stored error.
    #[inline]
    pub(super) unsafe fn concrete_ref<T>(&self) -> &T {
        let raw = ptr::from_ref(&*self.allocation).cast::<HeapError<T>>();
        unsafe { &*ptr::addr_of!((*raw).value) }
    }
    /// # Safety
    /// T must be the exact concrete type of the stored error.
    #[inline]
    pub(super) unsafe fn concrete_mut<T>(&mut self) -> &mut T {
        let raw = ptr::from_mut(&mut *self.allocation).cast::<HeapError<T>>();
        unsafe { &mut *ptr::addr_of_mut!((*raw).value) }
    }
    /// # Safety
    /// T must be the exact concrete type of the stored error.
    #[inline]
    pub(super) unsafe fn into_parts<T>(self) -> ErrorParts<T> {
        unsafe { take_parts::<T>(Box::into_raw(self.allocation).cast()) }
    }
    pub(super) fn into_boxed_parts(self) -> ErrorParts<Box<dyn SendSyncError>> {
        let into_box = self.allocation.into_box;
        unsafe { into_box(Box::into_raw(self.allocation).cast()) }
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
            // Until f succeeds, this box owns diagnostics and the allocation;
            // its uninitialized payload cannot be dropped a second time.
            let allocation = unsafe {
                Box::from_raw(Box::into_raw(self.allocation).cast::<HeapError<MaybeUninit<T>>>())
            };
            let value = f(unsafe { allocation.value.as_ptr().read() });
            let raw = Box::into_raw(allocation).cast::<HeapError<U>>();
            unsafe {
                ptr::addr_of_mut!((*raw).value).write(value);
                ptr::addr_of_mut!((*raw).into_box).write(box_error::<U>);
                ptr::addr_of_mut!((*raw).children).write(None);
                // Unsizing supplies U's real trait metadata for ordinary Box
                // borrowing and destruction; no custom ownership header.
                Self {
                    allocation: Box::from_raw(raw),
                }
            }
        } else {
            let parts = unsafe { self.into_parts::<T>() };
            Self::new(f(parts.error), parts.diagnostics)
        }
    }
    pub(super) fn children(&self) -> Option<ChildVisitor> {
        self.allocation.children
    }
    /// # Safety
    /// The stored concrete type must be ErrorUnionVec<E>.
    pub(super) unsafe fn enable_children<E: crate::TypeSet>(&mut self) {
        self.allocation.children = Some(visit_batch::<E>);
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
