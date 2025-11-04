#![doc = include_str!("../README.md")]

mod context;
mod traced_union;
mod union_to_enum;
mod macros;
mod str_error;
mod type_set;

// aliases
pub type Result<T, E = (AnyError,)> = std::result::Result<T, TracedUnion<E>>;
pub type AnyError = Box<dyn SendSyncError>;

// data structures
pub use context::AbsentValueError;
pub use traced_union::SendSyncError;
pub use str_error::StrContext;

pub use traced_union::TracedUnion;
pub use type_set::{E1, E2, E3, E4, E5, E6, E7, E8, E9};
pub use type_set::End;
pub use type_set::Cons;
pub use type_set::Recurse;


// traits
pub use context::Context;
pub use traced_union::Union;
pub use traced_union::IntoUnion;
pub use traced_union::IntoUnionSingle;
pub use traced_union::ReshapeUnion;
