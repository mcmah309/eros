#[derive(Debug)]
struct Payload(u8);
impl core::fmt::Display for Payload {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "payload {}", self.0)
    }
}
impl core::error::Error for Payload {}

fn main() {
    let _: eros::ErrorUnion<(Payload,)> = eros::ErrorUnion::new(Payload(1));
}
