use eros::{ErrorUnion, MsgError, ReshapeUnion};

fn main() {
    let result: eros::Result<(), (std::fmt::Error, MsgError)> =
        Err(ErrorUnion::new(std::fmt::Error));
    let result = result.recover::<MsgError, _>(|_| ());
    let _ = result.recover::<MsgError, _>(|_| ());
}
