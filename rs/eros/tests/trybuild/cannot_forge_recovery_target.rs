use eros::{
    ErrorUnion, MsgError,
    type_set::{End, RecoveryTarget},
};

struct ForgedIndex;

impl RecoveryTarget<(std::io::Error,), ForgedIndex> for (MsgError, std::fmt::Error) {
    type Selected = Self;
    type Remainder = End;

    fn split(_: ErrorUnion<(std::io::Error,)>) -> Result<ErrorUnion<Self>, ErrorUnion<()>> {
        unimplemented!()
    }
}

fn main() {}
