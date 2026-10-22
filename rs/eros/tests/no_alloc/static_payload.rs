#[derive(Debug)]
struct Message(&'static str);
impl core::fmt::Display for Message {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.0)
    }
}
impl core::error::Error for Message {}

fn main() {
    // Static text needs no heap, but the fat reference exceeds one word.
    let _: eros::ErrorUnion<(Message,)> = eros::ErrorUnion::new(Message("message"));
}
