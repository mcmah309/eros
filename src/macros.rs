/// `format!` like macro to return early from a function with a `TracedError`
#[macro_export]
macro_rules! bail {
    ($($error:tt)+) => {
        return Err(eros::TracedError::new(Box::new(eros::StrError::Owned(format!($($error)*)))));
    };
}

/// `format!` like macro to create a `TracedError`
#[macro_export]
macro_rules! traced {
    ($($error:tt)+) => {
        eros::TracedError::new(Box::new(eros::StrError::Owned(format!($($error)*))))
    };
}
