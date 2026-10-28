/// Returns early with an [`ErrorUnion`](crate::ErrorUnion) wrapped in `Err`.
///
/// The error set is inferred from the return type. Typed results must include
/// the supplied error type.
///
/// ```
/// fn validate(port: u16) -> eros::Result<()> {
///     if port == 0 {
///         eros::bail!("port must be nonzero");
///     }
///     Ok(())
/// }
/// assert!(validate(0).is_err());
/// ```
///
/// See [`error!`](crate::error!) for more syntax information.
#[macro_export]
macro_rules! bail {
    ($err:expr $(,)?) => {{
        let error = $crate::__private::format_error!($crate, $err);
        return $crate::Result::Err($crate::ErrorUnion::new(error))
    }};
    ($fmt:expr, $($arg:tt)*) => {
        return $crate::Result::Err($crate::ErrorUnion::new(
            $crate::__private::formatted_message!($fmt, $($arg)*)
        ))
    };
}

/// Creates an [`ErrorUnion`](crate::ErrorUnion) from a message or error.
///
/// The returned union uses [`AnyError`](crate::AnyError). [`bail!`](crate::bail!)
/// and [`ensure!`](crate::ensure!) instead infer the error set from the return type.
///
/// Plain string literals work without `alloc`:
/// ```
/// let error = eros::error!("User not found");
/// assert_eq!(error.to_string(), "User not found");
/// ```
/// With `alloc`, string literals also support runtime formatting:
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
/// With `alloc`, plain literals use [`MsgError::from_static_ref`](crate::MsgError::from_static_ref),
/// while formatted messages use `MsgError::from_owned`. Without `alloc`, each
/// plain literal expands to an anonymous zero-sized error whose `Display` and
/// `Debug` write the static text. These anonymous types use erased error sets
/// and cannot be downcast to `MsgError`. Runtime formatting requires `alloc`.
/// Escape literal braces as `{{` and `}}`, just like `format!`.
///
/// Wrap string constants and variables explicitly in [`MsgError`](crate::MsgError).
/// Static messages borrow the text and keep it unchanged:
/// ```
/// static NOT_FOUND: &str = "User {id} not found";
/// let error = eros::error!(eros::MsgError::from_static_ref(&NOT_FOUND));
/// assert_eq!(error.to_string(), "User {id} not found");
/// ```
///
/// Other expressions wrap the original error:
/// ```
/// let source = std::fmt::Error;
/// let error = eros::error!(source);
/// assert!(error.is_inner::<std::fmt::Error>());
/// ```
#[macro_export]
macro_rules! error {
    ($msg:expr $(,)?) => {{
        let error = $crate::__private::format_error!($crate, $msg);
        $crate::ErrorUnion::new::<_, $crate::AnyError, _>(error)
    }};
    ($fmt:expr, $($arg:tt)*) => {
        $crate::ErrorUnion::new::<_, $crate::AnyError, _>($crate::__private::formatted_message!($fmt, $($arg)*))
    };
}

/// Returns an [`ErrorUnion`](crate::ErrorUnion) error if the condition is false.
///
/// The error set is inferred from the return type, as with [`bail!`](crate::bail!).
/// The condition is evaluated once; the error is constructed only on failure.
///
/// ```
/// fn validate(port: u16) -> eros::Result<()> {
///     eros::ensure!(port != 0, "port must be nonzero");
///     Ok(())
/// }
/// assert!(validate(0).is_err());
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

#[doc(hidden)]
#[cfg(feature = "alloc")]
#[macro_export]
macro_rules! __eros_format_message {
    ($($argument:tt)*) => {
        $crate::MsgError::from_owned($crate::__private::format!($($argument)*))
    };
}

#[doc(hidden)]
#[cfg(not(feature = "alloc"))]
#[macro_export]
macro_rules! __eros_format_message {
    ($($argument:tt)*) => {
        compile_error!("formatted error messages require the eros alloc feature")
    };
}

#[doc(hidden)]
#[cfg(feature = "alloc")]
#[macro_export]
macro_rules! __eros_static_message {
    ($message:literal) => {
        $crate::MsgError::from_static_ref(&$message)
    };
}

#[doc(hidden)]
#[cfg(not(feature = "alloc"))]
#[macro_export]
macro_rules! __eros_static_message {
    ($message:literal) => {{
        struct __ErosMessage;
        impl ::core::fmt::Display for __ErosMessage {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str($message)
            }
        }
        impl ::core::fmt::Debug for __ErosMessage {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Display::fmt(self, f)
            }
        }
        impl ::core::error::Error for __ErosMessage {}
        __ErosMessage
    }};
}
