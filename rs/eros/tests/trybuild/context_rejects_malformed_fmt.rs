#[eros::context]
fn malformed(#[fmt(42)] value: u8) -> eros::Result<()> {
    Ok(())
}

fn main() {}
