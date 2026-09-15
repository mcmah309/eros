use eros::{
    ErrorUnion, MsgError,
    type_set::{End, NarrowTarget},
};

struct ForgedIndex;

impl NarrowTarget<(std::io::Error,), ForgedIndex> for MsgError {
    type Output = MsgError;
    type Remainder = End;

    fn split(_: ErrorUnion<(std::io::Error,)>) -> Result<MsgError, ErrorUnion<()>> {
        unimplemented!()
    }
}

fn main() {}
