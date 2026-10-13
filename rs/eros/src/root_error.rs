use crate::storage::ErrorUnionInner;
use crate::type_set::{Contains, IsFold};
use crate::{ErrorUnion, SendSyncError, TypeSet};
use alloc::boxed::Box;
use core::{any::Any, marker::PhantomData};

impl<E: TypeSet> ErrorUnion<E> {
    /// Classifies the inner error, wrapping unmatched errors in a named catch-all.
    ///
    /// `Other` must contain `CatchAll`. Matching concrete inner types retain
    /// their storage. Otherwise, `wrap` receives the boxed inner error exactly
    /// once and returns the catch-all variant. Only the inner error is checked,
    /// not its sources or contexts. Context, location, and backtrace are preserved.
    /// Requires `alloc`.
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
    /// struct DomainError(Box<dyn SendSyncError>);
    /// impl fmt::Display for DomainError {
    ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "domain operation failed: {}", self.0)
    ///     }
    /// }
    /// impl Error for DomainError {
    ///     fn source(&self) -> Option<&(dyn Error + 'static)> {
    ///         Some(&*self.0)
    ///     }
    /// }
    ///
    /// let error: ErrorUnion = ErrorUnion::new(eros::MsgError::from("failure"));
    /// let error: ErrorUnion<(fmt::Error, DomainError)> =
    ///     error.into_typed(DomainError);
    /// let error: ErrorUnion<(DomainError, std::io::Error, fmt::Error)> = error.widen();
    /// assert!(error.is_inner::<DomainError>());
    /// // Erasing the set keeps the domain wrapper as the concrete root.
    /// assert!(error.into_untyped().is_inner::<DomainError>());
    /// ```
    ///
    /// A wrapper constructor is required:
    ///
    /// ```compile_fail
    /// let error: eros::ErrorUnion = eros::ErrorUnion::new(std::fmt::Error);
    /// let _: eros::ErrorUnion<(std::fmt::Error, Box<dyn eros::SendSyncError>)> =
    ///     error.into_typed();
    /// ```
    ///
    /// The destination must include the wrapper type:
    ///
    /// ```compile_fail
    /// let error: eros::ErrorUnion = eros::ErrorUnion::new(std::fmt::Error);
    /// let _: eros::ErrorUnion<(std::fmt::Error,)> = error.into_typed(|_| std::io::Error::other("failure"));
    /// ```
    pub fn into_typed<Other, CatchAll, Index>(
        self,
        wrap: impl FnOnce(Box<dyn SendSyncError>) -> CatchAll,
    ) -> ErrorUnion<Other>
    where
        Other: TypeSet,
        CatchAll: SendSyncError,
        Other::Variants: Contains<CatchAll, Index>,
    {
        let inner = if Other::Variants::is_fold(self.inner() as &dyn Any) {
            self.inner
        } else {
            self.map_inner(wrap).inner
        };
        ErrorUnion {
            inner,
            _pd: PhantomData,
        }
    }

    /// Returns an untyped union, unwrapping a boxed catch-all root if present.
    ///
    /// Removes one `Box<dyn SendSyncError>` layer so subsequent downcasts,
    /// narrowing, and recovery see its underlying concrete type. Other roots
    /// are simply erased. Context, location, and backtrace are preserved.
    /// Unlike `into()` or `any_union()`, this unwraps the catch-all rather than
    /// only erasing the error set. Requires `alloc`.
    /// The underlying error retains its box, even if it could fit inline.
    /// Named catch-all newtypes from [`Self::into_typed`] remain intact.
    pub fn into_untyped(self) -> ErrorUnion {
        if self.is_inner::<Box<dyn SendSyncError>>() {
            // SAFETY: the concrete root type was checked above.
            let parts = unsafe {
                self.inner
                    .downcast_error_unchecked_with_parts::<Box<dyn SendSyncError>>()
            };
            ErrorUnion {
                inner: ErrorUnionInner::from_boxed_parts(parts),
                _pd: PhantomData,
            }
        } else {
            ErrorUnion::erase(self)
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
    /// Context, the original Eros capture location, and the saved Eros backtrace
    /// are preserved. No new backtrace or location is captured.
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
    /// assert_eq!(error.to_string(), "cannot open configuration <- permission denied");
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
                #[cfg(feature = "context")]
                parts.context,
                #[cfg(feature = "location")]
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
        #[cfg(feature = "location")]
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
        #[cfg(feature = "location")]
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
