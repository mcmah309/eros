#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "context")]
use alloc::vec::Vec;

#[cfg(feature = "context")]
use crate::ContextFrame;
use crate::SendSyncError;

// Feature selection is global: diagnostics and their payload share one box.
// Otherwise a word-sized payload can live directly in the two-word handle.
#[cfg(all(
    feature = "alloc",
    any(
        feature = "backtrace",
        feature = "context",
        feature = "error_span_trace",
        feature = "error_location"
    )
))]
mod boxed;
#[cfg(all(
    feature = "alloc",
    any(
        feature = "backtrace",
        feature = "context",
        feature = "error_span_trace",
        feature = "error_location"
    )
))]
use boxed::Storage;
#[cfg(any(
    not(feature = "alloc"),
    not(any(
        feature = "backtrace",
        feature = "context",
        feature = "error_span_trace",
        feature = "error_location"
    ))
))]
mod inline;
#[cfg(any(
    not(feature = "alloc"),
    not(any(
        feature = "backtrace",
        feature = "context",
        feature = "error_span_trace",
        feature = "error_location"
    ))
))]
use inline::Storage;

#[cfg(feature = "alloc")]
type ChildVisitor = for<'a> fn(&'a dyn SendSyncError, &mut dyn FnMut(crate::ErrorReport<'a>));

#[cfg(feature = "alloc")]
fn visit_batch<'a, E: crate::TypeSet>(
    root: &'a dyn SendSyncError,
    visitor: &mut dyn FnMut(crate::ErrorReport<'a>),
) {
    let batch = (root as &dyn core::any::Any)
        .downcast_ref::<crate::ErrorUnionVec<E>>()
        .expect("child visitor must match the stored batch");
    for child in batch {
        visitor(child.report());
    }
}

pub(crate) struct ErrorUnionInner {
    storage: Storage,
}

// Root diagnostics share the allocation when available. Without an allocator,
// the caller location needs one word alongside the payload and operations.
const HANDLE_WORDS: usize = if cfg!(all(not(feature = "alloc"), feature = "error_location")) {
    3
} else {
    2
};
const _: () = assert!(size_of::<ErrorUnionInner>() <= HANDLE_WORDS * size_of::<usize>());

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

impl ErrorDiagnostics {
    // Transfer metadata without capturing new diagnostics. A span clone shares
    // the already captured span until the old root is dropped.
    fn take(&mut self) -> Self {
        Self {
            #[cfg(feature = "backtrace")]
            backtrace: core::mem::replace(
                &mut self.backtrace,
                std::backtrace::Backtrace::disabled(),
            ),
            #[cfg(feature = "error_span_trace")]
            span_trace: self.span_trace.clone(),
            #[cfg(feature = "context")]
            context: core::mem::take(&mut self.context),
            #[cfg(feature = "error_location")]
            location: self.location,
        }
    }
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
        // SAFETY: this constructor established exactly ErrorUnionVec<E>.
        unsafe {
            inner.storage.enable_children::<E>();
        }
        inner
    }

    pub(crate) fn visit_children<'a>(&'a self, _visitor: &mut dyn FnMut(crate::ErrorReport<'a>)) {
        #[cfg(feature = "alloc")]
        if let Some(children) = self.storage.children() {
            children(self.error(), _visitor);
        }
    }

    #[cfg(feature = "alloc")]
    pub(crate) fn is_collection(&self) -> bool {
        self.storage.children().is_some()
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
        Self {
            storage: Storage::new(
                value,
                ErrorDiagnostics {
                    #[cfg(feature = "backtrace")]
                    backtrace,
                    #[cfg(feature = "error_span_trace")]
                    span_trace,
                    #[cfg(feature = "context")]
                    context,
                    #[cfg(feature = "error_location")]
                    location,
                },
            ),
        }
    }

    #[cfg(any(
        feature = "backtrace",
        feature = "context",
        feature = "error_span_trace",
        feature = "error_location"
    ))]
    pub(crate) const fn diagnostics(&self) -> &ErrorDiagnostics {
        self.storage.diagnostics()
    }

    #[cfg(any(feature = "backtrace", feature = "context"))]
    pub(crate) fn diagnostics_mut(&mut self) -> &mut ErrorDiagnostics {
        self.storage.diagnostics_mut()
    }

    #[inline]
    pub(crate) fn error(&self) -> &dyn SendSyncError {
        self.storage.error()
    }
    #[inline]
    pub(crate) fn error_mut(&mut self) -> &mut dyn SendSyncError {
        self.storage.error_mut()
    }
    #[inline]
    pub(crate) fn is_error_type<T: 'static>(&self) -> bool {
        self.storage.is_error_type::<T>()
    }

    /// # Safety
    /// T must be the exact concrete type of the owned error.
    #[inline]
    pub(crate) unsafe fn downcast_error_unchecked<T: 'static>(self) -> T {
        let error;
        {
            let parts = unsafe { self.downcast_error_unchecked_with_parts::<T>() };
            // Keep the root owned while dropping diagnostics, including if a
            // context destructor panics before the root reaches its caller.
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
        unsafe { self.storage.into_parts::<T>() }
    }

    #[cfg(feature = "alloc")]
    #[inline]
    pub(crate) fn into_boxed_parts(self) -> ErrorParts<Box<dyn SendSyncError>> {
        self.storage.into_boxed_parts()
    }

    #[inline]
    pub(crate) fn downcast_error_ref<T: 'static>(&self) -> Option<&T> {
        if self.is_error_type::<T>() {
            Some(unsafe { self.downcast_error_ref_unchecked::<T>() })
        } else {
            None
        }
    }
    #[inline]
    pub(crate) fn downcast_error_mut<T: 'static>(&mut self) -> Option<&mut T> {
        if self.is_error_type::<T>() {
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
        unsafe { self.storage.concrete_ref::<T>() }
    }
    /// # Safety
    /// T must be the exact concrete type of the owned error.
    #[inline]
    pub(crate) unsafe fn downcast_error_mut_unchecked<T: 'static>(&mut self) -> &mut T {
        debug_assert!(self.is_error_type::<T>());
        unsafe { self.storage.concrete_mut::<T>() }
    }

    pub(crate) fn map_error<T: SendSyncError>(
        mut self,
        classifier: impl FnOnce(crate::InnerError) -> T,
    ) -> Self {
        let diagnostics = self.storage.take_diagnostics();
        // The callback owns the old root. Metadata remains owned here if the
        // callback, the original destructor, or replacement construction panics.
        let value = classifier(crate::InnerError { inner: self });
        Self {
            storage: Storage::new(value, diagnostics),
        }
    }

    /// # Safety
    /// T must be the exact concrete type of the owned error.
    #[inline]
    pub(crate) unsafe fn map_single<T: SendSyncError, U: SendSyncError>(
        self,
        f: impl FnOnce(T) -> U,
    ) -> Self {
        debug_assert!(self.is_error_type::<T>());
        Self {
            storage: unsafe { self.storage.map_single::<T, U>(f) },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn handle_and_its_option_obey_the_feature_size_limit() {
        let words = HANDLE_WORDS;
        assert_eq!(size_of::<ErrorUnionInner>(), words * size_of::<usize>());
        assert_eq!(
            size_of::<Option<ErrorUnionInner>>(),
            words * size_of::<usize>()
        );
        assert_eq!(align_of::<ErrorUnionInner>(), align_of::<usize>());
    }
}
