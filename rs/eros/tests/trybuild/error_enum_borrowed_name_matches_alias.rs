#[eros::error_enum(SharedError)]
type SharedErrorRef = (std::fmt::Error,);

#[eros::error_enum(MutableError)]
type MutableErrorMut = (std::fmt::Error,);

fn main() {}
