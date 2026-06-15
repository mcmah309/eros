use crate::{ErrorUnion, SendSyncError};

/// A marker type for `ErrorUnion` representing all possible errors
// Note: `AnyError` is not constructable. If so we could have correctness issues
pub enum AnyError {}

impl std::fmt::Display for AnyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!(
            "AnyError is a marker type and it should not be possible to be constructed or used directly."
        )
    }
}

impl std::fmt::Debug for AnyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!(
            "AnyError is a marker type and it should not be possible to be constructed or used directly."
        )
    }
}

//************************************************************************//

impl<A> From<A> for ErrorUnion<AnyError>
where
    A: SendSyncError,
{
    #[cfg_attr(feature = "location", track_caller)]
    fn from(value: A) -> Self {
        ErrorUnion::new(value)
    }
}

// (A,)
//************************************************************************//

impl<T, A> From<T> for ErrorUnion<(A,)>
where
    T: SendSyncError + Into<A>,
    A: SendSyncError,
{
    #[cfg_attr(feature = "location", track_caller)]
    fn from(value: T) -> Self {
        ErrorUnion::new(value.into())
    }
}

// Note we do not implement things like below and keep the pattern going up to 9,
// because `into()` would only work/be implemented for one position and we can't implement
// for all positions otherwise we would get conflicting From implementations.
// I have also not found a generic way to implement from a Subset to a Superset (in all positions),
// either explictly writing out each From case or with a combination of the type system.
// Such an implementation may be possible and would likely utilize the traits `SupersetOf` or `Contains`.
// There instead of just implementing for one position, we don't implement for any. The recommended
// procedure then becomes use `union` or `widen` rather than `into`. Unfortunately this means we need
// to be explicit and `?` won't make an implicit conversion.
//************************************************************************//

// impl<A, B> From<A> for ErrorUnion<(A, B)>
// where
//     A: SendSyncError,
//     B: SendSyncError,
// {
//     fn from(value: A) -> Self {
//         ErrorUnion::new(value)
//     }
// }

// impl<A, B> From<ErrorUnion<(A,)>> for ErrorUnion<(A, B)>
// where
//     A: SendSyncError,
//     B: SendSyncError,
// {
//     fn from(value: ErrorUnion<(A,)>) -> Self {
//         value.widen()
//     }
// }

// Conflicts with above
// impl<A, B> From<ErrorUnion<(B,)>> for ErrorUnion<(A, B)>
// where
//     A: SendSyncError,
//     B: SendSyncError,
// {
//     fn from(value: ErrorUnion<(B,)>) -> Self {
//         value.widen()
//     }
// }

// Convert explicit to any with just into
//************************************************************************//

impl<A> From<ErrorUnion<(A,)>> for ErrorUnion<AnyError>
where
    A: SendSyncError,
{
    fn from(value: ErrorUnion<(A,)>) -> Self {
        ErrorUnion::erase(value)
    }
}

impl<A, B> From<ErrorUnion<(A, B)>> for ErrorUnion<AnyError>
where
    A: SendSyncError,
    B: SendSyncError,
{
    fn from(value: ErrorUnion<(A, B)>) -> Self {
        ErrorUnion::erase(value)
    }
}

impl<A, B, C> From<ErrorUnion<(A, B, C)>> for ErrorUnion<AnyError>
where
    A: SendSyncError,
    B: SendSyncError,
    C: SendSyncError,
{
    fn from(value: ErrorUnion<(A, B, C)>) -> Self {
        ErrorUnion::erase(value)
    }
}

impl<A, B, C, D> From<ErrorUnion<(A, B, C, D)>> for ErrorUnion<AnyError>
where
    A: SendSyncError,
    B: SendSyncError,
    C: SendSyncError,
    D: SendSyncError,
{
    fn from(value: ErrorUnion<(A, B, C, D)>) -> Self {
        ErrorUnion::erase(value)
    }
}

impl<A, B, C, D, E> From<ErrorUnion<(A, B, C, D, E)>> for ErrorUnion<AnyError>
where
    A: SendSyncError,
    B: SendSyncError,
    C: SendSyncError,
    D: SendSyncError,
    E: SendSyncError,
{
    fn from(value: ErrorUnion<(A, B, C, D, E)>) -> Self {
        ErrorUnion::erase(value)
    }
}

impl<A, B, C, D, E, F> From<ErrorUnion<(A, B, C, D, E, F)>> for ErrorUnion<AnyError>
where
    A: SendSyncError,
    B: SendSyncError,
    C: SendSyncError,
    D: SendSyncError,
    E: SendSyncError,
    F: SendSyncError,
{
    fn from(value: ErrorUnion<(A, B, C, D, E, F)>) -> Self {
        ErrorUnion::erase(value)
    }
}

impl<A, B, C, D, E, F, G> From<ErrorUnion<(A, B, C, D, E, F, G)>> for ErrorUnion<AnyError>
where
    A: SendSyncError,
    B: SendSyncError,
    C: SendSyncError,
    D: SendSyncError,
    E: SendSyncError,
    F: SendSyncError,
    G: SendSyncError,
{
    fn from(value: ErrorUnion<(A, B, C, D, E, F, G)>) -> Self {
        ErrorUnion::erase(value)
    }
}

impl<A, B, C, D, E, F, G, H> From<ErrorUnion<(A, B, C, D, E, F, G, H)>> for ErrorUnion<AnyError>
where
    A: SendSyncError,
    B: SendSyncError,
    C: SendSyncError,
    D: SendSyncError,
    E: SendSyncError,
    F: SendSyncError,
    G: SendSyncError,
    H: SendSyncError,
{
    fn from(value: ErrorUnion<(A, B, C, D, E, F, G, H)>) -> Self {
        ErrorUnion::erase(value)
    }
}

impl<A, B, C, D, E, F, G, H, I> From<ErrorUnion<(A, B, C, D, E, F, G, H, I)>>
    for ErrorUnion<AnyError>
where
    A: SendSyncError,
    B: SendSyncError,
    C: SendSyncError,
    D: SendSyncError,
    E: SendSyncError,
    F: SendSyncError,
    G: SendSyncError,
    H: SendSyncError,
    I: SendSyncError,
{
    fn from(value: ErrorUnion<(A, B, C, D, E, F, G, H, I)>) -> Self {
        ErrorUnion::erase(value)
    }
}
