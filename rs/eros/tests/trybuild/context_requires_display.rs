struct NotDisplay;

#[eros::context("value={}", value)]
fn explicit(value: &NotDisplay) -> eros::Result<()> {
    let _ = value;
    Ok(())
}

#[eros::eager_context]
fn automatic(#[fmt("{}")] value: NotDisplay) -> eros::Result<()> {
    let _ = value;
    Ok(())
}

fn main() {}
