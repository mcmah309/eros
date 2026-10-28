#[derive(Debug)]
struct Code(usize);
impl core::fmt::Display for Code {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "{}", self.0) }
}
impl core::error::Error for Code {}
fn main() { let _: eros::ErrorUnion<(Code,)> = eros::ErrorUnion::new(Code(7)); }
