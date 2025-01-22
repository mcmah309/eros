use eros::{Context, GenericError, ErrorUnion};



#[test]
fn nesting_unit_context() {
    fn func1()  -> eros::GenericResult<()> {
            return Err(eros::generic!("This is an issue"));
            // eros::bail!("This is an issue");
            // return Err(ErrorUnion::new(GenericError::new("This is an issue"))).context("From func1");
        }

    fn func2() -> Result<(), ErrorUnion<(i32,GenericError)>> {
        func1().context("From func2".to_string()).map_err(ErrorUnion::broaden)
    }

    fn func3() -> eros::Result<(), (GenericError,i32,bool)> {
        return func2().with_context(|| "From func3").map_err(ErrorUnion::broaden)
    }

    let result: Result<(), ErrorUnion<(GenericError,i32, bool)>> = func3();
    println!("{}", result.unwrap_err());
}

#[test]
fn nesting_error_context() {
    fn func1()  -> Result<(), ErrorUnion<(std::io::Error,)>>{
        // return Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here").into());
        // return Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here"))?;
        return Err(ErrorUnion::new(std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here"))).context("From func1");
    }

    fn func2() -> Result<(), ErrorUnion<(i32,std::io::Error,)>> {
        func1().context("From func2".to_string()).map_err(ErrorUnion::broaden)
    }

    fn func3() -> Result<(), ErrorUnion<(std::io::Error,i32,bool)>> {
        return func2().with_context(|| "From func3").map_err(ErrorUnion::broaden)
    }

    let result: Result<(), ErrorUnion<(std::io::Error,i32, bool)>> = func3();
    // println!("{}", result.unwrap_err());
    println!("{}", result.unwrap_err());
}