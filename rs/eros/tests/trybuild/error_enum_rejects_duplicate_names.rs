#[eros::error_enum(InvalidError, "{0}")]
type Invalid = (std::fmt::Error, std::fmt::Error);

fn main() {}
