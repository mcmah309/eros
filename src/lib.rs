#![doc = include_str!("../README.md")]

mod context;
mod macros;
mod str_error;
mod traced_union;
mod type_set;
mod union_to_enum;

// aliases
pub type Result<T, E = (AnyError,)> = std::result::Result<T, TracedUnion<E>>;

#[derive(Debug)]
pub struct AnyError(Box<dyn SendSyncError>);

impl AnyError {
    pub fn downcast<T: 'static>(self) -> std::result::Result<Box<T>, AnyError> {
        if (self.0.as_ref() as &dyn Any).is::<T>() {
            Ok((self.0 as Box<dyn Any>).downcast::<T>().unwrap())
        } else {
            Err(self)
        }
    }
}

impl Display for AnyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Needs to be ref so can be used below
// impl Error for &AnyError {} //todo
// todo remove error fold and all error logic, clean up tests
// todo look into yeeting into multiple types

// A
//************************************************************************//

// Now `Result<_,T>` can be converted to `Result<_,AnyError>`
impl<A> From<A> for AnyError
where
    A: SendSyncError,
{
    fn from(value: A) -> Self {
        AnyError(Box::new(value))
    }
}

// Now `Result<_,AnyError>` can be converted to `Result<_,TracedUnion>`
impl From<AnyError> for TracedUnion {
    fn from(value: AnyError) -> Self {
        TracedUnion::new(value)
    }
}

// (A,)
//************************************************************************//

impl<T> From<T> for TracedUnion<(AnyError,)>
where
    T: SendSyncError,
{
    fn from(value: T) -> Self {
        TracedUnion::any_error(value)
    }
}

impl<T> From<T> for TracedUnion<(T,)>
where
    T: SendSyncError,
{
    fn from(value: T) -> Self {
        TracedUnion::error(value)
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
        TracedUnion::error(value)
    }
}

impl<B> From<B> for TracedUnion<(AnyError, B)>
where
    B: SendSyncError,
{
    fn from(value: B) -> Self {
        TracedUnion::error(value)
    }
}

impl<B> From<AnyError> for TracedUnion<(AnyError, B)>
where
    B: SendSyncError,
{
    fn from(value: AnyError) -> Self {
        TracedUnion::new(value)
    }
}

impl<A> From<AnyError> for TracedUnion<(A, AnyError)>
where
    A: SendSyncError,
{
    fn from(value: AnyError) -> Self {
        TracedUnion::new(value)
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
pub use traced_union::IntoUnionSingle;
pub use traced_union::ReshapeUnion;
pub use traced_union::Union;
