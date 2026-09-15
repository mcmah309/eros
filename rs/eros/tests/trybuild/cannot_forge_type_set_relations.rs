use eros::type_set::{Contains, Narrow, SupersetOf};
use eros::{AnyError, MsgError, TypeSet};

// Local indices must not let downstream crates invent membership proofs for
// type lists supplied by otherwise valid, sealed TypeSets.
struct Index;

type Variants = <(std::fmt::Error,) as TypeSet>::Variants;
type Empty = <() as TypeSet>::Variants;

impl Contains<MsgError, Index> for Variants {}

impl Narrow<MsgError, Index> for Variants {
    type Remainder = Empty;
}

impl SupersetOf<<(MsgError,) as TypeSet>::Variants, Index> for Variants {
    type Remainder = Empty;
}

// An erased union must not acquire a concrete remainder or widen into a tuple.
impl Narrow<MsgError, Index> for AnyError {
    type Remainder = Variants;
}

impl SupersetOf<AnyError, Index> for Variants {
    type Remainder = Empty;
}

fn main() {}
