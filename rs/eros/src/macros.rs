/// Returns early with an [`ErrorUnion`](crate::ErrorUnion) wrapped in `Err`.
///
/// The error set is inferred from the return type. Typed results must include
/// the supplied error type.
///
/// ```
/// # #[cfg(feature = "alloc")]
/// # {
/// fn validate(port: u16) -> eros::Result<(), (eros::MsgError,)> {
///     if port == 0 {
///         eros::bail!("port must be nonzero");
///     }
///     Ok(())
/// }
/// assert!(validate(0).is_err());
/// # }
/// ```
///
/// See [`error!`](crate::error!) for more syntax information.
#[macro_export]
#[cfg(feature = "alloc")]
macro_rules! bail {
    ($err:expr $(,)?) => {{
        let error = $crate::__private::format_error!($crate, $err);
        return $crate::Result::Err($crate::ErrorUnion::new(error))
    }};
    ($fmt:expr, $($arg:tt)*) => {
        return $crate::Result::Err($crate::ErrorUnion::new(
            $crate::MsgError::from_owned($crate::__private::format!($fmt, $($arg)*))
        ))
    };
}

/// Creates an [`ErrorUnion`](crate::ErrorUnion) from a message or error.
///
/// The returned union uses [`AnyError`](crate::AnyError). [`bail!`](crate::bail!)
/// and [`ensure!`](crate::ensure!) instead infer the error set from the return type.
///
/// String literals work like `format!`:
/// ```
/// # #[cfg(feature = "alloc")]
/// # {
/// let id = 7;
/// eros::error!("User not found");
/// eros::error!("User {} not found", id);
/// let error = eros::error!("User {id} not found");
/// assert_eq!(error.to_string(), "User 7 not found");
/// # }
/// ```
/// `error!` optimizes allocations unlike `format!`.
/// Plain literals use [`MsgError::from_static`](crate::MsgError::from_static); formatted
/// messages use [`MsgError::from_owned`](crate::MsgError::from_owned). Storage is selected
/// at compile time. Escape literal braces as `{{` and `}}`, just like `format!`.
///
/// Wrap string constants and variables explicitly in [`MsgError`](crate::MsgError).
/// Static messages borrow the text and keep it unchanged:
/// ```
/// # #[cfg(feature = "alloc")]
/// # {
/// static NOT_FOUND: &str = "User {id} not found";
/// let error = eros::error!(eros::MsgError::from_static(NOT_FOUND));
/// assert_eq!(error.to_string(), "User {id} not found");
/// # }
/// ```
///
/// Other expressions wrap the original error:
/// ```
/// # #[cfg(feature = "alloc")]
/// # {
/// let source = std::fmt::Error;
/// let error = eros::error!(source);
/// assert!(error.is_inner::<std::fmt::Error>());
/// # }
/// ```
#[macro_export]
#[cfg(feature = "alloc")]
macro_rules! error {
    ($msg:expr $(,)?) => {{
        let error = $crate::__private::format_error!($crate, $msg);
        $crate::ErrorUnion::new::<_, $crate::AnyError, _>(error)
    }};
    ($fmt:expr, $($arg:tt)*) => {
        $crate::ErrorUnion::new::<_, $crate::AnyError, _>($crate::MsgError::from_owned($crate::__private::format!($fmt, $($arg)*)))
    };
}

/// Returns an [`ErrorUnion`](crate::ErrorUnion) error if the condition is false.
///
/// The error set is inferred from the return type, as with [`bail!`](crate::bail!).
/// The condition is evaluated once; the error is constructed only on failure.
///
/// ```
/// # #[cfg(feature = "alloc")]
/// # {
/// fn validate(port: u16) -> eros::Result<(), (eros::MsgError,)> {
///     eros::ensure!(port != 0, "port must be nonzero");
///     Ok(())
/// }
/// assert!(validate(0).is_err());
/// # }
/// ```
///
/// See [`error!`](crate::error!) for more syntax information.
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

/// Returns early with a zero-sized concrete error. String messages require `alloc`.
#[cfg(not(feature = "alloc"))]
#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        compile_error!("string error messages require the eros alloc feature")
    };
    ($err:expr $(,)?) => {
        return $crate::Result::Err($crate::ErrorUnion::new($err))
    };
    ($fmt:expr, $($arg:tt)*) => {
        compile_error!("formatted error messages require the eros alloc feature")
    };
}

/// Creates an erased union containing a zero-sized concrete error.
/// String messages require the `alloc` feature.
#[cfg(not(feature = "alloc"))]
#[macro_export]
macro_rules! error {
    ($msg:literal $(,)?) => {
        compile_error!("string error messages require the eros alloc feature")
    };
    ($err:expr $(,)?) => {
        $crate::ErrorUnion::new::<_, $crate::AnyError, _>($err)
    };
    ($fmt:expr, $($arg:tt)*) => {
        compile_error!("formatted error messages require the eros alloc feature")
    };
}
