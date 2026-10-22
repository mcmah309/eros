#[eros::error_enum(InvalidError, "{0}")]
type Invalid<T> = (T,);

fn main() {}
