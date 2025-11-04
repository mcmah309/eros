
fn is_error<T: core::error::Error>(_: &T) {}

fn main() {
    let x: eros::TracedUnion<(u16, std::io::Error)> = eros::TracedUnion::error(1);
    is_error(&&x);
}
