use eros::TypeSet;
use eros::type_set::TupleForm;

struct CustomSet;

impl TypeSet for CustomSet {
    type Variants = <() as TypeSet>::Variants;
    type Enum = ();
    type RefEnum<'a> = ();
    type MutEnum<'a> = ();
}

impl TupleForm for CustomSet {
    type Tuple = ();
}

fn main() {}
