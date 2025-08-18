
#[macro_export]
macro_rules! bail {
    ($error:expr) => {
        return Err(eros::GenericCtxError::new(std::convert::identity::<eros::GenericError>($error.into())));
    };
    ($($error:tt)+) => {
        return Err(eros::GenericCtxError::new(std::convert::identity::<eros::GenericError>(format!($($error)*).into())));
    };
}

#[macro_export]
macro_rules! eros {
    ($error:expr) => {
        eros::GenericCtxError::new(std::convert::identity::<eros::GenericError>($error).into());
    };
    ($($error:tt)+) => {
        eros::GenericCtxError::new(std::convert::identity::<eros::GenericError>(format!($($error)*).into()))
    };
}