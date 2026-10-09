fn operation() -> eros::Result<(), (eros::MsgError,)> {
    eros::ensure!(true, "value {}", 7);
    Ok(())
}
fn main() {
    let _ = operation();
}
