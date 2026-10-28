use crate::storage::ErrorUnionInner;
use crate::type_set::{Contains, IsFold};
use crate::{ErrorUnion, SendSyncError, TypeSet};
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
use core::{any::Any, marker::PhantomData};

impl<E: TypeSet> ErrorUnion<E> {
    /// Classifies implementation errors under an error type meaningful to callers.
    ///
    /// `Other` must contain `ClassifiedError`. Concrete types already represented
    /// in `Other` retain their storage. For other types, `classifier` receives the
    /// boxed inner error once and produces `ClassifiedError`. This lets an API
    /// expose selected errors while hiding other implementation error types
    /// behind a category its callers understand.
    ///
    /// Classification also preserves domain identity: an `io::Error` from
    /// configuration loading can become `ConfigError`, while an `io::Error`
    /// from a database operation becomes `DatabaseError`. Combining those paths
    /// retains two distinct categories instead of a shared `io::Error` variant.
    /// A wrapper can retain the original failure as its source for diagnostics.
    ///
    /// Only the inner error is checked, not its sources or contexts. All saved
    /// diagnostics are preserved.
    ///
    /// Without `alloc`, the classifier borrows the original error as
    /// `&dyn SendSyncError` and returns a replacement satisfying the same storage
    /// limits as [`Self::new`]. The original is dropped after classification;
    /// the replacement cannot retain it as an owned source.
    ///
    /// With `alloc`, the classifier receives ownership of a boxed error.
    /// The classifier receives the original box when root diagnostics are
    /// disabled and the root was already boxed. Otherwise the root moves into
    /// a new box. The returned wrapper follows Eros's compile-time storage choice.
    ///
    /// Pass a tuple struct constructor to distinguish errors from one domain
    /// when combining paths. No `From` implementation is required:
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// use eros::{ErrorUnion, SendSyncError};
    /// use std::{error::Error, fmt};
    ///
    /// #[derive(Debug)]
    /// struct ConfigError(Box<dyn SendSyncError>);
    /// impl fmt::Display for ConfigError {
    ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "configuration failed: {}", self.0)
    ///     }
    /// }
    /// impl Error for ConfigError {
    ///     fn source(&self) -> Option<&(dyn Error + 'static)> {
    ///         Some(&*self.0)
    ///     }
    /// }
    ///
    /// let error: ErrorUnion = ErrorUnion::new(eros::MsgError::from("failure"));
    /// let error: ErrorUnion<(fmt::Error, ConfigError)> =
    ///     error.classify(ConfigError);
    /// let error: ErrorUnion<(ConfigError, std::io::Error, fmt::Error)> = error.widen();
    /// assert!(error.is_inner::<ConfigError>());
    /// // Erasing the set keeps the domain wrapper as the concrete root.
    /// assert!(error.erase().is_inner::<ConfigError>());
    /// # }
    /// ```
    ///
    /// A wrapper constructor is required:
    ///
    /// ```compile_fail
    /// let error: eros::ErrorUnion = eros::ErrorUnion::new(std::fmt::Error);
    /// let _: eros::ErrorUnion<(std::fmt::Error, Box<dyn eros::SendSyncError>)> =
    ///     error.classify();
    /// ```
    ///
    /// The destination must include the wrapper type:
    ///
    /// ```compile_fail
    /// let error: eros::ErrorUnion = eros::ErrorUnion::new(std::fmt::Error);
    /// let _: eros::ErrorUnion<(std::fmt::Error,)> = error.classify(|_| std::io::Error::other("failure"));
    /// ```
    pub fn classify<Other, ClassifiedError, Index>(
        self,
        #[cfg(feature = "alloc")] classifier: impl FnOnce(Box<dyn SendSyncError>) -> ClassifiedError,
        #[cfg(not(feature = "alloc"))] classifier: impl FnOnce(&dyn SendSyncError) -> ClassifiedError,
    ) -> ErrorUnion<Other>
    where
        Other: TypeSet,
        ClassifiedError: SendSyncError,
        Other::Variants: Contains<ClassifiedError, Index>,
    {
        let inner = if Other::Variants::is_fold(self.inner() as &dyn Any) {
            self.inner
        } else {
            #[cfg(feature = "alloc")]
            {
                self.map_inner(classifier).inner
            }
            #[cfg(not(feature = "alloc"))]
            {
                let replacement = classifier(self.inner());
                let inner = ErrorUnionInner::new_from_parts(
                    replacement,
                    #[cfg(feature = "error_location")]
                    self.location(),
                );
                // Keep both roots owned if the original destructor panics.
                drop(self);
                inner
            }
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
    fn replacement_preserves_metadata() {
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
        let _error = error.map_inner(|_| MsgError::from("startup failed"));

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
