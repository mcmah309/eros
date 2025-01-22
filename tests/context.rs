use eros::{Context, GenericError, OneOf};



#[test]
fn nesting_unit_context() {
    fn func1()  -> eros::Result<()> {
            return Err(eros::generic!("This is an issue"));
            eros::bail!("This is an issue");
            return Err(OneOf::new(GenericError::new("This is an issue"))).context("From func1");
        }

    fn func2() -> Result<(), OneOf<(i32,GenericError)>> {
        func1().context("From func2".to_string()).map_err(OneOf::broaden)
    }

    fn func3() -> Result<(), OneOf<(GenericError,i32,bool)>> {
        return func2().with_context(|| "From func3").map_err(OneOf::broaden)
    }

    let result: Result<(), OneOf<(GenericError,i32, bool)>> = func3();
    println!("{}", result.unwrap_err());
}

#[test]
fn nesting_error_context() {
    fn func1()  -> Result<(), OneOf<(std::io::Error,)>>{
        return Err(OneOf::new(std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here"))).context("From func1");
    }

    fn func2() -> Result<(), OneOf<(i32,std::io::Error,)>> {
        func1().context("From func2".to_string()).map_err(OneOf::broaden)
    }

    fn func3() -> Result<(), OneOf<(std::io::Error,i32,bool)>> {
        return func2().with_context(|| "From func3").map_err(OneOf::broaden)
    }

    let result: Result<(), OneOf<(std::io::Error,i32, bool)>> = func3();
    // println!("{}", result.unwrap_err());
    println!("{}", result.unwrap_err());
}