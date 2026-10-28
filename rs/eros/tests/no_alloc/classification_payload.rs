#[derive(Debug)]
struct Payload([usize; 2]);
impl core::fmt::Display for Payload {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl core::error::Error for Payload {}

fn main() {
    let error = eros::error!("implementation failure");
    let _: eros::ErrorUnion<(Payload,)> = error.classify(|_| Payload([1, 2]));
}
