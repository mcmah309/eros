use crate::storage::ErrorUnionInner;
use crate::type_set::{Contains, IsFold};
use crate::{ErrorUnion, SendSyncError, TypeSet};
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
    /// diagnostics are preserved. Requires `alloc`.
    /// Heap roots reuse their allocation inside the wrapper; inline roots are
    /// boxed. Eros selects inline or boxed storage for the wrapper itself.
    ///
    /// Pass a tuple struct constructor to distinguish errors from one domain
    /// when combining paths. No `From` implementation is required:
    ///
    /// ```
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
        classifier: impl FnOnce(Box<dyn SendSyncError>) -> ClassifiedError,
    ) -> ErrorUnion<Other>
    where
        Other: TypeSet,
        ClassifiedError: SendSyncError,
        Other::Variants: Contains<ClassifiedError, Index>,
    {
        let inner = if Other::Variants::is_fold(self.inner() as &dyn Any) {
            self.inner
        } else {
            self.map_inner(classifier).inner
        };
        ErrorUnion {
            inner,
            _pd: PhantomData,
        }
    }

    /// Replaces the inner error, passing ownership of the old boxed error to a closure.
    ///
    /// The closure runs once, receives the previous inner error, and returns the new
    /// concrete error value. Eros selects inline or boxed storage for the replacement.
    /// Preserve the original failure by storing the old inner error in the new
    /// error and returning it from `Error::source()`. Eros uses the new error's
    /// source chain without automatically adding the old inner error.
    ///
    /// All saved diagnostics are preserved, including context, the original
    /// Eros capture location, backtrace, and span trace. No new diagnostics
    /// are captured.
    ///
    /// Heap-stored roots retain their allocation when passed to the closure.
    /// Inline non-zero-sized roots are boxed to satisfy the closure's argument.
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
                parts.backtrace,
                #[cfg(feature = "error_spantrace")]
                parts.span_trace,
                #[cfg(feature = "context")]
                parts.context,
                #[cfg(feature = "error_location")]
                parts.location,
            ),
            _pd: core::marker::PhantomData,
        }
    }
}

#[cfg(test)]
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
            error.inner.backtrace = std::backtrace::Backtrace::force_capture();
            error.backtrace().to_string()
        };
        #[cfg(feature = "error_location")]
        let original_location = error.inner.location;
        #[cfg(feature = "context")]
        let error = error.context("read /etc/app.toml").context("start service");
        #[cfg(feature = "context")]
        let original_context = error.inner.context.as_ptr();

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
        assert!(core::ptr::eq(_error.inner.location, original_location));
        #[cfg(feature = "context")]
        {
            assert_eq!(_error.inner.context.as_ptr(), original_context);
            assert_eq!(_error.inner.context.len(), 2);
            assert_eq!(
                alloc::format!("{}", _error.inner.context[0].context),
                "read /etc/app.toml"
            );
            assert_eq!(
                alloc::format!("{}", _error.inner.context[1].context),
                "start service"
            );
        }
    }
}
