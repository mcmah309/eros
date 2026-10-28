#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "context")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use core::any::Any;
#[cfg(feature = "alloc")]
use core::mem::MaybeUninit;
use core::{any::TypeId, mem::ManuallyDrop, ptr::NonNull};

#[cfg(feature = "context")]
use crate::ContextFrame;
use crate::SendSyncError;

/// A thin owning pointer with alloc; a static operations pointer for a ZST otherwise.
struct StoredError {
    #[cfg(feature = "alloc")]
    allocation: NonNull<ErrorHeader>,
    #[cfg(not(feature = "alloc"))]
    ops: &'static ErrorOps,
}

#[cfg(feature = "alloc")]
#[repr(C)]
struct ErrorHeader {
    ops: &'static ErrorOps,
    diagnostics: ErrorDiagnostics,
}

// The common header is at offset zero, while Rust supplies T's required alignment.
#[cfg(feature = "alloc")]
#[repr(C)]
struct HeapError<T: ?Sized> {
    header: ErrorHeader,
    value: T,
}

struct ErrorOps {
    type_id: TypeId,
    pointer: unsafe fn(*mut ()) -> *mut dyn SendSyncError,
    drop: unsafe fn(*mut ()),
    #[cfg(feature = "alloc")]
    into_box: unsafe fn(*mut ()) -> ErrorParts<Box<dyn SendSyncError>>,
    #[cfg(feature = "alloc")]
    children: Option<ChildVisitor>,
}

#[cfg(feature = "alloc")]
type ChildVisitor = for<'a> fn(&'a dyn SendSyncError, &mut dyn FnMut(crate::ErrorReport<'a>));

struct OpsFor<T>(core::marker::PhantomData<T>);
impl<T: SendSyncError> OpsFor<T> {
    const OPS: ErrorOps = ErrorOps {
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
    const WITH_CHILDREN: ErrorOps = ErrorOps {
        children: Some(|root, visitor| {
            let batch = (root as &dyn Any)
                .downcast_ref::<crate::ErrorUnionVec<E>>()
                .expect("child visitor must match the stored batch");
            for child in batch {
                visitor(child.report());
            }
        }),
        ..Self::OPS
    };
}

/// # Safety
/// data must belong to a live StoredError constructed for exactly T.
#[inline]
unsafe fn concrete_pointer<T>(data: *mut ()) -> *mut T {
    #[cfg(feature = "alloc")]
    // SAFETY: data points to our original HeapError<T> allocation. addr_of_mut!
    // borrows neither the allocation nor T, preserving interior mutability.
    unsafe {
        core::ptr::addr_of_mut!((*data.cast::<HeapError<T>>()).value)
    }
    #[cfg(not(feature = "alloc"))]
    {
        let _ = data;
        // Construction established an inhabited ZST. Its logical owned value
        // needs no bytes, but every borrow still requires correct alignment.
        NonNull::<T>::dangling().as_ptr()
    }
}

/// # Safety
/// data must own a live T; returned references must obey the owner's borrow.
#[inline]
unsafe fn error_pointer<T: SendSyncError>(data: *mut ()) -> *mut dyn SendSyncError {
    // Coercion supplies Rust's real trait-object metadata.
    unsafe { concrete_pointer::<T>(data) }
}

/// # Safety
/// The caller must own the live T and never access it again afterwards.
#[inline]
unsafe fn drop_error<T: SendSyncError>(data: *mut ()) {
    #[cfg(feature = "alloc")]
    // SAFETY: recover the original allocation, dropping T and freeing its heap.
    unsafe {
        drop(Box::from_raw(data.cast::<HeapError<T>>()));
    }
    #[cfg(not(feature = "alloc"))]
    // SAFETY: construction established validity, including for aligned ZSTs
    // with destructors. This transfers logical ownership into drop_in_place.
    unsafe {
        concrete_pointer::<T>(data).drop_in_place();
    }
}

/// # Safety
/// The caller must exclusively own the live T and suppress StoredError's drop.
#[cfg(feature = "alloc")]
#[inline]
unsafe fn box_error<T: SendSyncError>(data: *mut ()) -> ErrorParts<Box<dyn SendSyncError>> {
    // Move the original concrete type into its own box, keeping diagnostics
    // owned while the header allocation is freed.
    let parts = unsafe { take_heap_error::<T>(data) };
    ErrorParts {
        error: Box::new(parts.error),
        diagnostics: parts.diagnostics,
    }
}

/// # Safety
/// data must be the exclusively owned allocation for exactly T. Its old owner
/// must suppress Drop, since this function moves every field out and frees it.
#[cfg(feature = "alloc")]
#[inline]
unsafe fn take_heap_error<T>(data: *mut ()) -> ErrorParts<T> {
    let HeapError {
        header: ErrorHeader { diagnostics, .. },
        value,
    } = *unsafe { Box::from_raw(data.cast::<HeapError<T>>()) };
    ErrorParts {
        error: value,
        diagnostics,
    }
}

impl StoredError {
    #[inline]
    fn new<T: SendSyncError>(
        value: T,
        #[cfg(feature = "alloc")] diagnostics: ErrorDiagnostics,
    ) -> Self {
        #[cfg(feature = "alloc")]
        {
            Self {
                allocation: NonNull::from(Box::leak(Box::new(HeapError {
                    header: ErrorHeader {
                        ops: &OpsFor::<T>::OPS,
                        diagnostics,
                    },
                    value,
                })))
                .cast(),
            }
        }
        #[cfg(not(feature = "alloc"))]
        {
            const {
                assert!(
                    size_of::<T>() == 0,
                    "T must be zero-sized when the alloc feature is disabled"
                );
            }
            // Transfer logical ownership; operations reconstruct aligned ZST
            // pointers and perform its destructor exactly once when needed.
            core::mem::forget(value);
            Self {
                ops: &OpsFor::<T>::OPS,
            }
        }
    }

    #[inline]
    fn ops(&self) -> &'static ErrorOps {
        #[cfg(feature = "alloc")]
        // SAFETY: the common header is initialized and lives until owner drop.
        // Borrow only the header, leaving T available for interior mutation.
        unsafe {
            self.allocation.as_ref().ops
        }
        #[cfg(not(feature = "alloc"))]
        {
            self.ops
        }
    }

    #[inline]
    fn data(&self) -> *mut () {
        #[cfg(feature = "alloc")]
        {
            self.allocation.as_ptr().cast()
        }
        #[cfg(not(feature = "alloc"))]
        {
            NonNull::<()>::dangling().as_ptr()
        }
    }

    #[cfg(feature = "alloc")]
    fn set_ops(&mut self, ops: &'static ErrorOps) {
        // SAFETY: exclusive ownership and identical concrete type operations.
        unsafe {
            self.allocation.as_mut().ops = ops;
        }
    }

    #[inline]
    fn error(&self) -> &dyn SendSyncError {
        // SAFETY: ops matches the live T, and the borrow is bounded by self.
        unsafe { &*(self.ops().pointer)(self.data()) }
    }

    #[inline]
    fn error_mut(&mut self) -> &mut dyn SendSyncError {
        // SAFETY: exclusive access to the owned error.
        unsafe { &mut *(self.ops().pointer)(self.data()) }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    fn into_box(self) -> ErrorParts<Box<dyn SendSyncError>> {
        let owner = ManuallyDrop::new(self);
        // SAFETY: transfer exclusive ownership and suppress the old drop.
        unsafe { (owner.ops().into_box)(owner.data()) }
    }
}

impl Drop for StoredError {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: construction selected these operations for our one live T.
        unsafe { (self.ops().drop)(self.data()) };
    }
}

// SAFETY: construction only accepts Send + Sync errors, and borrows enforce
// T's aliasing rules. Neither pointer exposes access independent of its owner.
unsafe impl Send for StoredError {}
unsafe impl Sync for StoredError {}

/// With alloc this handle owns a header, diagnostics, and trailing payload in
/// one allocation. Without alloc only the optional location needs inline bytes.
pub(crate) struct ErrorUnionInner {
    error: StoredError,
    #[cfg(not(feature = "alloc"))]
    diagnostics: ErrorDiagnostics,
}

/// Stored in the allocation's header with alloc, or inline without an allocator.
pub(crate) struct ErrorDiagnostics {
    #[cfg(feature = "backtrace")]
    pub(crate) backtrace: std::backtrace::Backtrace,
    #[cfg(feature = "error_span_trace")]
    pub(crate) span_trace: tracing_error::SpanTrace,
    #[cfg(feature = "context")]
    pub(crate) context: Vec<ContextFrame>,
    #[cfg(feature = "error_location")]
    pub(crate) location: &'static core::panic::Location<'static>,
}

pub(crate) struct ErrorParts<T> {
    pub(crate) error: T,
    pub(crate) diagnostics: ErrorDiagnostics,
}

impl ErrorUnionInner {
    #[cfg(feature = "alloc")]
    #[cfg_attr(feature = "error_location", track_caller)]
    pub(crate) fn new_vec<E: crate::TypeSet>(value: crate::ErrorUnionVec<E>) -> Self {
        let mut inner = Self::new(value);
        inner
            .error
            .set_ops(&OpsFor::<crate::ErrorUnionVec<E>>::WITH_CHILDREN);
        inner
    }

    pub(crate) fn visit_children<'a>(&'a self, _visitor: &mut dyn FnMut(crate::ErrorReport<'a>)) {
        #[cfg(feature = "alloc")]
        if let Some(children) = self.error.ops().children {
            children(self.error(), _visitor);
        }
    }

    #[cfg(feature = "alloc")]
    pub(crate) fn is_collection(&self) -> bool {
        self.error.ops().children.is_some()
    }

    #[cfg_attr(feature = "error_location", track_caller)]
    pub(crate) fn new<T: SendSyncError>(value: T) -> Self {
        Self::new_from_parts(
            value,
            #[cfg(feature = "backtrace")]
            std::backtrace::Backtrace::capture(),
            #[cfg(feature = "error_span_trace")]
            tracing_error::SpanTrace::capture(),
            #[cfg(feature = "context")]
            Vec::new(),
            #[cfg(feature = "error_location")]
            core::panic::Location::caller(),
        )
    }

    #[inline]
    pub(crate) fn new_from_parts<T: SendSyncError>(
        value: T,
        #[cfg(feature = "backtrace")] backtrace: std::backtrace::Backtrace,
        #[cfg(feature = "error_span_trace")] span_trace: tracing_error::SpanTrace,
        #[cfg(feature = "context")] context: Vec<ContextFrame>,
        #[cfg(feature = "error_location")] location: &'static core::panic::Location<'static>,
    ) -> Self {
        let diagnostics = ErrorDiagnostics {
            #[cfg(feature = "backtrace")]
            backtrace,
            #[cfg(feature = "error_span_trace")]
            span_trace,
            #[cfg(feature = "context")]
            context,
            #[cfg(feature = "error_location")]
            location,
        };
        Self::from_parts(ErrorParts {
            error: value,
            diagnostics,
        })
    }

    #[inline]
    fn from_parts<T: SendSyncError>(parts: ErrorParts<T>) -> Self {
        Self {
            error: StoredError::new(
                parts.error,
                #[cfg(feature = "alloc")]
                parts.diagnostics,
            ),
            #[cfg(not(feature = "alloc"))]
            diagnostics: parts.diagnostics,
        }
    }

    #[cfg(any(
        feature = "backtrace",
        feature = "error_span_trace",
        feature = "context",
        feature = "error_location"
    ))]
    pub(crate) const fn diagnostics(&self) -> &ErrorDiagnostics {
        #[cfg(feature = "alloc")]
        // SAFETY: the initialized header lives for the owner's entire borrow.
        // Borrow only diagnostics; the trailing payload remains independent.
        unsafe {
            &self.error.allocation.as_ref().diagnostics
        }
        #[cfg(not(feature = "alloc"))]
        {
            &self.diagnostics
        }
    }

    #[cfg(any(feature = "backtrace", feature = "context"))]
    pub(crate) fn diagnostics_mut(&mut self) -> &mut ErrorDiagnostics {
        // SAFETY: exclusive access to the live header's diagnostics.
        unsafe { &mut (*self.error.allocation.as_ptr()).diagnostics }
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
        self.error.ops().type_id == TypeId::of::<T>()
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
            let _diagnostics = parts.diagnostics;
        }
        error
    }

    /// # Safety
    /// T must be the exact concrete type of the owned error.
    #[inline]
    pub(crate) unsafe fn downcast_error_unchecked_with_parts<T: 'static>(self) -> ErrorParts<T> {
        debug_assert!(self.is_error_type::<T>());
        let owner = ManuallyDrop::new(self.error);
        #[cfg(feature = "alloc")]
        // SAFETY: exact type, exclusive ownership, and the old drop is suppressed.
        unsafe {
            take_heap_error::<T>(owner.data())
        }
        #[cfg(not(feature = "alloc"))]
        // SAFETY: transfer the inhabited ZST, suppressing the old owner's drop.
        unsafe {
            ErrorParts {
                error: concrete_pointer::<T>(owner.data()).read(),
                diagnostics: self.diagnostics,
            }
        }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    pub(crate) fn into_boxed_parts(self) -> ErrorParts<Box<dyn SendSyncError>> {
        self.error.into_box()
    }

    #[inline]
    pub(crate) fn downcast_error_ref<T: 'static>(&self) -> Option<&T> {
        if self.is_error_type::<T>() {
            // SAFETY: the exact type was checked using the shared operations.
            Some(unsafe { self.downcast_error_ref_unchecked::<T>() })
        } else {
            None
        }
    }
    #[inline]
    pub(crate) fn downcast_error_mut<T: 'static>(&mut self) -> Option<&mut T> {
        if self.is_error_type::<T>() {
            // SAFETY: the type matches and the owner is exclusively borrowed.
            Some(unsafe { self.downcast_error_mut_unchecked::<T>() })
        } else {
            None
        }
    }

    /// # Safety
    /// T must be the exact concrete type of the owned error.
    #[inline]
    pub(crate) unsafe fn downcast_error_ref_unchecked<T: 'static>(&self) -> &T {
        debug_assert!(self.is_error_type::<T>());
        // SAFETY: project the known concrete payload directly, without creating
        // a trait object or dispatching through the pointer operation.
        unsafe { &*concrete_pointer::<T>(self.error.data()) }
    }
    /// # Safety
    /// T must be the exact concrete type of the owned error.
    #[inline]
    pub(crate) unsafe fn downcast_error_mut_unchecked<T: 'static>(&mut self) -> &mut T {
        debug_assert!(self.is_error_type::<T>());
        // SAFETY: exact type and exclusive access, with lifetime following self.
        unsafe { &mut *concrete_pointer::<T>(self.error.data()) }
    }

    /// # Safety
    /// T must be the exact concrete type of the owned error.
    #[inline]
    pub(crate) unsafe fn map_single<T, U>(self, f: impl FnOnce(T) -> U) -> Self
    where
        T: SendSyncError,
        U: SendSyncError,
    {
        debug_assert!(self.is_error_type::<T>());
        #[cfg(feature = "alloc")]
        if const {
            size_of::<HeapError<T>>() == size_of::<HeapError<U>>()
                && align_of::<HeapError<T>>() == align_of::<HeapError<U>>()
        } {
            let owner = ManuallyDrop::new(self);
            // SAFETY: MaybeUninit<T> has T's size/alignment, so this guard owns
            // the original allocation. Once T moves to f, unwinding drops the
            // diagnostics and frees the allocation without touching the dead T.
            let allocation =
                unsafe { Box::from_raw(owner.error.data().cast::<HeapError<MaybeUninit<T>>>()) };
            let old = unsafe { allocation.value.as_ptr().read() };
            let value = f(old);
            let raw = Box::into_raw(allocation).cast::<HeapError<U>>();
            // SAFETY: the complete layouts match, including alignment. Moving
            // U into the payload and installing its operations restores a live
            // owner. Diagnostics retain their original place and ownership.
            unsafe {
                core::ptr::addr_of_mut!((*raw).value).write(value);
                core::ptr::addr_of_mut!((*raw).header.ops).write(&OpsFor::<U>::OPS);
                return Self {
                    error: StoredError {
                        allocation: NonNull::new_unchecked(raw.cast()),
                    },
                };
            }
        }
        // Different layouts need fresh storage. Both moved values have normal
        // local owners here, so a panic in f also drops saved diagnostics.
        let parts = unsafe { self.downcast_error_unchecked_with_parts::<T>() };
        Self::from_parts(ErrorParts {
            error: f(parts.error),
            diagnostics: parts.diagnostics,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_storage_is_one_word_with_a_pointer_niche() {
        assert_eq!(size_of::<StoredError>(), size_of::<usize>());
        assert_eq!(align_of::<StoredError>(), align_of::<usize>());
        assert_eq!(size_of::<Option<StoredError>>(), size_of::<usize>());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn equal_size_with_different_alignment_requires_replacement_storage() {
        #[derive(Debug)]
        #[repr(align(256))]
        struct Aligned(u8);
        impl core::fmt::Display for Aligned {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
        impl core::error::Error for Aligned {}

        // The two complete allocations have equal sizes but different required
        // alignments. Reusing the first as the second would give Box the wrong
        // deallocation layout, even if its address happened to be aligned.
        const BYTES: usize = size_of::<HeapError<Aligned>>() - size_of::<ErrorHeader>();
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
        let error = ErrorUnionInner::new(Unaligned([73; BYTES]));
        // SAFETY: construction established exactly Unaligned.
        let mapped = unsafe { error.map_single::<Unaligned, Aligned>(|old| Aligned(old.0[0])) };
        let value = mapped.downcast_error_ref::<Aligned>().unwrap();
        assert_eq!((value as *const Aligned).addr() % 256, 0);
        assert_eq!(value.0, 73);
        drop(mapped); // Miri checks that the allocator receives the correct layout.
    }
}
