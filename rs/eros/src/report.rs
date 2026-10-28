use core::fmt;

use crate::{ErrorUnion, SendSyncError, TypeSet, formatting::Report, storage::ErrorUnionInner};

/// A borrowed report independent of an [`ErrorUnion`]'s static error set.
///
/// Created by [`ErrorUnion::report`]. Formatting follows the union's Display and
/// Debug conventions. The view neither owns nor copies the saved diagnostics.
#[derive(Clone, Copy)]
pub struct ErrorReport<'a> {
    pub(crate) inner: &'a ErrorUnionInner,
}

impl<E: TypeSet> ErrorUnion<E> {
    /// Borrows this union's report without erasing its owned error set.
    pub const fn report(&self) -> ErrorReport<'_> {
        ErrorReport { inner: &self.inner }
    }
}

impl<'a> ErrorReport<'a> {
    /// Borrows the concrete root, supporting downcasts by custom reporters.
    pub fn inner(&self) -> &'a dyn SendSyncError {
        self.inner.error()
    }

    /// Borrows the immediate batch children in insertion order.
    ///
    /// Visits every child, including those omitted by the built-in report limit.
    pub fn visit_children(&self, visitor: &mut dyn FnMut(ErrorReport<'a>)) {
        self.inner.visit_children(visitor);
    }

    /// Borrows saved contexts in attachment order (innermost first).
    #[cfg(feature = "context")]
    pub fn contexts(
        &self,
    ) -> impl DoubleEndedIterator<Item = &'a crate::ContextFrame> + ExactSizeIterator {
        self.inner.context.iter()
    }

    /// Borrows the saved Eros backtrace, without capturing a new one.
    #[cfg(feature = "backtrace")]
    pub const fn backtrace(&self) -> &'a std::backtrace::Backtrace {
        &self.inner.backtrace
    }

    /// Borrows the original span trace without capturing a new one.
    /// Requires `error_span_trace` (also enabled by `span_trace`).
    #[cfg(feature = "error_span_trace")]
    pub const fn span_trace(&self) -> &'a tracing_error::SpanTrace {
        &self.inner.span_trace
    }

    /// Returns the original Eros capture location.
    /// Requires `error_location` (also enabled by `location`).
    #[cfg(feature = "error_location")]
    pub const fn location(&self) -> &'static core::panic::Location<'static> {
        self.inner.location
    }
}

impl fmt::Display for ErrorReport<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Report::from_inner(self.inner).display(f)
    }
}

impl fmt::Debug for ErrorReport<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Report::from_inner(self.inner).debug(f)
    }
}
