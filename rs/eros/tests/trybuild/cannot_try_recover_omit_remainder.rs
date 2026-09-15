use eros::{ErrorUnion, MsgError, ReshapeUnion};

fn main() {
    let result: eros::Result<(), (std::fmt::Error, MsgError)> =
        Err(ErrorUnion::new(std::fmt::Error));
    let _: eros::Result<(), (MsgError,)> = result.try_recover::<MsgError, _, _, _>(|_| Ok(()));
}
