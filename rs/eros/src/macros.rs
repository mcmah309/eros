/// Returns early with an [`ErrorUnion`](crate::ErrorUnion) wrapped in `Err`.
///
/// See [`error!`](crate::error!) for more syntax information.
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

/// Creates an [`ErrorUnion`](crate::ErrorUnion) from a message or error.
///
/// String literals work like `format!`:
/// ```
/// let id = 7;
/// eros::error!("User not found");
/// eros::error!("User {} not found", id);
/// let error = eros::error!("User {id} not found");
/// assert_eq!(error.to_string(), "User 7 not found");
/// ```
/// `error!` optimizes allocations unlike `format!`.
/// Plain literals use [`MsgError::Static`](crate::MsgError::Static); formatted
/// messages use [`MsgError::Owned`](crate::MsgError::Owned). Storage is selected
/// at compile time. Escape literal braces as `{{` and `}}`, just like `format!`.
///
/// All-caps names, such as `NOT_FOUND` or `messages::NOT_FOUND`, must refer to a
/// `&'static str`. These messages use `MsgError::Static` and keep their text unchanged:
/// ```
/// static NOT_FOUND: &str = "User {id} not found";
/// let error = eros::error!(NOT_FOUND);
/// assert_eq!(error.to_string(), "User {id} not found");
/// ```
///
/// Other expressions wrap the original error. Put an all-caps error value in a
/// block to bypass the string convention:
/// ```
/// let source = std::fmt::Error;
/// let error = eros::error!(source);
/// assert!(error.is_inner::<std::fmt::Error>());
///
/// const ERROR: std::fmt::Error = std::fmt::Error;
/// let error = eros::error!({ ERROR });
/// assert!(error.is_inner::<std::fmt::Error>());
/// ```
#[macro_export]
macro_rules! error {
    ($msg:expr $(,)?) => {{
        let error = $crate::__private::format_error!($crate, $msg);
        $crate::ErrorUnion::new::<_, $crate::AnyError, _>(error)
    }};
    ($fmt:expr, $($arg:tt)*) => {
        $crate::ErrorUnion::new::<_, $crate::AnyError, _>($crate::MsgError::Owned($crate::__private::format!($fmt, $($arg)*)))
    };
}

/// Returns an [`ErrorUnion`](crate::ErrorUnion) error if the condition is false.
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
