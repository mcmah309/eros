#[cfg(feature = "alloc")]
use crate::storage::ErrorUnionInner;
use crate::type_set::{Contains, IsFold};
use crate::{Classifier, ErrorUnion, SendSyncError, TypeSet};
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::{any::Any, marker::PhantomData};

impl<E: TypeSet> ErrorUnion<E> {
    /// Classifies implementation errors under an error type meaningful to callers.
    ///
    /// `Other` must contain `ClassifiedError`. Concrete types already represented
    /// in `Other` retain their storage and bypass the classifier. For other types,
    /// the classifier receives ownership of the original error and returns a
    /// replacement. Callbacks taking [`crate::InnerError`] work with every feature
    /// configuration. With `alloc`, callbacks taking `Box<dyn SendSyncError>`
    /// are also accepted, including tuple struct constructors.
    ///
    /// Pass an error value directly to discard the original when classification
    /// is needed. This works for unit structs and errors with fields. The value
    /// is dropped unused when the original type already belongs to `Other`.
    ///
    /// ```
    /// let error = eros::error!("device unavailable");
    /// let error: eros::ErrorUnion<(core::fmt::Error,)> = error.classify(core::fmt::Error);
    /// assert_eq!(error.into_single(), core::fmt::Error);
    /// ```
    ///
    /// Closure parameters may need an explicit type, such as `|inner: InnerError|`
    /// or `|_: InnerError|`. The final generic parameter selects the callback's
    /// input type and is normally inferred.
    ///
    /// Only the concrete inner type is checked, not its sources or contexts.
    /// All saved diagnostics are preserved without capturing new ones. The
    /// replacement follows the same storage limits as [`Self::new`].
    ///
    /// ```
    /// use eros::{ErrorUnion, InnerError};
    ///
    /// fn category(_: InnerError) -> core::fmt::Error {
    ///     core::fmt::Error
    /// }
    /// let error = eros::error!("device unavailable");
    /// let error: ErrorUnion<(core::fmt::Error,)> = error.classify(category);
    /// assert!(error.is_inner::<core::fmt::Error>());
    /// ```
    ///
    /// Inspect or extract the original with [`crate::InnerError::downcast_ref`] and
    /// [`crate::InnerError::downcast`]. With `alloc`, the owner also provides
    /// `into_boxed()` to keep the original as a domain wrapper's source.
    /// A constructor taking a boxed error can also be passed directly:
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// #[derive(Debug, thiserror::Error)]
    /// #[error("configuration failed: {0}")]
    /// struct ConfigError(#[source] Box<dyn eros::SendSyncError>);
    /// let error = eros::error!("device unavailable");
    /// let error: eros::ErrorUnion<(ConfigError,)> =
    ///     error.classify(ConfigError);
    /// assert!(error.is_inner::<ConfigError>());
    /// # }
    /// ```
    ///
    /// The destination must include the replacement type:
    ///
    /// ```compile_fail
    /// let error: eros::ErrorUnion = eros::ErrorUnion::new(core::fmt::Error);
    /// let _: eros::ErrorUnion<()> = error.classify(|_: eros::InnerError| core::fmt::Error);
    /// ```
    pub fn classify<Other, ClassifiedError, Index, Input>(
        self,
        classifier: impl Classifier<ClassifiedError, Input>,
    ) -> ErrorUnion<Other>
    where
        Other: TypeSet,
        ClassifiedError: SendSyncError,
        Other::Variants: Contains<ClassifiedError, Index>,
    {
        let inner = if Other::Variants::is_fold(self.inner() as &dyn Any) {
            self.inner
        } else {
            self.inner
                .map_error(|inner| classifier.classify_error(inner))
        };
        ErrorUnion {
            inner,
            _pd: PhantomData,
        }
    }

    /// Replaces the inner error, passing ownership of the old boxed error to a closure.
    ///
    /// The closure runs once, receives the previous inner error, and returns the new
    /// concrete error value. Eros selects storage for the replacement at compile time.
    /// Preserve the original failure by storing the old inner error in the new
    /// error and returning it from `Error::source()`. Eros uses the new error's
    /// source chain without automatically adding the old inner error.
    ///
    /// All saved diagnostics are preserved, including context, the original
    /// Eros capture location, backtrace, and span trace. No new diagnostics
    /// are captured.
    ///
    /// When root diagnostics are disabled, an already boxed root reuses its
    /// allocation. Other roots move into a new box before the closure runs;
    /// boxing a zero-sized concrete root needs no allocation.
    ///
    /// The returned union has the replacement type as its single variant.
    /// For a union with a single variant, [`Self::map_single`] passes the concrete
    /// inner error to the closure instead of a boxed error.
    ///
    /// Wrap the old inner error as the source of a new error:
    ///
    /// ```
    /// use eros::SendSyncError;
    /// use std::{error::Error, fmt};
    ///
    /// #[derive(Debug)]
    /// struct ConfigError {
    ///     source: Box<dyn SendSyncError>,
    /// }
    ///
    /// impl fmt::Display for ConfigError {
    ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         f.write_str("cannot open configuration")
    ///     }
    /// }
    ///
    /// impl Error for ConfigError {
    ///     fn source(&self) -> Option<&(dyn Error + 'static)> {
    ///         Some(&*self.source)
    ///     }
    /// }
    ///
    /// let error = eros::error!("permission denied")
    ///     .map_inner(|old| ConfigError { source: old });
    ///
    /// assert_eq!(error.to_string(), "cannot open configuration\n    caused by:\n        - permission denied");
    /// assert!(error.is_inner::<ConfigError>());
    /// assert_eq!(error.source().unwrap().to_string(), "permission denied");
    /// ```
    #[cfg(feature = "alloc")]
    pub fn map_inner<T, F>(self, f: F) -> ErrorUnion<(T,)>
    where
        T: SendSyncError,
        F: FnOnce(Box<dyn SendSyncError>) -> T,
    {
        let parts = self.inner.into_boxed_parts();
        ErrorUnion {
            inner: ErrorUnionInner::new_from_parts(
                f(parts.error),
                #[cfg(feature = "backtrace")]
                parts.diagnostics.backtrace,
                #[cfg(feature = "error_span_trace")]
                parts.diagnostics.span_trace,
                #[cfg(feature = "context")]
                parts.diagnostics.context,
                #[cfg(feature = "error_location")]
                parts.diagnostics.location,
            ),
            _pd: core::marker::PhantomData,
        }
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    #[cfg(feature = "backtrace")]
    use alloc::string::ToString;

    use crate::{ErrorUnion, MsgError};

    #[test]
    fn replacement_and_classification_preserve_metadata() {
        #[allow(unused_mut)]
        let mut error: ErrorUnion = ErrorUnion::new(MsgError::from("permission denied"));

        #[cfg(feature = "backtrace")]
        let original_backtrace = {
            error.inner.diagnostics_mut().backtrace = std::backtrace::Backtrace::force_capture();
            error.backtrace().to_string()
        };
        #[cfg(feature = "error_location")]
        let original_location = error.inner.diagnostics().location;
        #[cfg(feature = "context")]
        let error = error.context("read /etc/app.toml").context("start service");
        #[cfg(feature = "context")]
        let original_context = error.inner.diagnostics().context.as_ptr();

        let error = error.map_inner(|_| MsgError::from("cannot open configuration"));
        let _error: ErrorUnion<(core::fmt::Error,)> =
            error.classify(|_: eros::InnerError| core::fmt::Error);

        #[cfg(feature = "backtrace")]
        {
            assert_eq!(
                _error.backtrace().status(),
                std::backtrace::BacktraceStatus::Captured
            );
            assert_eq!(_error.backtrace().to_string(), original_backtrace);
        }
        #[cfg(feature = "error_location")]
        assert!(core::ptr::eq(
            _error.inner.diagnostics().location,
            original_location
        ));
        #[cfg(feature = "context")]
        {
            assert_eq!(
                _error.inner.diagnostics().context.as_ptr(),
                original_context
            );
            assert_eq!(_error.inner.diagnostics().context.len(), 2);
            assert_eq!(
                alloc::format!("{}", _error.inner.diagnostics().context[0].context),
                "read /etc/app.toml"
            );
            assert_eq!(
                alloc::format!("{}", _error.inner.diagnostics().context[1].context),
                "start service"
            );
        }
    }
}
