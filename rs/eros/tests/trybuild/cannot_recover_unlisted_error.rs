use eros::{ErrorUnion, MsgError, ReshapeUnion};

fn main() {
    let result: eros::Result<(), (std::fmt::Error,)> = Err(ErrorUnion::new(std::fmt::Error));
    let _ = result.recover::<MsgError, _>(|_| ());
}
