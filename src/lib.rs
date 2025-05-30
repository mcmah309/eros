mod context;
mod generic_error;
mod macros;
mod error_union;
mod error_union_to_enum;
mod string_kind;
mod type_set;

pub type Result<T,E> = std::result::Result<T, ErrorUnion<E>>;
pub type GenericResult<T> = std::result::Result<T, GenericCtxError>;

pub use context::Context;
pub use generic_error::GenericCtxError;
pub use generic_error::GenericError;
pub use generic_error::Generalize;
pub use string_kind::StringKind;

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
