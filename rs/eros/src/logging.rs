use crate::{
    ErrorUnion,
    type_set::{DebugFold, DisplayFold, TypeSet},
};

pub trait LogExt<O> {
    fn log_error(self) -> O;
    fn log_warn(self) -> O;
}

impl<T, E> LogExt<Result<T, ErrorUnion<E>>> for Result<T, ErrorUnion<E>>
where
    E: TypeSet,
    <E as TypeSet>::Variants: std::fmt::Debug + DebugFold + std::fmt::Display + DisplayFold,
{
    /// If `Err`, logs this error as "error". The logging backend is configured by feature flag, as well as
    /// if the error is logged as its display or debug version
    fn log_error(self) -> Result<T, ErrorUnion<E>> {
        self.inspect_err(|e| {
            e.log_error();
        })
    }

    /// If `Err`, logs this error as "warn". The logging backend is configured by feature flag, as well as
    /// if the error is logged as its display or debug version
    fn log_warn(self) -> Result<T, ErrorUnion<E>> {
        self.inspect_err(|e| {
            e.log_warn();
        })
    }
}

impl<T> LogExt<Result<T, ErrorUnion>> for Result<T, ErrorUnion> {
    /// If `Err`, logs this error as "error". The logging backend is configured by feature flag, as well as
    /// if the error is logged as its display or debug version
    fn log_error(self) -> Result<T, ErrorUnion> {
        self.inspect_err(|e| {
            e.log_error();
        })
    }

    /// If `Err`, logs this error as "warn". The logging backend is configured by feature flag, as well as
    /// if the error is logged as its display or debug version
    fn log_warn(self) -> Result<T, ErrorUnion> {
        self.inspect_err(|e| {
            e.log_warn();
        })
    }
}

impl<E> ErrorUnion<E>
where
    E: TypeSet,
    <E as TypeSet>::Variants: std::fmt::Debug + DebugFold + std::fmt::Display + DisplayFold,
{
    /// Logs this error as "error". The logging backend is configured by feature flag, as well as
    /// if the error is logged as its display or debug version
    #[cfg(feature = "logging")]
    pub fn log_error(&self) {
        #[cfg(all(
            feature = "log_display",
            not(feature = "log_debug"),
            feature = "tracing"
        ))]
        tracing::error!("{}", self);
        #[cfg(all(feature = "log_debug", feature = "tracing"))]
        tracing::error!("{:#?}", self);
    }

    /// Logs this error as "warn". The logging backend is configured by feature flag, as well as
    /// if the error is logged as its display or debug version
    #[cfg(feature = "logging")]
    pub fn log_warn(&self) {
        #[cfg(all(
            feature = "log_display",
            not(feature = "log_debug"),
            feature = "tracing"
        ))]
        tracing::warn!("{}", self);
        #[cfg(all(feature = "log_debug", feature = "tracing"))]
        tracing::warn!("{:#?}", self);
    }
}

impl ErrorUnion {
    /// Logs this error as "error". The logging backend is configured by feature flag, as well as
    /// if the error is logged as its display or debug version
    #[cfg(feature = "logging")]
    pub fn log_error(&self) {
        #[cfg(all(
            feature = "log_display",
            not(feature = "log_debug"),
            feature = "tracing"
        ))]
        tracing::error!("{}", self);
        #[cfg(all(feature = "log_debug", feature = "tracing"))]
        tracing::error!("{:#?}", self);
    }

    /// Logs this error as "warn". The logging backend is configured by feature flag, as well as
    /// if the error is logged as its display or debug version
    #[cfg(feature = "logging")]
    pub fn log_warn(&self) {
        #[cfg(all(
            feature = "log_display",
            not(feature = "log_debug"),
            feature = "tracing"
        ))]
        tracing::warn!("{}", self);
        #[cfg(all(feature = "log_debug", feature = "tracing"))]
        tracing::warn!("{:#?}", self);
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::E;

    use super::*;
    use crate::any_error::AnyError;

    // Just testing it compiles
    #[test]
    #[should_panic]
    fn test_anyerror_log_error() {
        let error_union: ErrorUnion<AnyError> = todo!();
        error_union.log_error();
        let result = Err::<(), _>(error_union);
        result.log_error();
    }

    // Just testing it compiles
    #[test]
    #[should_panic]
    fn test_anyeror_log_warn() {
        let error_union: ErrorUnion<AnyError> = todo!();
        error_union.log_warn();
        let result = Err::<(), _>(error_union);
        result.log_warn();
    }

    #[test]
    fn test_normal_error_union_log_error() {
        let error_union: ErrorUnion<(std::io::Error,)> =
            ErrorUnion::new(std::io::Error::new(std::io::ErrorKind::Other, "Test error"));
        error_union.log_error();
        let result = Err::<(), _>(error_union);
        result.log_error();
    }
}
