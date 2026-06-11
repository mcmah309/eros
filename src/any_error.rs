use crate::{SendSyncError, TracedUnion};

/// A marker type for `TracedUnion` representing all possible errors
#[derive(Debug)]
pub struct AnyError;

//************************************************************************//

impl<A> From<A> for TracedUnion<AnyError>
where
    A: SendSyncError,
{
    fn from(value: A) -> Self {
        TracedUnion::new(value)
    }
}

// (A,)
//************************************************************************//

impl<T, A> From<T> for TracedUnion<(A,)>
where
    T: SendSyncError + Into<A>,
    A: SendSyncError,
{
    fn from(value: T) -> Self {
        TracedUnion::new(value.into())
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

// impl<A, B> From<A> for TracedUnion<(A, B)>
// where
//     A: SendSyncError,
//     B: SendSyncError,
// {
//     fn from(value: A) -> Self {
//         TracedUnion::new(value)
//     }
// }

// impl<A, B> From<TracedUnion<(A,)>> for TracedUnion<(A, B)>
// where
//     A: SendSyncError,
//     B: SendSyncError,
// {
//     fn from(value: TracedUnion<(A,)>) -> Self {
//         value.widen()
//     }
// }

// Conflicts with above
// impl<A, B> From<TracedUnion<(B,)>> for TracedUnion<(A, B)>
// where
//     A: SendSyncError,
//     B: SendSyncError,
// {
//     fn from(value: TracedUnion<(B,)>) -> Self {
//         value.widen()
//     }
// }

// Convert explicit to any with just into
//************************************************************************//

impl<A> From<TracedUnion<(A,)>> for TracedUnion<AnyError>
where
    A: SendSyncError,
{
    fn from(value: TracedUnion<(A,)>) -> Self {
        TracedUnion::erase(value)
    }
}

impl<A, B> From<TracedUnion<(A, B)>> for TracedUnion<AnyError>
where
    A: SendSyncError,
    B: SendSyncError,
{
    fn from(value: TracedUnion<(A, B)>) -> Self {
        TracedUnion::erase(value)
    }
}

impl<A, B, C> From<TracedUnion<(A, B, C)>> for TracedUnion<AnyError>
where
    A: SendSyncError,
    B: SendSyncError,
    C: SendSyncError,
{
    fn from(value: TracedUnion<(A, B, C)>) -> Self {
        TracedUnion::erase(value)
    }
}

impl<A, B, C, D> From<TracedUnion<(A, B, C, D)>> for TracedUnion<AnyError>
where
    A: SendSyncError,
    B: SendSyncError,
    C: SendSyncError,
    D: SendSyncError,
{
    fn from(value: TracedUnion<(A, B, C, D)>) -> Self {
        TracedUnion::erase(value)
    }
}

impl<A, B, C, D, E> From<TracedUnion<(A, B, C, D, E)>> for TracedUnion<AnyError>
where
    A: SendSyncError,
    B: SendSyncError,
    C: SendSyncError,
    D: SendSyncError,
    E: SendSyncError,
{
    fn from(value: TracedUnion<(A, B, C, D, E)>) -> Self {
        TracedUnion::erase(value)
    }
}

impl<A, B, C, D, E, F> From<TracedUnion<(A, B, C, D, E, F)>> for TracedUnion<AnyError>
where
    A: SendSyncError,
    B: SendSyncError,
    C: SendSyncError,
    D: SendSyncError,
    E: SendSyncError,
    F: SendSyncError,
{
    fn from(value: TracedUnion<(A, B, C, D, E, F)>) -> Self {
        TracedUnion::erase(value)
    }
}

impl<A, B, C, D, E, F, G> From<TracedUnion<(A, B, C, D, E, F, G)>> for TracedUnion<AnyError>
where
    A: SendSyncError,
    B: SendSyncError,
    C: SendSyncError,
    D: SendSyncError,
    E: SendSyncError,
    F: SendSyncError,
    G: SendSyncError,
{
    fn from(value: TracedUnion<(A, B, C, D, E, F, G)>) -> Self {
        TracedUnion::erase(value)
    }
}

impl<A, B, C, D, E, F, G, H> From<TracedUnion<(A, B, C, D, E, F, G, H)>> for TracedUnion<AnyError>
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
    fn from(value: TracedUnion<(A, B, C, D, E, F, G, H)>) -> Self {
        TracedUnion::erase(value)
    }
}

impl<A, B, C, D, E, F, G, H, I> From<TracedUnion<(A, B, C, D, E, F, G, H, I)>>
    for TracedUnion<AnyError>
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
    fn from(value: TracedUnion<(A, B, C, D, E, F, G, H, I)>) -> Self {
        TracedUnion::erase(value)
    }
}
