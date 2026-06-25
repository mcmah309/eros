#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("../README.md")]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// Re-export alloc items so that the exported macros (e.g. `error!`/`bail!`) work in
// both `std` and `no_std` (with `alloc`) consumer crates without requiring the
// consumer to manually depend on `alloc`.
#[doc(hidden)]
pub mod __private {
    pub use alloc::{borrow::Cow, boxed::Box, string::String, sync::Arc, vec::Vec};
    pub use alloc::format;
}

mod any_error;
mod context;
mod macros;
mod str_error;
mod error_union;
mod type_set;
mod union_to_enum;
#[cfg(feature = "user_context")]
mod user_context;
#[cfg(feature = "logging")]
mod logging;

#[cfg(feature = "logging")]
pub use logging::LogExt;

// re-export macro
pub use eros_macros::context;

// aliases
pub type Result<T, E = AnyError> = core::result::Result<T, ErrorUnion<E>>;

// data structures
pub use any_error::AnyError;
pub use context::ContextSource;
#[cfg(feature = "context")]
pub use context::AbsentValueError;
pub use str_error::StrError;
pub use error_union::SendSyncError;
pub use error_union::ErrorUnion;
pub use type_set::TypeSet;
pub use type_set::Cons;
pub use type_set::End;
pub use type_set::Recurse;
pub use type_set::{
    E1, E2, E3, E4, E5, E6, E7, E8, E9, E10,
    E11, E12, E13, E14, E15, E16, E17, E18, E19, E20,
    E21, E22, E23, E24, E25, E26,
};

// traits
pub use context::Context;
pub use error_union::ReshapeUnion;
pub use error_union::IntoUnion;
pub use error_union::IntoDynUnion;