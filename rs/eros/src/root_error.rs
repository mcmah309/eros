use crate::type_set::SupersetOf;
use crate::{ErrorUnion, InnerError, SendSyncError, TypeSet};
use core::marker::PhantomData;

impl<E: TypeSet> ErrorUnion<E> {
    /// Maps every inner error through an owned [`InnerError`].
    ///
    /// The callback always runs, even if the original already has the replacement
    /// type. It receives ownership of the original concrete error and returns
    /// the replacement. The returned union has that type as its single variant.
    /// All saved diagnostics are preserved without capturing new ones.
    ///
    /// Inspection, mutation, and owned downcasting work without allocation. The
    /// callback type is identical with every feature configuration. The
    /// replacement follows [`Self::new`]'s storage limits. For a singleton union,
    /// [`Self::map_single`] passes the concrete error directly to the callback.
    ///
    /// ```
    /// use eros::{ErrorUnion, InnerError};
    /// use core::fmt;
    /// fn replace(inner: InnerError) -> fmt::Error {
    ///     assert!(inner.is::<fmt::Error>());
    ///     fmt::Error
    /// }
    /// let error: ErrorUnion<(fmt::Error,)> = ErrorUnion::new(fmt::Error);
    /// let error = error.map_inner(replace);
    /// assert_eq!(error.into_single(), fmt::Error);
    /// ```
    ///
    /// The original is not automatically added to the source chain. With `alloc`,
    /// convert the owner using `.into()` or `.into_boxed()` to keep it as a boxed
    /// source. A wrapper taking `InnerError` can retain it directly instead:
    ///
    /// ```
    /// # #[cfg(feature = "alloc")]
    /// # {
    /// #[derive(Debug, thiserror::Error)]
    /// #[error("configuration failed: {0}")]
    /// struct ConfigError(#[source] Box<dyn eros::SendSyncError>);
    /// let error = eros::error!("permission denied")
    ///     .map_inner(|inner| ConfigError(inner.into()));
    /// assert_eq!(error.source().unwrap().to_string(), "permission denied");
    /// # }
    /// ```
    #[inline]
    pub fn map_inner<T, F>(self, f: F) -> ErrorUnion<(T,)>
    where
        T: SendSyncError,
        F: FnOnce(InnerError) -> T,
    {
        ErrorUnion {
            inner: self.inner.map_error(f),
            _pd: PhantomData,
        }
    }

    /// Maps every inner error and widens to the destination error set.
    ///
    /// Equivalent to `self.map_inner(f).widen()`. The destination is inferred
    /// from the surrounding type and must contain the replacement type. The
    /// callback always runs, including when the original already belongs to
    /// the destination set. All saved diagnostics are preserved, just as with
    /// [`Self::map_inner`]. Available with every feature configuration.
    ///
    /// ```
    /// use eros::ErrorUnion;
    /// use core::{fmt, num::ParseIntError};
    /// let error: ErrorUnion = ErrorUnion::new(fmt::Error);
    /// let error: ErrorUnion<(ParseIntError, fmt::Error)> = error.collapse(|inner| {
    ///     assert!(inner.is::<fmt::Error>());
    ///     fmt::Error
    /// });
    /// assert!(error.is_inner::<fmt::Error>());
    /// ```
    ///
    /// The destination cannot omit the replacement type:
    ///
    /// ```compile_fail
    /// use eros::ErrorUnion;
    /// use core::{fmt, num::ParseIntError};
    /// let error: ErrorUnion = ErrorUnion::new(fmt::Error);
    /// let _: ErrorUnion<(ParseIntError,)> = error.collapse(|_| fmt::Error);
    /// ```
    #[inline]
    pub fn collapse<Other, T, Index>(self, f: impl FnOnce(InnerError) -> T) -> ErrorUnion<Other>
    where
        Other: TypeSet,
        T: SendSyncError,
        Other::Variants: SupersetOf<<(T,) as TypeSet>::Variants, Index>,
    {
        self.map_inner(f).widen()
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    #[cfg(feature = "backtrace")]
    use alloc::string::ToString;

    use crate::{ErrorUnion, MsgError};

    #[test]
    fn repeated_mapping_preserves_metadata() {
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
        let _error: ErrorUnion<(core::fmt::Error, MsgError)> = error.collapse(|_| core::fmt::Error);

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
