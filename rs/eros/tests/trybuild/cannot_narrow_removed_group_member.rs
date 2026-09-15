use eros::{ErrorUnion, MsgError};

fn main() {
    let error: ErrorUnion<(std::fmt::Error, MsgError)> = ErrorUnion::new(std::fmt::Error);
    let remainder = error.narrow::<(MsgError,), _>().unwrap_err();
    let _ = remainder.narrow::<MsgError, _>();
}
