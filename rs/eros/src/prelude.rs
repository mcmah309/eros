//! Extension traits for converting results, reshaping error sets, and adding context.
//!
//! ```
//! use eros::prelude::*;
//!
//! let result = Err::<(), _>(std::fmt::Error)
//!     .union::<(std::fmt::Error,), _>()
//!     .context("format response");
//! result.recover::<std::fmt::Error, _>(|_| ()).into_value();
//! ```

pub use crate::{Context, IntoAnyUnion, IntoUnion, ReshapeUnion};
