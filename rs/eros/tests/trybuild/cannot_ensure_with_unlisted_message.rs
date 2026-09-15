fn main() {}

fn typed() -> eros::Result<(), (std::fmt::Error,)> {
    eros::ensure!(false, "message");
    Ok(())
}
