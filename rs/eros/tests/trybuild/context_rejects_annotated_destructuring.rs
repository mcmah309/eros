#[eros::context]
fn destructured(#[fmt("{:?}")] (left, right): (u8, u8)) -> eros::Result<()> {
    Ok(())
}

fn main() {}
