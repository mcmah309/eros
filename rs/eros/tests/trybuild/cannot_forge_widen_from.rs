use eros::type_set::WidenFrom;
use eros::{AnyError, TypeSet};

struct Index;

// A downstream index cannot turn an open union into a closed one.
impl WidenFrom<AnyError, Index> for <(std::fmt::Error,) as TypeSet>::Variants {}

fn main() {}
