/// The errors other than the concrete types listed in an error set.
///
/// Put this marker last in a tuple, for example
/// `ErrorUnion<(std::io::Error, OtherError)>`. Such a union accepts any
/// [`SendSyncError`](crate::SendSyncError), while keeping the listed types
/// available for typed handling. Tuples support up to 26 entries, including
/// the marker.
///
/// `narrow::<OtherError, _>()` selects an error only when its concrete inner
/// type is absent from the other entries. The selected error is returned as
/// an [`ErrorUnion`](crate::ErrorUnion) with [`AnyError`](crate::AnyError),
/// preserving diagnostics. The remainder contains only the listed types.
/// `recover` and `try_recover` support the same target.
///
/// The selected union is erased: it does not encode which types were excluded.
/// Removing a concrete type first also removes it from later exclusion checks.
/// Only the inner error's type is checked, never its sources or contexts.
///
/// This marker cannot be constructed or extracted as a concrete error.
/// A tuple target containing it selects the whole open set, whereas the bare
/// `OtherError` target selects only the other errors.
///
/// ```
/// use eros::{ErrorUnion, OtherError, ReshapeUnion};
/// use core::{fmt, num::ParseIntError};
///
/// let error: ErrorUnion<(ParseIntError, OtherError)> = ErrorUnion::new(fmt::Error);
/// let other: ErrorUnion = error.narrow::<OtherError, _>().unwrap();
/// assert!(other.is_inner::<fmt::Error>());
///
/// let result: eros::Result<u16, (ParseIntError, OtherError)> =
///     Err(ErrorUnion::new(fmt::Error));
/// let result: eros::Result<u16, (ParseIntError,)> =
///     result.recover::<OtherError, _>(|error| {
///         assert!(error.is_inner::<fmt::Error>());
///         8080
///     });
/// assert_eq!(result.unwrap(), 8080);
/// ```
pub enum OtherError {}
