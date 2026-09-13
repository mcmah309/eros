use alloc::boxed::Box;
use core::{mem, ptr};

use crate::{ErrorUnion, SendSyncError, TypeSet, error_union::ErrorUnionInner};

impl<E: TypeSet> ErrorUnion<E> {
    /// Replaces the root, passing ownership of the old boxed error to a closure.
    ///
    /// The closure runs once, receives the previous root, and returns the new
    /// concrete error value. Eros handles boxing the replacement. Preserve the
    /// original failure by storing the old root in the new error and returning
    /// it from `Error::source()`. Eros uses the new error's source chain without
    /// automatically adding the old root.
    ///
    /// Context, the original Eros capture location, and the saved Eros backtrace
    /// are preserved. No new backtrace or location is captured.
    ///
    /// The returned union has the replacement type as its single variant.
    ///
    /// Wrap the old root as the source of a new error:
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
    ///     .map_root(|old| ConfigError { source: old });
    ///
    /// assert_eq!(error.to_string(), "cannot open configuration <- permission denied");
    /// assert!(error.is_inner::<ConfigError>());
    /// assert_eq!(error.source().unwrap().to_string(), "permission denied");
    /// ```
    pub fn map_root<T, F>(self, f: F) -> ErrorUnion<(T,)>
    where
        T: SendSyncError,
        F: FnOnce(Box<dyn SendSyncError>) -> T,
    {
        let raw = Box::into_raw(self.inner);
        unsafe {
            // The saved function pointer knows the old error's concrete type
            // and moves it into a box, just as into_inner() does.
            let root = ((*raw).into_box_fn)(ptr::addr_of_mut!((*raw).error));
            #[cfg(feature = "backtrace")]
            let backtrace = ptr::read(ptr::addr_of!((*raw).backtrace));
            #[cfg(feature = "context")]
            let context = ptr::read(ptr::addr_of!((*raw).context));
            #[cfg(feature = "location")]
            let location = ptr::read(ptr::addr_of!((*raw).location));

            // Every owned field has been moved. Free the container without
            // dropping those values, following downcast_error_unchecked.
            // The root and metadata now have normal owners if f panics.
            drop(Box::from_raw(
                raw as *mut mem::ManuallyDrop<ErrorUnionInner<dyn SendSyncError>>,
            ));

            ErrorUnion::new_from_parts(
                f(root),
                #[cfg(feature = "backtrace")]
                backtrace,
                #[cfg(feature = "context")]
                context,
                #[cfg(feature = "location")]
                location,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "backtrace")]
    use alloc::string::ToString;

    use crate::{ErrorUnion, StrError};

    #[test]
    fn replacement_preserves_metadata() {
        #[allow(unused_mut)]
        let mut error: ErrorUnion = ErrorUnion::new(StrError::from("permission denied"));

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

        let error = error.map_root(|_| StrError::from("cannot open configuration"));
        let _error = error.map_root(|_| StrError::from("startup failed"));

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
