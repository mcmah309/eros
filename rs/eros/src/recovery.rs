use core::marker::PhantomData;

use crate::narrowing::OtherNarrow;
use crate::type_set::{GroupNarrow, IsFold, Open, SupersetOf, TupleForm};
use crate::{AnyError, ErrorUnion, OtherError, SendSyncError, TypeSet};

mod sealed {
    pub struct Token;
}

/// Inferred proof marker for recovery of a single concrete error type.
pub struct SingleRecovery<Index>(PhantomData<Index>);
/// Inferred proof marker for recovery of a tuple of error types.
pub struct GroupRecovery<Index>(PhantomData<Index>);
/// Inferred proof marker for recovery of unnamed errors from an open set.
pub struct OtherRecovery;

/// Sealed selection of a recovery handler's error set and its remainder.
pub trait RecoveryTarget<E: TypeSet, Index> {
    type Selected: TypeSet;
    type Remainder: TupleForm;

    fn split(
        error: ErrorUnion<E>,
    ) -> Result<ErrorUnion<Self::Selected>, ErrorUnion<<Self::Remainder as TupleForm>::Tuple>>;

    #[doc(hidden)]
    fn __seal(_: sealed::Token);
}

/// Connects a handler's argument type to its recovery target for inference.
///
/// Separate implementations let annotated closures infer either a concrete
/// single-error target or a tuple target. An associated type alone would not
/// allow Rust to infer the target from the closure argument. This trait is sealed.
pub trait RecoveryHandler<Target, E, Index, Output>:
    FnOnce(ErrorUnion<Target::Selected>) -> Output
where
    E: TypeSet,
    Target: RecoveryTarget<E, Index>,
{
    #[doc(hidden)]
    fn __seal(_: sealed::Token);
}

impl<Target, E, Index> RecoveryTarget<E, SingleRecovery<Index>> for Target
where
    Target: SendSyncError,
    E: TypeSet,
    E::Variants: SupersetOf<<(Target,) as TypeSet>::Variants, Index>,
{
    type Selected = (Target,);
    type Remainder =
        <E::Variants as SupersetOf<<Self::Selected as TypeSet>::Variants, Index>>::Remainder;

    fn split(
        error: ErrorUnion<E>,
    ) -> Result<ErrorUnion<Self::Selected>, ErrorUnion<<Self::Remainder as TupleForm>::Tuple>> {
        error.narrow::<(Target,), GroupNarrow<Index>>()
    }

    fn __seal(_: sealed::Token) {}
}

impl<F, Target, E, Index, Output> RecoveryHandler<Target, E, SingleRecovery<Index>, Output> for F
where
    Target: SendSyncError,
    E: TypeSet,
    E::Variants: SupersetOf<<(Target,) as TypeSet>::Variants, Index>,
    F: FnOnce(ErrorUnion<(Target,)>) -> Output,
{
    fn __seal(_: sealed::Token) {}
}

impl<E, Types> RecoveryTarget<E, OtherRecovery> for OtherError
where
    E: TypeSet<Variants = Open<Types>>,
    Types: TupleForm + IsFold,
{
    type Selected = AnyError;
    type Remainder = Types;

    fn split(error: ErrorUnion<E>) -> Result<ErrorUnion, ErrorUnion<Types::Tuple>> {
        error.narrow::<OtherError, OtherNarrow>()
    }

    fn __seal(_: sealed::Token) {}
}

impl<F, E, Types, Output> RecoveryHandler<OtherError, E, OtherRecovery, Output> for F
where
    E: TypeSet<Variants = Open<Types>>,
    Types: TupleForm + IsFold,
    F: FnOnce(ErrorUnion) -> Output,
{
    fn __seal(_: sealed::Token) {}
}

// Unlike (T,) for a concrete error, (OtherError,) has no single-error recovery
// implementation. Its tuple target selects the whole open set.
impl<E, Index> RecoveryTarget<E, GroupRecovery<Index>> for (OtherError,)
where
    E: TypeSet,
    E::Variants: SupersetOf<<(OtherError,) as TypeSet>::Variants, Index>,
{
    type Selected = Self;
    type Remainder = <E::Variants as SupersetOf<<Self as TypeSet>::Variants, Index>>::Remainder;

    fn split(
        error: ErrorUnion<E>,
    ) -> Result<ErrorUnion<Self>, ErrorUnion<<Self::Remainder as TupleForm>::Tuple>> {
        error.narrow::<Self, GroupNarrow<Index>>()
    }

    fn __seal(_: sealed::Token) {}
}

impl<F, E, Index, Output> RecoveryHandler<(OtherError,), E, GroupRecovery<Index>, Output> for F
where
    E: TypeSet,
    E::Variants: SupersetOf<<(OtherError,) as TypeSet>::Variants, Index>,
    F: FnOnce(ErrorUnion<(OtherError,)>) -> Output,
{
    fn __seal(_: sealed::Token) {}
}

macro_rules! group_recovery {
    ($($member:ident),+) => {
        impl<SourceErrors, Index, $($member),+> RecoveryTarget<SourceErrors, GroupRecovery<Index>> for ($($member,)+)
        where
            ($($member,)+): TypeSet,
            SourceErrors: TypeSet,
            SourceErrors::Variants: SupersetOf<<Self as TypeSet>::Variants, Index>,
        {
            type Selected = Self;
            type Remainder = <SourceErrors::Variants as SupersetOf<<Self as TypeSet>::Variants, Index>>::Remainder;

            fn split(error: ErrorUnion<SourceErrors>) -> Result<ErrorUnion<Self>, ErrorUnion<<Self::Remainder as TupleForm>::Tuple>> {
                error.narrow::<Self, GroupNarrow<Index>>()
            }

            fn __seal(_: sealed::Token) {}
        }

        impl<Handler, Errors, Index, Output, $($member),+> RecoveryHandler<($($member,)+), Errors, GroupRecovery<Index>, Output> for Handler
        where
            ($($member,)+): TypeSet,
            Errors: TypeSet,
            Errors::Variants: SupersetOf<<($($member,)+) as TypeSet>::Variants, Index>,
            Handler: FnOnce(ErrorUnion<($($member,)+)>) -> Output,
        {
            fn __seal(_: sealed::Token) {}
        }
    };
}

// Singleton handler arguments already infer the concrete error target. Giving
// (T,) a second target implementation would make that inference ambiguous.
group_recovery!(A, B);
group_recovery!(A, B, C);
group_recovery!(A, B, C, D);
group_recovery!(A, B, C, D, E);
group_recovery!(A, B, C, D, E, F);
group_recovery!(A, B, C, D, E, F, G);
group_recovery!(A, B, C, D, E, F, G, H);
group_recovery!(A, B, C, D, E, F, G, H, I);
group_recovery!(A, B, C, D, E, F, G, H, I, J);
group_recovery!(A, B, C, D, E, F, G, H, I, J, K);
group_recovery!(A, B, C, D, E, F, G, H, I, J, K, L);
group_recovery!(A, B, C, D, E, F, G, H, I, J, K, L, M);
group_recovery!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
group_recovery!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
group_recovery!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
group_recovery!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
group_recovery!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
group_recovery!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
group_recovery!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
group_recovery!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U
);
group_recovery!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V
);
group_recovery!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W
);
group_recovery!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X
);
group_recovery!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y
);
group_recovery!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);
