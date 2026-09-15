use eros::{ErrorUnion, MsgError};

fn main() {
    let error: ErrorUnion<(std::fmt::Error, MsgError)> = ErrorUnion::new(std::fmt::Error);
    let _ = error.subset::<(std::fmt::Error, std::io::Error), _>();
}
