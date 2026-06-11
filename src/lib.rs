#![doc = include_str!("../README.md")]

mod context;
mod macros;
mod str_error;
mod traced_union;
mod type_set;
mod union_to_enum;

// aliases
pub type Result<T, E = AnyError> = std::result::Result<T, TracedUnion<E>>;

/// A marker type for `TracedUnion` representing all possible errors
#[derive(Debug)]
pub struct AnyError;

//************************************************************************//

impl<A> From<A> for TracedUnion<AnyError>
where
    A: SendSyncError,
{
    fn from(value: A) -> Self {
        TracedUnion::new(value)
    }
}

// (A,)
//************************************************************************//

impl<T, A> From<T> for TracedUnion<(A,)>
where
    T: SendSyncError + Into<A>,
    A: SendSyncError,
{
    fn from(value: T) -> Self {
        TracedUnion::new(value.into())
    }
}

// (A,B)
//************************************************************************//

impl<A, B> From<A> for TracedUnion<(A, B)>
where
    A: SendSyncError,
    B: SendSyncError,
{
    fn from(value: A) -> Self {
        TracedUnion::new(value)
    }
}

// impl<A, B> From<A> for TracedUnion<(A, B)>
// where
//     A: SendSyncError,
//     B: SendSyncError,
// {
//     fn from(value: A) -> Self {
//         TracedUnion::new(value)
//     }
// }

impl<A, B> From<TracedUnion<(A,)>> for TracedUnion<(A, B)>
where
    A: SendSyncError,
    B: SendSyncError,
{
    fn from(value: TracedUnion<(A,)>) -> Self {
        value.widen()
    }
}

// impl<A, B> From<TracedUnion<(A,)>> for TracedUnion<(A, B)>
// where
//     A: SendSyncError,
//     B: SendSyncError,
// {
//     fn from(value: TracedUnion<(A,)>) -> Self {
//         value.widen()
//     }
// }

// wa wa
// impl<T, E, Index> From<T> for TracedUnion<E>
// where
//     T: SendSyncError,
//     E: TypeSet,
//     E::Variants: type_set::Contains<T, Index>
// {
//     fn from(value: A) -> Self {
//         TracedUnion::new(value)
//     }
// }

// ding dong
//************************************************************************//

impl<A> From<TracedUnion<(A,)>> for TracedUnion<AnyError>
where
    A: SendSyncError,
{
    fn from(value: TracedUnion<(A,)>) -> Self {
        TracedUnion::erase(value)
    }
}

impl<A, B> From<TracedUnion<(A, B)>> for TracedUnion<AnyError>
where
    A: SendSyncError,
    B: SendSyncError,
{
    fn from(value: TracedUnion<(A, B)>) -> Self {
        TracedUnion::erase(value)
    }
}

impl<A, B, C> From<TracedUnion<(A, B, C)>> for TracedUnion<AnyError>
where
    A: SendSyncError,
    B: SendSyncError,
    C: SendSyncError,
{
    fn from(value: TracedUnion<(A, B, C)>) -> Self {
        TracedUnion::erase(value)
    }
}

// todo (and do I even need widen anymore?? yes for explicit)
// todo (Remove SendSyncError from anywhere inside it)
// todo (Consider renaming TracedUnion into Eros since a lot of the functionality only really works for errors now)
// todo hopefully will never have to use `union` or `into_union` again?
// todo if we only ever allow creating AnyError in our methods, maybe we can do the type check way to avoid double boxing? Rust may be able to compile way typeid checks where it knows it is not possible
// (A,B,C)
//************************************************************************//

//************************************************************************//

// impl<T> From<T> for TracedUnion
// where
//     T: SendSyncError,
// {
//     fn from(value: T) -> Self {
//         TracedUnion::any_error(value)
//     }
// }

// impl std::error::Error for TracedUnion<(AnyError,)> {}
// impl<A> std::error::Error for TracedUnion<(AnyError, A)> where A: std::error::Error + 'static {}
// impl<A> std::error::Error for TracedUnion<(A, AnyError)> where A: std::error::Error + 'static {}
// impl<A, B> std::error::Error for TracedUnion<(AnyError, A, B)>
// where
//     A: std::error::Error + 'static,
//     B: std::error::Error + 'static,
// {
// }
// impl<A, B> std::error::Error for TracedUnion<(A, AnyError, B)>
// where
//     A: std::error::Error + 'static,
//     B: std::error::Error + 'static,
// {
// }
// impl<A, B> std::error::Error for TracedUnion<(A, B, AnyError)>
// where
//     A: std::error::Error + 'static,
//     B: std::error::Error + 'static,
// {
// }

use std::any::Any;
use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;
use std::ops::Deref;
use std::ops::DerefMut;

// data structures
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
pub use traced_union::IntoUnion;
// pub use traced_union::InnerInto;
pub use traced_union::ReshapeUnion;

use crate::type_set::TypeSet;
pub use traced_union::Union;
