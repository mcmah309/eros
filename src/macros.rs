/// `format!` like macro to return early from a function with a [`crate::ErrorUnion`]
#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return $crate::Result::Err($crate::traced!($msg))
    };
    ($err:expr $(,)?) => {
        return $crate::Result::Err($crate::traced!($err))
    };
    ($fmt:expr, $($arg:tt)*) => {
        return $crate::Result::Err($crate::traced!($fmt, $($arg)*))
    };
}

/// `format!` like macro to create a [`crate::ErrorUnion`]
#[macro_export]
macro_rules! traced {
    ($msg:literal $(,)?) => {
        $crate::ErrorUnion::new::<_, eros::AnyError, _>($crate::StrContext::Static($msg))
    };
    ($err:expr $(,)?) => {
        $crate::ErrorUnion::new::<_, eros::AnyError, _>($err)
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::ErrorUnion::new::<_, eros::AnyError, _>($crate::StrContext::Owned(format!($fmt, $($arg)*)))
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