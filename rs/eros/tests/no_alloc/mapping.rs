#[derive(Debug)]
struct Payload([usize; 2]);
impl core::fmt::Display for Payload {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "payload {:?}", self.0)
    }
}
impl core::error::Error for Payload {}

fn main() {
    let error: eros::ErrorUnion<(core::fmt::Error,)> = eros::ErrorUnion::new(core::fmt::Error);
    let _ = error.map_single(|_| Payload([1, 2]));
}
