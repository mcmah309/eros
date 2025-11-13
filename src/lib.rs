#![doc = include_str!("../README.md")]
// #![feature(more_maybe_bounds)]

mod context;
mod macros;
mod str_error;
mod traced_union;
mod type_set;
mod union_to_enum;

// aliases
pub type Result<T, E = (AnyError,)> = std::result::Result<T, TracedUnion<E>>;
pub type AnyError = dyn SendSyncError;

// data structures
pub use context::AbsentValueError;
pub use str_error::StrContext;
pub use traced_union::SendSyncError;

pub use traced_union::TracedUnion;
pub use type_set::Cons;
pub use type_set::End;
pub use type_set::Recurse;

// traits
pub use context::Context;
pub use traced_union::IntoUnion;
pub use traced_union::IntoUnionSingle;
pub use traced_union::ReshapeUnion;
pub use traced_union::Union;

struct X {
    i: i32,
    x: [i32],
}

trait Y {}

struct Z {
    i: dyn Y,
}

struct W {}

impl Y for W {}

fn x<T: Sized>(t: T){}
fn y() {
    x(TracedUnion::<(AnyError,)>::any_error(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            "Address in use message here",
        )));
}
