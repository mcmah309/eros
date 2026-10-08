use core::any::Any;

use crate::{AnyError, SendSyncError};

#[doc(hidden)]
pub use crate::narrowing::{GroupNarrow, NarrowTarget, SingleNarrow};
#[doc(hidden)]
pub use crate::recovery::{GroupRecovery, RecoveryHandler, RecoveryTarget, SingleRecovery};

mod sealed {
    pub trait Sealed {}

    // A required method taking this unnameable type seals each relation,
    // including its generic parameters. Sealing only Self would let downstream
    // crates supply a local Index and invent membership or remainder proofs.
    // These methods are never called.
    pub struct Token;
}

// Type-list and index helpers that appear in associated types and inferred proofs.
#[doc(hidden)]
pub enum End {}
#[doc(hidden)]
pub struct Cons<Head, Tail>(core::marker::PhantomData<Head>, Tail);
#[doc(hidden)]
pub struct Recurse<Tail>(Tail);

impl sealed::Sealed for End {}
impl<Head, Tail> sealed::Sealed for Cons<Head, Tail> {}

/// A set of possible errors for an [`ErrorUnion`](crate::ErrorUnion).
///
/// Implemented for tuples of up to 26 [`SendSyncError`] types, the empty tuple
/// `()`, and [`AnyError`].
/// This trait is sealed and cannot be implemented outside Eros.
pub trait TypeSet: sealed::Sealed + Send + Sync + 'static {
    /// The type list used by [`Contains`], [`Narrow`], and [`SupersetOf`].
    type Variants: TupleForm + IsFold;
}

#[rustfmt::skip]
impl sealed::Sealed for AnyError {}

#[rustfmt::skip]
impl TypeSet for AnyError {
    type Variants = AnyError;
}

#[rustfmt::skip]
impl sealed::Sealed for () {}

#[rustfmt::skip]
impl TypeSet for () {
    type Variants = End;
}

#[rustfmt::skip]
impl<A: SendSyncError> sealed::Sealed for (A,) {}

#[rustfmt::skip]
impl<A: SendSyncError> TypeSet for (A,) {
    type Variants = Cons<A, End>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError> sealed::Sealed for (A, B) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError> TypeSet for (A, B) {
    type Variants = Cons<A, Cons<B, End>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError> sealed::Sealed for (A, B, C) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError> TypeSet for (A, B, C) {
    type Variants = Cons<A, Cons<B, Cons<C, End>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError> sealed::Sealed for (A, B, C, D) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError> TypeSet for (A, B, C, D) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, End>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError> sealed::Sealed for (A, B, C, D, E) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError> TypeSet for (A, B, C, D, E) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, End>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError> sealed::Sealed for (A, B, C, D, E, F) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError> TypeSet for (A, B, C, D, E, F) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, End>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError> TypeSet for (A, B, C, D, E, F, G) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, End>>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G, H) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError> TypeSet for (A, B, C, D, E, F, G, H) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, End>>>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G, H, I) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError> TypeSet for (A, B, C, D, E, F, G, H, I) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, End>>>>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G, H, I, J) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError> TypeSet for (A, B, C, D, E, F, G, H, I, J) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, End>>>>>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G, H, I, J, K) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError> TypeSet for (A, B, C, D, E, F, G, H, I, J, K) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, End>>>>>>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G, H, I, J, K, L) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, End>>>>>>>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G, H, I, J, K, L, M) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, End>>>>>>>>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G, H, I, J, K, L, M, N) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, End>>>>>>>>>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, End>>>>>>>>>>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, End>>>>>>>>>>>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, End>>>>>>>>>>>>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, End>>>>>>>>>>>>>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, End>>>>>>>>>>>>>>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, End>>>>>>>>>>>>>>>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError, U: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError, U: SendSyncError> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, End>>>>>>>>>>>>>>>>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError, U: SendSyncError, V: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError, U: SendSyncError, V: SendSyncError> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, End>>>>>>>>>>>>>>>>>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError, U: SendSyncError, V: SendSyncError, W: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError, U: SendSyncError, V: SendSyncError, W: SendSyncError> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, Cons<W, End>>>>>>>>>>>>>>>>>>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError, U: SendSyncError, V: SendSyncError, W: SendSyncError, X: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError, U: SendSyncError, V: SendSyncError, W: SendSyncError, X: SendSyncError> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, Cons<W, Cons<X, End>>>>>>>>>>>>>>>>>>>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError, U: SendSyncError, V: SendSyncError, W: SendSyncError, X: SendSyncError, Y: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError, U: SendSyncError, V: SendSyncError, W: SendSyncError, X: SendSyncError, Y: SendSyncError> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, Cons<W, Cons<X, Cons<Y, End>>>>>>>>>>>>>>>>>>>>>>>>>;
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError, U: SendSyncError, V: SendSyncError, W: SendSyncError, X: SendSyncError, Y: SendSyncError, Z: SendSyncError> sealed::Sealed for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z) {}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError, U: SendSyncError, V: SendSyncError, W: SendSyncError, X: SendSyncError, Y: SendSyncError, Z: SendSyncError> TypeSet for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z) {
    type Variants = Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, Cons<W, Cons<X, Cons<Y, Cons<Z, End>>>>>>>>>>>>>>>>>>>>>>>>>>;
}

//************************************************************************//

/// Associates a type list with its corresponding error set type.
///
/// For example, `Cons<A, Cons<B, End>>` has `Tuple = (A, B)`.
/// This association determines the error set of the remainder union after
/// [`ErrorUnion::narrow`](crate::ErrorUnion::narrow). This trait is sealed.
pub trait TupleForm: sealed::Sealed {
    /// The corresponding error set.
    type Tuple: TypeSet;
}

impl TupleForm for AnyError {
    type Tuple = AnyError;
}

impl TupleForm for End {
    type Tuple = ();
}

#[rustfmt::skip]
impl<A: SendSyncError> TupleForm
    for Cons<A, End>
{
    type Tuple = (A,);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError> TupleForm
    for Cons<A, Cons<B, End>>
{
    type Tuple = (A, B);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError> TupleForm
    for Cons<A, Cons<B, Cons<C, End>>>
{
    type Tuple = (A, B, C);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError> TupleForm
    for Cons<A, Cons<B, Cons<C, Cons<D, End>>>>
{
    type Tuple = (A, B, C, D);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError> TupleForm
    for Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, End>>>>>
{
    type Tuple = (A, B, C, D, E);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError> TupleForm
    for Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, End>>>>>>
{
    type Tuple = (A, B, C, D, E, F);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError> TupleForm
    for Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, End>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError> TupleForm
    for Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, End>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError> TupleForm
    for Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, End>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, End>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, End>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, End>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, End>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, End>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, End>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, End>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, End>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, End>>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, End>>>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, End>>>>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError, U: SendSyncError> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, End>>>>>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError, U: SendSyncError, V: SendSyncError> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, End>>>>>>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError, U: SendSyncError, V: SendSyncError, W: SendSyncError> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, Cons<W, End>>>>>>>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError, U: SendSyncError, V: SendSyncError, W: SendSyncError, X: SendSyncError> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, Cons<W, Cons<X, End>>>>>>>>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError, U: SendSyncError, V: SendSyncError, W: SendSyncError, X: SendSyncError, Y: SendSyncError> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, Cons<W, Cons<X, Cons<Y, End>>>>>>>>>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
}

#[rustfmt::skip]
impl<A: SendSyncError, B: SendSyncError, C: SendSyncError, D: SendSyncError, E: SendSyncError, F: SendSyncError, G: SendSyncError, H: SendSyncError, I: SendSyncError, J: SendSyncError, K: SendSyncError, L: SendSyncError, M: SendSyncError, N: SendSyncError, O: SendSyncError, P: SendSyncError, Q: SendSyncError, R: SendSyncError, S: SendSyncError, T: SendSyncError, U: SendSyncError, V: SendSyncError, W: SendSyncError, X: SendSyncError, Y: SendSyncError, Z: SendSyncError> TupleForm
    for
    Cons<A, Cons<B, Cons<C, Cons<D, Cons<E, Cons<F, Cons<G, Cons<H, Cons<I, Cons<J, Cons<K, Cons<L, Cons<M, Cons<N, Cons<O, Cons<P, Cons<Q, Cons<R, Cons<S, Cons<T, Cons<U, Cons<V, Cons<W, Cons<X, Cons<Y, Cons<Z, End>>>>>>>>>>>>>>>>>>>>>>>>>>
{
    type Tuple = (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
}

//************************************************************************//

/// Tests whether a value's concrete type occurs in a type list.
/// This trait is sealed.
pub trait IsFold: sealed::Sealed {
    /// Returns whether the list contains the value's concrete type.
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

impl IsFold for AnyError {
    fn is_fold(_: &dyn Any) -> bool {
        true
    }
}

//************************************************************************//

/// A type list that contains `T`.
///
/// Applied to [`TypeSet::Variants`] when constructing an error union.
/// `Index` is inferred at call sites. This trait is sealed.
pub trait Contains<T, Index> {
    #[doc(hidden)]
    fn __seal(_: sealed::Token);
}

/// Base case implementation for when the Cons Head is T.
impl<T, Tail> Contains<T, End> for Cons<T, Tail> {
    fn __seal(_: sealed::Token) {}
}

/// Recursive case for when the Cons Tail contains T.
impl<T, Index, Head, Tail> Contains<T, Cons<Index, ()>> for Cons<Head, Tail>
where
    Tail: Contains<T, Index>,
{
    fn __seal(_: sealed::Token) {}
}

impl<T> Contains<T, End> for AnyError {
    fn __seal(_: sealed::Token) {}
}

//************************************************************************//

/// A type list from which `T` can be removed.
///
/// Applied to [`TypeSet::Variants`] by [`ErrorUnion::narrow`](crate::ErrorUnion::narrow).
/// `Index` is inferred at call sites. This trait is sealed.
pub trait Narrow<T, Index>: TupleForm {
    /// The type list remaining after removing `T`.
    type Remainder: TupleForm;

    #[doc(hidden)]
    fn __seal(_: sealed::Token);
}

/// Base case where the search Target is in the Head of the Variants.
impl<Target, Tail> Narrow<Target, End> for Cons<Target, Tail>
where
    Tail: TupleForm,
    Cons<Target, Tail>: TupleForm,
{
    type Remainder = Tail;

    fn __seal(_: sealed::Token) {}
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

    fn __seal(_: sealed::Token) {}
}

fn _narrow_test() {
    use core::{fmt::Error, num::ParseIntError};
    fn can_narrow<Types, Target, Remainder, Index>()
    where
        Types: Narrow<Target, Index, Remainder = Remainder>,
    {
    }

    type T0 = <(Error, ParseIntError) as TypeSet>::Variants;

    can_narrow::<T0, Error, _, _>();
    can_narrow::<T0, ParseIntError, Cons<Error, End>, _>();
}

//************************************************************************//

/// A type list containing every member of `Other`.
///
/// Applied to [`TypeSet::Variants`] by [`ErrorUnion::widen`](crate::ErrorUnion::widen)
/// and [`ErrorUnion::narrow`](crate::ErrorUnion::narrow).
/// `Index` is inferred at call sites. This trait is sealed.
pub trait SupersetOf<Other, Index> {
    /// The type list remaining after removing the members of `Other`.
    type Remainder: TupleForm;

    #[doc(hidden)]
    fn __seal(_: sealed::Token);
}

/// Base case
impl<T: TupleForm> SupersetOf<End, End> for T {
    type Remainder = T;

    fn __seal(_: sealed::Token) {}
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

    fn __seal(_: sealed::Token) {}
}

impl SupersetOf<AnyError, End> for AnyError {
    type Remainder = AnyError;

    fn __seal(_: sealed::Token) {}
}

fn _superset_test() {
    use core::{
        cell::{BorrowError, BorrowMutError},
        fmt::Error,
        num::{ParseFloatError, ParseIntError, TryFromIntError},
        str::Utf8Error,
    };
    fn is_superset<S1, S2, Remainder, Index>()
    where
        S1: SupersetOf<S2, Index, Remainder = Remainder>,
    {
    }

    type T0 = <(Error,) as TypeSet>::Variants;
    type T1A = <(Error, ParseIntError) as TypeSet>::Variants;
    type T1B = <(ParseIntError, Error) as TypeSet>::Variants;
    type T2 = <(ParseIntError, ParseFloatError, Error) as TypeSet>::Variants;
    type T3 = <(
        BorrowError,
        BorrowMutError,
        Error,
        TryFromIntError,
        ParseIntError,
        Utf8Error,
        ParseFloatError,
    ) as TypeSet>::Variants;

    is_superset::<T0, T0, _, _>();
    is_superset::<T1A, T1A, _, _>();
    is_superset::<T1A, T1B, _, _>();
    is_superset::<T1B, T1A, _, _>();
    is_superset::<T2, T2, _, _>();
    is_superset::<T1A, T0, _, _>();
    is_superset::<T1B, T0, _, _>();
    is_superset::<T2, T0, <(ParseIntError, ParseFloatError) as TypeSet>::Variants, _>();
    is_superset::<T2, T1A, <(ParseFloatError,) as TypeSet>::Variants, _>();
    is_superset::<T2, T1B, <(ParseFloatError,) as TypeSet>::Variants, _>();
    is_superset::<
        T3,
        T1A,
        <(
            BorrowError,
            BorrowMutError,
            TryFromIntError,
            Utf8Error,
            ParseFloatError,
        ) as TypeSet>::Variants,
        _,
    >();
    is_superset::<T3, T1B, _, _>();
    is_superset::<T3, T0, _, _>();
    is_superset::<T3, T2, _, _>();

    type T5sup =
        <(BorrowError, BorrowMutError, Error, Utf8Error, ParseIntError) as TypeSet>::Variants;
    type T5sub = <(BorrowError, ParseIntError) as TypeSet>::Variants;
    type T5rem = <(BorrowMutError, Error, Utf8Error) as TypeSet>::Variants;

    is_superset::<T5sup, T5sub, T5rem, _>();
}

impl<Target: SendSyncError> Narrow<Target, End> for AnyError {
    type Remainder = AnyError;
    fn __seal(_: sealed::Token) {}
}

impl<Head, Tail, TailIndex> SupersetOf<Cons<Head, Tail>, Cons<End, TailIndex>> for AnyError
where
    Head: SendSyncError,
    AnyError: SupersetOf<Tail, TailIndex>,
{
    type Remainder = AnyError;
    fn __seal(_: sealed::Token) {}
}
