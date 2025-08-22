/// `format!` like macro to return early from a function with a `TracedError`
#[macro_export]
macro_rules! bail {
    ($error:literal) => {
        return Err($crate::TracedError::boxed($crate::StrError::Static($error)))
    };
    ($($error:tt)+) => {
        return Err($crate::TracedError::boxed($crate::StrError::Owned(format!($($error)*))));
    };
}

/// `format!` like macro to create a `TracedError`
#[macro_export]
macro_rules! traced {
    ($error:literal) => {
        $crate::TracedError::boxed($crate::StrError::Static($error))
    };
    ($($error:tt)+) => {
        $crate::TracedError::boxed($crate::StrError::Owned(format!($($error)*)))
    };
}
