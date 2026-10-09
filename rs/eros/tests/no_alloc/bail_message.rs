fn operation() -> eros::Result<(), (eros::MsgError,)> {
    eros::bail!("value {}", 7);
}
fn main() {
    let _ = operation();
}
