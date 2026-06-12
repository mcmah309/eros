
fn is_error<T: core::error::Error>(_: &T) {}

fn main() {
    let x: eros::ErrorUnion<(u16, std::io::Error)> = eros::ErrorUnion::error(1);
    is_error(&&x);
}
