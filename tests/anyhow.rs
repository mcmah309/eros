use anyhow::Context;

#[test]
fn nesting_context_anyhow() {
    fn func1()  -> anyhow::Result<()>{
        // let error = std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here");
        // anyhow::bail!(error);
        return Err(anyhow::anyhow!(std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here"))).context("From func1");
        // return Err(anyhow::anyhow!("Base Error")).context("From func1");
    }

    fn func2() -> anyhow::Result<()> {
        return func1().context("From func2".to_string());
    }

    fn func3() -> anyhow::Result<()> {
        return func2().with_context(|| "From func3");
    }

    let result = func3();
    println!("{:?}", result.unwrap_err());
}