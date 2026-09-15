use eros::{ErrorUnion, MsgError, ReshapeUnion};

fn main() {
    let result: eros::Result<(), (MsgError, std::io::Error, std::fmt::Error)> = Ok(());
    let result = result.recover(|_: ErrorUnion<(MsgError, std::fmt::Error)>| ());
    let _ = result.recover::<std::fmt::Error, _>(|_| ());
}
