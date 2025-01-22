use eros::{Context, GenericError, U};



#[test]
fn nesting_unit_context() {
    fn func1()  -> eros::Result<()> {
            return Err(eros::generic!("This is an issue"));
            // eros::bail!("This is an issue");
            // return Err(OneOf::new(GenericError::new("This is an issue"))).context("From func1");
        }

    fn func2() -> Result<(), U<(i32,GenericError)>> {
        func1().context("From func2".to_string()).map_err(U::broaden)
    }

    fn func3() -> Result<(), U<(GenericError,i32,bool)>> {
        return func2().with_context(|| "From func3").map_err(U::broaden)
    }

    let result: Result<(), U<(GenericError,i32, bool)>> = func3();
    println!("{}", result.unwrap_err());
}

#[test]
fn nesting_error_context() {
    fn func1()  -> Result<(), U<(std::io::Error,)>>{
        // return Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here").into());
        // return Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here"))?;
        return Err(U::new(std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here"))).context("From func1");
    }

    fn func2() -> Result<(), U<(i32,std::io::Error,)>> {
        func1().context("From func2".to_string()).map_err(U::broaden)
    }

    fn func3() -> Result<(), U<(std::io::Error,i32,bool)>> {
        return func2().with_context(|| "From func3").map_err(U::broaden)
    }

    let result: Result<(), U<(std::io::Error,i32, bool)>> = func3();
    // println!("{}", result.unwrap_err());
    println!("{}", result.unwrap_err());
}