use eros::{ErrorUnion, MsgError, ReshapeUnion};

fn main() {
    let result: eros::Result<(), (MsgError, std::io::Error, std::fmt::Error)> = Ok(());
    let _: eros::Result<(), (MsgError,)> =
        result.try_recover(|_: ErrorUnion<(MsgError, std::fmt::Error)>| Ok(()));
}
