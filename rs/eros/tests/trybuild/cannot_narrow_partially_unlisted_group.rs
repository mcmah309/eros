use eros::{ErrorUnion, MsgError};

fn main() {
    let error: ErrorUnion<(std::fmt::Error, MsgError)> = ErrorUnion::new(std::fmt::Error);
    let _ = error.narrow::<(std::fmt::Error, std::io::Error), _>();
}
