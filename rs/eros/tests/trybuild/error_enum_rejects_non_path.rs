#[eros::error_enum(InvalidError, "{0}")]
type Invalid = (&'static std::fmt::Error,);

fn main() {}
