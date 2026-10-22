use eros::IntoAnyUnion;

#[derive(Debug)]
struct Payload([usize; 2]);
impl core::fmt::Display for Payload {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl core::error::Error for Payload {}

fn main() {
    let _: eros::Result<()> = Err(Payload([1, 2])).any_union();
}
