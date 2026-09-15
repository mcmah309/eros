use eros::{ErrorUnion, MsgError};

fn main() {
    let error: ErrorUnion<(std::fmt::Error,)> = ErrorUnion::new(std::fmt::Error);
    let _ = error.narrow::<MsgError, _>();
}
