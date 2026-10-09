#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "alloc", doc = include_str!("../README.md"))]
#![cfg_attr(not(feature = "alloc"), doc = include_str!("../NO_ALLOC.md"))]

#[cfg(feature = "alloc")]
extern crate alloc;

// Lets procedural macros use the public crate path within Eros as well.
extern crate self as eros;

#[cfg(any(feature = "std", test))]
extern crate std;

// Re-export alloc items so that the exported macros (e.g. `error!`/`bail!`) work in
// both `std` and `no_std` (with `alloc`) consumer crates without requiring the
// consumer to manually depend on `alloc`.
#[doc(hidden)]
pub mod __private {
    use crate::{ErrorUnion, TypeSet};

    #[cfg(feature = "alloc")]
    pub use alloc::format;
    pub use eros_macros::format_error;

    /// Moves the inner error out for generated enum conversions.
    ///
    /// # Safety
    /// `T` must be the exact concrete type of the union's inner error.
    #[inline]
    pub unsafe fn downcast_error_unchecked<T: 'static>(union_of: ErrorUnion<impl TypeSet>) -> T {
        // SAFETY: The caller guarantees that the inner error is T.
        unsafe { union_of.inner.downcast_error_unchecked::<T>() }
    }

    /// Borrows the inner error for generated enum conversions.
    ///
    /// # Safety
    /// `T` must be the exact concrete type of the union's inner error.
    #[inline]
    pub unsafe fn downcast_error_ref_unchecked<T: 'static>(
        union_of: &ErrorUnion<impl TypeSet>,
    ) -> &T {
        // SAFETY: The caller guarantees that the inner error is T.
        unsafe { union_of.inner.downcast_error_ref_unchecked::<T>() }
    }

    /// Mutably borrows the inner error for generated enum conversions.
    ///
    /// # Safety
    /// `T` must be the exact concrete type of the union's inner error.
    #[inline]
    pub unsafe fn downcast_error_mut_unchecked<T: 'static>(
        union_of: &mut ErrorUnion<impl TypeSet>,
    ) -> &mut T {
        // SAFETY: The caller guarantees that the inner error is T.
        unsafe { union_of.inner.downcast_error_mut_unchecked::<T>() }
    }
}

mod any_error;
mod context;
#[cfg(feature = "diagnostic")]
mod diagnostic;
mod error_union;
mod formatting;
mod macros;
#[cfg(feature = "alloc")]
mod msg_error;
mod narrowing;
pub mod prelude;
mod recovery;
#[cfg(feature = "alloc")]
mod root_error;
pub mod type_set;
#[cfg(all(test, feature = "user_context"))]
mod user_context;
#[cfg(not(feature = "alloc"))]
mod zst_error;

// re-export macro
pub use eros_macros::{
    context, eager_context, error_enum, error_enum_kind, error_enum_mut, error_enum_ref,
};

// aliases
pub type Result<T, E = AnyError> = core::result::Result<T, ErrorUnion<E>>;

// data structures
pub use any_error::AnyError;
pub use context::AbsentValueError;
#[cfg(feature = "context")]
pub use context::ContextFrame;
pub use context::ContextValue;
pub use error_union::ErrorUnion;
pub use error_union::SendSyncError;
pub use error_union::StdError;
#[cfg(feature = "alloc")]
pub use msg_error::MsgError;
pub use type_set::TypeSet;

// traits
pub use context::Context;
pub use error_union::IntoAnyUnion;
pub use error_union::IntoUnion;
pub use error_union::ReshapeUnion;
