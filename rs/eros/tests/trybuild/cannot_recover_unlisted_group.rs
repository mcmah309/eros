use eros::{ErrorUnion, MsgError, ReshapeUnion};

fn main() {
    let result: eros::Result<(), (MsgError, std::fmt::Error)> = Ok(());
    let _ = result.recover(|_: ErrorUnion<(MsgError, std::io::Error)>| ());
}
