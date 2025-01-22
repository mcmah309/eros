use eros::{Context, ErrorUnion, GenericError, GenericResult};



#[test]
fn error_union() {
    fn func1()  -> eros::Result<(), (std::io::Error,)> {
            return Err(std::io::Error::new(std::io::ErrorKind::AddrInUse,"Address in use message here").into());
        }

    fn func2() -> Result<(), ErrorUnion<(i32,std::io::Error)>> {
        func1().context("From func2".to_string()).map_err(ErrorUnion::inflate)
    }

    fn func3() -> eros::Result<(), (std::io::Error,i32,bool)> {
        return func2().with_context(|| "From func3").map_err(ErrorUnion::inflate)
    }

    let result: Result<(), ErrorUnion<(std::io::Error,i32, bool)>> = func3();
    println!("{}", result.unwrap_err());
}

// #[test]
// fn nesting_error_context() {
//     fn func1()  -> Result<(), ErrorUnion<(std::io::Error,)>>{
//         return Err(ErrorUnion::new(std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here"))).context("From func1");
//     }

//     fn func2() -> Result<(), ErrorUnion<(i32,std::io::Error,)>> {
//         func1().context("From func2".to_string()).map_err(ErrorUnion::broaden)
//     }

//     fn func3() -> Result<(), ErrorUnion<(std::io::Error,i32,bool)>> {
//         return func2().with_context(|| "From func3").map_err(ErrorUnion::broaden)
//     }

//     let result: Result<(), ErrorUnion<(std::io::Error,i32, bool)>> = func3();
//     // println!("{}", result.unwrap_err());
//     println!("{}", result.unwrap_err());
// }

// #[test]
// fn nesting_generic_error() {
//     fn func1()  -> GenericResult<()>{
//         eros::bail!("This is a bailing message");
//     }

//     fn func2() -> eros::Result<(), (GenericError,)> {
//         func1().context("From func2".to_string()).map_err(ErrorUnion::broaden)
//     }

//     fn func3() -> Result<(), ErrorUnion<(GenericError,i32,bool)>> {
//         return func2().with_context(|| "From func3").map_err(ErrorUnion::broaden)
//     }

//     let result: Result<(), ErrorUnion<(GenericError,i32, bool)>> = func3();
//     // println!("{}", result.unwrap_err());
//     println!("{}", result.unwrap_err());
// }