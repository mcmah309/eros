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
