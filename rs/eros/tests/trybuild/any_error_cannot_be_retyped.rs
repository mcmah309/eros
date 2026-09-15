use eros::type_set::Cons;
use eros::{AnyError, ErrorUnion};
use std::fmt;

fn widen_to_tuple(error: ErrorUnion) -> ErrorUnion<(fmt::Error,)> {
    error.widen()
}

fn widen_to_empty(error: ErrorUnion) -> ErrorUnion<()> {
    error.widen()
}

fn narrow_to_concrete(error: ErrorUnion) {
    let _ = error.narrow::<fmt::Error, _>();
}

fn subset_to_concrete(error: ErrorUnion) {
    let _ = error.subset::<(fmt::Error,), _>();
}

fn into_typed(error: ErrorUnion) -> ErrorUnion<(fmt::Error,)> {
    error.into()
}

fn into_single(error: ErrorUnion) -> fmt::Error {
    error.into_single()
}

fn wildcard_tail_is_not_a_type_set() -> Option<ErrorUnion<Cons<fmt::Error, AnyError>>> {
    None
}

fn main() {}
