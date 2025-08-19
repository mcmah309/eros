#![cfg_attr(feature = "nightly", feature(min_specialization))]

mod context;
mod generic_error;
mod macros;
mod error_union;
mod error_union_to_enum;
mod str_error;
mod type_set;

pub type UnionResult<T,E> = std::result::Result<T, ErrorUnion<E>>;
pub type UResult<T,E> = UnionResult<T, E>;
pub type TracedResult<T> = std::result::Result<T, TracedError>;
pub type Result<T> = TracedResult<T>;

pub use context::Context;
pub use error_union::InflateResult;
pub use error_union::DeflateResult;
pub use error_union::IntoUnion;
pub use generic_error::BoxedError;
pub use generic_error::TracedError;
pub use generic_error::IntoTracedError;
pub use str_error::StrError;

/// Similar to anonymous unions / enums in languages that support type narrowing.
pub use error_union::ErrorUnion;

pub use type_set::{E1, E2, E3, E4, E5, E6, E7, E8, E9};

/* ------------------------- Helpers ----------------------- */

/// The final element of a type-level Cons list.
#[doc(hidden)]
#[derive(Debug)]
pub enum End {}

impl std::error::Error for End {}

/// A compile-time list of types, similar to other basic functional list structures.
#[doc(hidden)]
#[derive(Debug)]
pub struct Cons<Head, Tail>(core::marker::PhantomData<Head>, Tail);

#[doc(hidden)]
#[derive(Debug)]
pub struct Recurse<Tail>(Tail);
