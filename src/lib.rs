#![cfg_attr(feature = "min_specialization", feature(min_specialization))]
#![doc = include_str!("../README.md")]

mod context;
mod error_union;
mod error_union_to_enum;
mod generic_error;
mod macros;
mod str_error;
mod type_set;

pub type UnionResult<T, E> = std::result::Result<T, ErrorUnion<E>>;
pub type UResult<T, E> = UnionResult<T, E>;
pub type TracedResult<T, E = Box<dyn AnyError>> = std::result::Result<T, TracedError<E>>;
pub type Result<T, E = Box<dyn AnyError>> = TracedResult<T, E>;
pub type TE<E = Box<dyn AnyError>> = TracedError<E>;

pub use generic_error::AnyError;
pub use generic_error::TracedError;
pub use str_error::StrError;

//traits
pub use context::Context;
pub use error_union::IntoUnionResult;
pub use error_union::ReshapeUnionResult;
pub use generic_error::IntoConcreteTracedError;
pub use generic_error::IntoDynTracedError;
pub use generic_error::OptionTracedExt;

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
