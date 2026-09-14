#[eros::context]
fn duplicate(#[fmt("{}")] #[fmt("{:?}")] value: u8) -> eros::Result<()> {
    Ok(())
}

fn main() {}
