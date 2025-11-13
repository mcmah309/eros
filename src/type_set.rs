use core::any::Any;
use core::fmt;
use std::backtrace::Backtrace;
use std::error::Error;

use crate::StrContext;

/* ------------------------- Helpers ----------------------- */

/// The final element of a type-level Cons list.
#[doc(hidden)]
#[derive(Debug)]
pub enum End {}

impl std::error::Error for End {}

/// A compile-time list of types, similar to other basic functional list structures.
#[doc(hidden)]
#[derive(Debug)]
pub struct Cons<Head: ?Sized, Tail: ?Sized>(core::marker::PhantomData<Head>, Tail);

#[doc(hidden)]
#[derive(Debug)]
pub struct Recurse<Tail: ?Sized>(Tail);

/* ------------------------- std::error::Error support ----------------------- */

pub trait ErrorFold {
    fn source_fold(any: &dyn Any) -> Option<&(dyn Error + 'static)>;
}

impl ErrorFold for End {
    fn source_fold(_: &dyn Any) -> Option<&(dyn Error + 'static)> {
        unreachable!("source_fold called on End");
    }
}

impl<Head, Tail> Error for Cons<Head, Tail>
where
    Head: Error,
    Tail: Error,
{
}

impl<Head, Tail> ErrorFold for Cons<Head, Tail>
where
    Cons<Head, Tail>: Error,
    Head: 'static + Error,
    Tail: ErrorFold,
{
    fn source_fold(any: &dyn Any) -> Option<&(dyn Error + 'static)> {
        if let Some(head_ref) = any.downcast_ref::<Head>() {
            head_ref.source()
        } else {
            Tail::source_fold(any)
        }
    }
}

/* ------------------------- Display support ----------------------- */

impl<Head, Tail> fmt::Display for Cons<Head, Tail>
where
    Head: fmt::Display,
    Tail: fmt::Display,
{
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unreachable!("Display called for Cons which is not constructable")
    }
}

impl fmt::Display for End {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unreachable!("Display::fmt called for an End, which is not constructible.")
    }
}

pub trait DisplayFold {
    fn display_fold(any: &dyn Any, formatter: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl DisplayFold for End {
    fn display_fold(_: &dyn Any, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        unreachable!("display_fold called on End");
    }
}

impl<Head, Tail> DisplayFold for Cons<Head, Tail>
where
    Cons<Head, Tail>: fmt::Display,
    Head: 'static + fmt::Display,
    Tail: DisplayFold,
{
    fn display_fold(any: &dyn Any, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(head_ref) = any.downcast_ref::<Head>() {
            head_ref.fmt(formatter)
        } else {
            Tail::display_fold(any, formatter)
        }
    }
}

/* ------------------------- Debug support ----------------------- */

pub trait DebugFold {
    fn debug_fold(
        any: &dyn Any,
        formatter: &mut fmt::Formatter<'_>,
        #[cfg(feature = "context")] context: &Vec<StrContext>,
        #[cfg(feature = "backtrace")] backtrace: &Backtrace,
    ) -> fmt::Result;
}

impl DebugFold for End {
    fn debug_fold(
        _: &dyn Any,
        _: &mut fmt::Formatter<'_>,
        #[cfg(feature = "context")] context: &Vec<StrContext>,
        #[cfg(feature = "backtrace")] backtrace: &Backtrace,
    ) -> fmt::Result {
        unreachable!("debug_fold called on End");
    }
}

impl<Head, Tail> DebugFold for Cons<Head, Tail>
where
    Cons<Head, Tail>: fmt::Debug,
    Head: 'static + fmt::Debug,
    Tail: DebugFold,
{
    fn debug_fold(
        any: &dyn Any,
        formatter: &mut fmt::Formatter<'_>,
        #[cfg(feature = "context")] context: &Vec<StrContext>,
        #[cfg(feature = "backtrace")] backtrace: &Backtrace,
    ) -> fmt::Result {
        if let Some(head_ref) = any.downcast_ref::<Head>() {
            head_ref.fmt(formatter)?;
            #[cfg(feature = "context")]
            {
                if !context.is_empty() {
                    write!(formatter, "\n\nContext:")?;
                    for context_item in context.iter() {
                        write!(formatter, "\n\t- {}", context_item)?;
                    }
                }
            }
            #[cfg(feature = "backtrace")]
            {
                use std::backtrace::BacktraceStatus;

                if matches!(backtrace.status(), BacktraceStatus::Captured) {
                    write!(formatter, "\n\nBacktrace:\n")?;
                    fmt::Display::fmt(backtrace, formatter)?;
                }
            }
            Ok(())
        } else {
            Tail::debug_fold(
                any,
                formatter,
                #[cfg(feature = "context")]
                context,
                #[cfg(feature = "backtrace")]
                backtrace,
            )
        }
    }
}

/* ------------------------- Any::is support ----------------------- */

pub trait IsFold {
    fn is_fold(any: &dyn Any) -> bool;
}

impl IsFold for End {
    fn is_fold(_: &dyn Any) -> bool {
        false
    }
}

impl<Head, Tail> IsFold for Cons<Head, Tail>
where
    Head: 'static,
    Tail: IsFold,
{
    fn is_fold(any: &dyn Any) -> bool {
        if any.is::<Head>() {
            true
        } else {
            Tail::is_fold(any)
        }
    }
}

/* ------------------------- TypeSet implemented for tuples ----------------------- */

pub trait TypeSet {
    type Variants: TupleForm + ?Sized;
}

impl TypeSet for () {
    type Variants = End;
}

impl<A: ?Sized> TypeSet for (A,) {
    type Variants = Cons<A, End>;
}

impl<A, B: ?Sized> TypeSet for (A, B) {
    type Variants = Cons<A, Cons<B, End>>;
}

impl<A, B, C: ?Sized> TypeSet for (A, B, C) {
    type Variants = Cons<A, Cons<B, Cons<C, End>>>;
}

impl<A, B, C, D: ?Sized> TypeSet for (A, B, C, D) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, End>>>>;
}

impl<A, B, C, D, E: ?Sized> TypeSet for (A, B, C, D, E) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, End>>>>>;
}

impl<A, B, C, D, E, F: ?Sized> TypeSet for (A, B, C, D, E, F) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, End>>>>>>;
}

impl<A, B, C, D, E, F, G: ?Sized> TypeSet for (A, B, C, D, E, F, G) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, End>>>>>>>;
}

impl<A, B, C, D, E, F, G, H: ?Sized> TypeSet for (A, B, C, D, E, F, G, H) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, End>>>>>>>>;
}

impl<A, B, C, D, E, F, G, H, I: ?Sized> TypeSet for (A, B, C, D, E, F, G, H, I) {
    type Variants =
        Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, End>>>>>>>>>;
}

/* ------------------------- TupleForm implemented for TypeSet ----------------------- */

pub trait TupleForm {
    type Tuple: TypeSet + ?Sized;
}

impl TupleForm for End {
    type Tuple = ();
}

impl<A: ?Sized> TupleForm for Cons<A, End> {
    type Tuple = (A,);
}

impl<A, B: ?Sized> TupleForm for Cons<A, Cons<B, End>> {
    type Tuple = (A, B);
}

impl<A, B, C: ?Sized> TupleForm for Cons<A, Cons<B, Cons<C, End>>> {
    type Tuple = (A, B, C);
}

impl<A, B, C, D: ?Sized> TupleForm for Cons<A, Cons<B, Cons<C, Cons<D, End>>>> {
    type Tuple = (A, B, C, D);
}

impl<A, B, C, D, E: ?Sized> TupleForm for Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, End>>>>> {
    type Tuple = (A, B, C, D, E);
}

impl<A, B, C, D, E, F: ?Sized> TupleForm for Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, End>>>>>> {
    type Tuple = (A, B, C, D, E, F);
}

impl<A, B, C, D, E, F, G: ?Sized> TupleForm
    for Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, End>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G);
}

impl<A, B, C, D, E, F, G, H: ?Sized> TupleForm
    for Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, End>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H);
}

impl<A, B, C, D, E, F, G, H, I: ?Sized> TupleForm
    for Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, End>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I);
}
/* ------------------------- Contains ----------------------- */

/// A trait that assists with compile-time type set inclusion testing.
/// The `Index` parameter is either `End` or `Cons<...>` depending on
/// whether the trait implementation is a base case or the recursive
/// case.
pub trait Contains<T: ?Sized, Index> {}

/// Base case implementation for when the Cons Head is T.
impl<T: ?Sized, Tail: ?Sized> Contains<T, End> for Cons<T, Tail> {}

/// Recursive case for when the Cons Tail contains T.
impl<T, Index, Head, Tail> Contains<T, Cons<Index, ()>> for Cons<Head, Tail> where
    Tail: Contains<T, Index>
{
}

/* ------------------------- Narrow ----------------------- */

/// A trait for pulling a specific type out of a Variants at compile-time
/// and having access to the other types as the Remainder.
pub trait Narrow<Target: ?Sized, Index>: TupleForm {
    type Remainder: TupleForm;
}

/// Base case where the search Target is in the Head of the Variants.
impl<Target, Tail> Narrow<Target, End> for Cons<Target, Tail>
where
    Tail: TupleForm,
    Cons<Target, Tail>: TupleForm,
{
    type Remainder = Tail;
}

/// Recursive case where the search Target is in the Tail of the Variants.
impl<Head, Tail, Target, Index> Narrow<Target, Recurse<Index>> for Cons<Head, Tail>
where
    Tail: Narrow<Target, Index>,
    Tail: TupleForm,
    Cons<Head, Tail>: TupleForm,
    Cons<Head, <Tail as Narrow<Target, Index>>::Remainder>: TupleForm,
{
    type Remainder = Cons<Head, <Tail as Narrow<Target, Index>>::Remainder>;
}

fn _narrow_test() {
    fn can_narrow<Types, Target, Remainder, Index>()
    where
        Types: Narrow<Target, Index, Remainder = Remainder>,
    {
    }

    type T0 = <(u32, String) as TypeSet>::Variants;

    can_narrow::<T0, u32, _, _>();
    can_narrow::<T0, String, Cons<u32, End>, _>();
}

/* ------------------------- SupersetOf ----------------------- */

/// When all types in a Variants are present in a second Variants
pub trait SupersetOf<Other: ?Sized, Index: ?Sized> {
    type Remainder: TupleForm;
}

/// Base case
impl<T: TupleForm> SupersetOf<End, End> for T {
    type Remainder = T;
}

/// Recursive case - more complex because we have to reason about the Index itself as a
/// heterogenous list.
impl<SubHead, SubTail, SuperHead, SuperTail, HeadIndex, TailIndex>
    SupersetOf<Cons<SubHead, SubTail>, Cons<HeadIndex, TailIndex>> for Cons<SuperHead, SuperTail>
where
    Cons<SuperHead, SuperTail>: Narrow<SubHead, HeadIndex>,
    <Cons<SuperHead, SuperTail> as Narrow<SubHead, HeadIndex>>::Remainder:
        SupersetOf<SubTail, TailIndex>,
{
    type Remainder =
        <<Cons<SuperHead, SuperTail> as Narrow<SubHead, HeadIndex>>::Remainder as SupersetOf<
            SubTail,
            TailIndex,
        >>::Remainder;
}

fn _superset_test() {
    fn is_superset<S1, S2, Remainder, Index>()
    where
        S1: SupersetOf<S2, Index, Remainder = Remainder>,
    {
    }

    type T0 = <(u32,) as TypeSet>::Variants;
    type T1A = <(u32, String) as TypeSet>::Variants;
    type T1B = <(String, u32) as TypeSet>::Variants;
    type T2 = <(String, i32, u32) as TypeSet>::Variants;
    type T3 = <(Vec<u8>, Vec<i8>, u32, f32, String, f64, i32) as TypeSet>::Variants;

    is_superset::<T0, T0, _, _>();
    is_superset::<T1A, T1A, _, _>();
    is_superset::<T1A, T1B, _, _>();
    is_superset::<T1B, T1A, _, _>();
    is_superset::<T2, T2, _, _>();
    is_superset::<T1A, T0, _, _>();
    is_superset::<T1B, T0, _, _>();
    is_superset::<T2, T0, <(String, i32) as TypeSet>::Variants, _>();
    is_superset::<T2, T1A, <(i32,) as TypeSet>::Variants, _>();
    is_superset::<T2, T1B, <(i32,) as TypeSet>::Variants, _>();
    is_superset::<T3, T1A, <(Vec<u8>, Vec<i8>, f32, f64, i32) as TypeSet>::Variants, _>();
    is_superset::<T3, T1B, _, _>();
    is_superset::<T3, T0, _, _>();
    is_superset::<T3, T2, _, _>();

    type T5sup = <(u8, u16, u32, u64, u128) as TypeSet>::Variants;
    type T5sub = <(u8, u128) as TypeSet>::Variants;
    type T5rem = <(u16, u32, u64) as TypeSet>::Variants;

    is_superset::<T5sup, T5sub, T5rem, _>();
}
