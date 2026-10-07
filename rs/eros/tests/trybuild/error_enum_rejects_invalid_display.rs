#[eros::error_enum(InvalidError, "invalid {1}")]
type Invalid = (std::fmt::Error,);

fn main() {}
