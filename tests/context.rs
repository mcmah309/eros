use eros::{AnyError, Context, ErrorUnion, IntoTraced, TracedError, TracedResult};

#[test]
fn error_union() {
    fn func1() -> eros::UnionResult<(), (std::io::Error,)> {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            "Address in use message here",
        )
        .into());
    }

    fn func2() -> Result<(), ErrorUnion<(i32, std::io::Error)>> {
        func1()
            .context("From func2".to_string())
            .map_err(ErrorUnion::inflate)
    }

    fn func3() -> eros::UnionResult<(), (std::io::Error, i32, bool)> {
        return func2()
            .with_context(|| "From func3")
            .map_err(ErrorUnion::inflate);
    }

    let result: Result<(), ErrorUnion<(std::io::Error, i32, bool)>> = func3();
    println!("{:?}", result.unwrap_err());
    // println!("{}", result.unwrap_err());
}

#[test]
fn generic_context_error_to_error_union() {
    fn func1() -> Result<(), TracedError> {
        return Err(TracedError::msg("This is root error message"));
    }

    fn func2() -> eros::TracedResult<()> {
        func1().context("Generic context")
    }

    fn func3() -> Result<(), ErrorUnion<(std::io::Error, AnyError)>> {
        func2()
            .map_err(TracedError::inflate)
            .context("Error union context")
    }

    let result: Result<(), ErrorUnion<(std::io::Error, AnyError)>> = func3();
    println!("{:?}", result.unwrap_err());
    // println!("{}", result.unwrap_err());
}

#[test]
fn generic_error_to_error_union() {
    fn func1() -> Result<(), AnyError> {
        return Err(AnyError::msg("This is root error message"));
    }

    fn func2() -> Result<(), ErrorUnion<(std::io::Error, AnyError)>> {
        func1()
            .map_err(AnyError::inflate)
            .context("Error union context")
    }

    let result: Result<(), ErrorUnion<(std::io::Error, AnyError)>> = func2();
    println!("{:?}", result.unwrap_err());
    // println!("{}", result.unwrap_err());
}

#[test]
fn bail() {
    fn func1() -> TracedResult<()> {
        eros::bail!("This is a bailing message {}", 1);
    }

    fn func2() -> eros::UnionResult<(), (AnyError,)> {
        func1()
            .context("From func2".to_string())
            .map_err(TracedError::inflate)
    }

    fn func3() -> Result<(), ErrorUnion<(AnyError, i32, bool)>> {
        return func2()
            .with_context(|| "From func3")
            .map_err(ErrorUnion::inflate);
    }

    fn func4() -> TracedResult<()> {
        let error = std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            "Address in use message here",
        ).into_traced();
        return Err(error);
    }

    let result: Result<(), ErrorUnion<(AnyError, i32, bool)>> = func3();
    println!("{:?}", result.unwrap_err());
    let result2: TracedResult<()> = func4();
    println!("{:?}", result2.unwrap_err());
}
