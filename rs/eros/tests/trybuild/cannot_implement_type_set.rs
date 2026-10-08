use eros::TypeSet;
use eros::type_set::TupleForm;

struct CustomSet;

impl TypeSet for CustomSet {
    type Variants = <() as TypeSet>::Variants;
}

impl TupleForm for CustomSet {
    type Tuple = ();
}

fn main() {}
