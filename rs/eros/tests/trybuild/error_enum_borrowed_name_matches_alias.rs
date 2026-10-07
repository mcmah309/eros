#[eros::error_enum(SharedError)]
#[eros::error_enum_ref]
type SharedErrorRef = (std::fmt::Error,);

#[eros::error_enum(MutableError)]
#[eros::error_enum_mut]
type MutableErrorMut = (std::fmt::Error,);

fn main() {}
