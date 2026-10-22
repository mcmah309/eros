#[eros::error_enum_kind(InvalidKind, "operation failed: {0}")]
type Invalid = (std::fmt::Error,);

fn main() {}
