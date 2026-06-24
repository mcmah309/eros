/// `format!` like macro to return early from a function with a [`crate::ErrorUnion`]
#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return $crate::Result::Err($crate::error!($msg))
    };
    ($err:expr $(,)?) => {
        return $crate::Result::Err($crate::error!($err))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return $crate::Result::Err($crate::error!($fmt, $($arg)*))
    };
}

/// `format!` like macro to create a [`crate::ErrorUnion`]
#[macro_export]
macro_rules! error {
    ($msg:literal $(,)?) => {
        $crate::ErrorUnion::new::<_, eros::AnyError, _>($crate::StrError::Static($msg))
    };
    ($err:expr $(,)?) => {
        $crate::ErrorUnion::new::<_, eros::AnyError, _>($err)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::ErrorUnion::new::<_, eros::AnyError, _>($crate::StrError::Owned($crate::__private::format!($fmt, $($arg)*)))
    };
}

/// `assert!` like macro for bailing on a condition failure
#[macro_export]
macro_rules! ensure {
    ($test:expr, $msg:literal $(,)?) => {
        if !($test) {
            $crate::bail!($msg);
        }
    };
    ($test:expr, $err:expr $(,)?) => {
        if !($test) {
            $crate::bail!($err);
        }
    };
    ($test:expr, $fmt:expr, $($arg:tt)*) => {
        if !($test) {
            $crate::bail!($fmt, $($arg)*);
        }
    };
}