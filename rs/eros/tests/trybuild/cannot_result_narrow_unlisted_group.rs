use eros::{ErrorUnion, MsgError, ReshapeUnion};

fn main() {
    let result: eros::Result<(), (MsgError, std::fmt::Error)> =
        Err(ErrorUnion::new(MsgError::from("root")));
    let _ = result.narrow::<(MsgError, std::io::Error), _>();
}
