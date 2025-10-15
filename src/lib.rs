#![cfg_attr(feature = "min_specialization", feature(min_specialization))]
#![doc = include_str!("../README.md")]

mod context;
mod error_union;
mod error_union_to_enum;
mod generic_error;
mod macros;
mod str_error;
mod type_set;

// aliases
pub type UResult<T, E> = std::result::Result<T, ErrorUnion<E>>;
pub type Result<T, E = Box<dyn AnyError>> = std::result::Result<T, TracedError<E>>;
pub type TE<E = Box<dyn AnyError>> = TracedError<E>;

// data structures
pub use context::AbsentValueError;
pub use generic_error::AnyError;
pub use generic_error::TracedError;
pub use str_error::StrError;

pub use error_union::ErrorUnion;
pub use type_set::{E1, E2, E3, E4, E5, E6, E7, E8, E9};
pub use type_set::End;
pub use type_set::Cons;
pub use type_set::Recurse;


// traits
pub use context::Context;
pub use error_union::Union;
pub use error_union::IntoUnion;
pub use error_union::ReshapeUnion;
pub use generic_error::Traced;
pub use generic_error::TracedDyn;
pub use generic_error::IntoTraced;
