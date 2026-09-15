use core::{any::Any, marker::PhantomData};

use crate::type_set::{IsFold, Narrow, SupersetOf, TupleForm};
use crate::{ErrorUnion, TypeSet};

mod sealed {
    pub struct Token;
}

/// Inferred proof marker for extracting a concrete error.
pub struct SingleNarrow<Index>(PhantomData<Index>);
/// Inferred proof marker for selecting an error set with its diagnostics.
pub struct GroupNarrow<Index>(PhantomData<Index>);

/// Sealed selection of a narrowing result and its remaining error set.
///
/// A concrete target returns that error. A tuple target returns an
/// [`ErrorUnion`] containing the selected types and the original diagnostics.
pub trait NarrowTarget<E: TypeSet, Index> {
    type Output;
    type Remainder: TupleForm;

    fn split(
        error: ErrorUnion<E>,
    ) -> Result<Self::Output, ErrorUnion<<Self::Remainder as TupleForm>::Tuple>>;

    #[doc(hidden)]
    fn __seal(_: sealed::Token);
}

impl<Target, E, Index> NarrowTarget<E, SingleNarrow<Index>> for Target
where
    Target: 'static,
    E: TypeSet,
    E::Variants: Narrow<Target, Index>,
{
    type Output = Target;
    type Remainder = <E::Variants as Narrow<Target, Index>>::Remainder;

    fn split(
        error: ErrorUnion<E>,
    ) -> Result<Self::Output, ErrorUnion<<Self::Remainder as TupleForm>::Tuple>> {
        if error.inner.is_error::<Target>() {
            // SAFETY: The stored error's concrete type was checked above.
            Ok(unsafe { error.inner.downcast_error_unchecked::<Target>() })
        } else {
            Err(ErrorUnion {
                inner: error.inner,
                _pd: PhantomData,
            })
        }
    }

    fn __seal(_: sealed::Token) {}
}

impl<Target, E, Index> NarrowTarget<E, GroupNarrow<Index>> for Target
where
    Target: TypeSet,
    E: TypeSet,
    E::Variants: SupersetOf<Target::Variants, Index>,
{
    type Output = ErrorUnion<Target>;
    type Remainder = <E::Variants as SupersetOf<Target::Variants, Index>>::Remainder;

    fn split(
        error: ErrorUnion<E>,
    ) -> Result<Self::Output, ErrorUnion<<Self::Remainder as TupleForm>::Tuple>> {
        if Target::Variants::is_fold(&error.inner.error as &dyn Any) {
            Ok(ErrorUnion {
                inner: error.inner,
                _pd: PhantomData,
            })
        } else {
            Err(ErrorUnion {
                inner: error.inner,
                _pd: PhantomData,
            })
        }
    }

    fn __seal(_: sealed::Token) {}
}
