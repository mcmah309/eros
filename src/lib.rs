#![doc = include_str!("../README.md")]

mod any_error;
mod context;
mod macros;
mod str_error;
mod traced_union;
mod type_set;
mod union_to_enum;

// aliases
pub type Result<T, E = AnyError> = std::result::Result<T, TracedUnion<E>>;

// data structures
pub use any_error::AnyError;
pub use context::AbsentValueError;
pub use str_error::StrContext;
pub use traced_union::SendSyncError;
pub use traced_union::TracedUnion;
pub use type_set::Cons;
pub use type_set::End;
pub use type_set::Recurse;
pub use type_set::{E1, E2, E3, E4, E5, E6, E7, E8, E9};

// traits
pub use context::Context;
pub use traced_union::ReshapeUnion;
pub use traced_union::IntoUnion;
pub use traced_union::IntoDynUnion;