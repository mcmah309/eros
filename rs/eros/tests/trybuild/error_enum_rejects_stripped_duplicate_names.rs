type Failure = std::fmt::Error;
type FailureError = std::io::Error;

#[eros::error_enum(InvalidError)]
type Invalid = (Failure, FailureError);

fn main() {}
