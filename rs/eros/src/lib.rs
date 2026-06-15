#![doc = include_str!("../README.md")]

mod any_error;
mod context;
mod macros;
mod str_error;
mod error_union;
mod type_set;
mod union_to_enum;
#[cfg(feature = "logging")]
mod logging;

#[cfg(feature = "logging")]
pub use logging::LogExt;

// re-export macro
pub use eros_macros::context;

// aliases
pub type Result<T, E = AnyError> = std::result::Result<T, ErrorUnion<E>>;

// data structures
pub use any_error::AnyError;
pub use context::AbsentValueError;
pub use str_error::StrError;
pub use error_union::SendSyncError;
pub use error_union::ErrorUnion;
pub use type_set::Cons;
pub use type_set::End;
pub use type_set::Recurse;
pub use type_set::{E1, E2, E3, E4, E5, E6, E7, E8, E9};

// traits
pub use context::Context;
pub use error_union::ReshapeUnion;
pub use error_union::IntoUnion;
pub use error_union::IntoDynUnion;