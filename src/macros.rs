/// `format!` like macro to return early from a function with a [`crate::TracedError`]
#[macro_export]
macro_rules! bail {
    ($error:literal) => {
        return Err($crate::TracedError::boxed($crate::StrError::Static($error)))
    };
    ($($error:tt)+) => {
        return Err($crate::TracedError::boxed($crate::StrError::Owned(format!($($error)*))));
    };
}

/// `format!` like macro to create a [`crate::TracedError`]
#[macro_export]
macro_rules! traced {
    ($error:literal) => {
        $crate::TracedError::boxed($crate::StrError::Static($error))
    };
    ($($error:tt)+) => {
        $crate::TracedError::boxed($crate::StrError::Owned(format!($($error)*)))
    };
}

/// `assert!` like macro for bailing on a condition failure
#[macro_export]
macro_rules! ensure {
    ($test:expr, $error:literal) => {
        if !($test) {
            $crate::bail!($error)
        }
    };
    ($test:expr, $($error:tt)+) => {
        if !($test) {
            $crate::bail!($($error)*)
        }
    };
}