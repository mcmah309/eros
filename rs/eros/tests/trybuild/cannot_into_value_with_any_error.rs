use eros::ReshapeUnion;

fn main() {
    let result: eros::Result<()> = Ok(());
    let _ = result.into_value();
}
