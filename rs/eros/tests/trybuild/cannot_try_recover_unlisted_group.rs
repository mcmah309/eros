use eros::{ErrorUnion, MsgError, ReshapeUnion};

fn main() {
    let result: eros::Result<(), (MsgError, std::fmt::Error)> = Ok(());
    let _: eros::Result<(), (std::fmt::Error,)> =
        result.try_recover(|_: ErrorUnion<(MsgError, std::io::Error)>| Ok(()));
}
