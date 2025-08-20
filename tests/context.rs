#![cfg(feature = "nightly")]

use eros::{
    bail, traced, Context, ErrorUnion, FlateUnionResult, IntoDynTracedError, IntoUnionResult,
    TracedError, TracedResult,
};

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
        func1().context("From func2".to_string()).inflate()
    }

    fn func3() -> eros::UnionResult<(), (std::io::Error, i32, bool)> {
        return func2()
            .with_context(|| "From func3")
            .map_err(ErrorUnion::inflate);
    }

    fn func4() -> eros::UnionResult<(), (std::io::Error, bool)> {
        return match func3().with_context(|| "From func4").deflate::<i32, _>() {
            Ok(_) => panic!("should exist"),
            Err(result) => result,
        };
    }

    let result: Result<(), ErrorUnion<(std::io::Error, i32, bool)>> = func3();
    println!("{:?}", result.unwrap_err());
    let result: Result<(), ErrorUnion<(std::io::Error, bool)>> = func4();
    println!("{:?}", result.unwrap_err());
}

#[test]
fn generic_context_error_to_error_union() {
    fn func1() -> Result<(), TracedError> {
        return Err(traced!("This is root error message"));
    }

    fn func2() -> eros::TracedResult<()> {
        func1().context("Generic context")
    }

    fn func3() -> Result<(), ErrorUnion<(std::io::Error, TracedError)>> {
        func2()
            .map_err(TracedError::inflate)
            .context("Error union context")
    }

    let result: Result<(), ErrorUnion<(std::io::Error, TracedError)>> = func3();
    println!("{:?}", result.unwrap_err());
    // println!("{}", result.unwrap_err());
}

#[test]
fn generic_error_to_error_union() {
    fn func1() -> Result<(), TracedError> {
        bail!("This is root error message")
    }

    fn func2() -> Result<(), ErrorUnion<(std::io::Error, TracedError)>> {
        func1()
            .map_err(TracedError::inflate)
            .context("Error union context")
    }

    let result: Result<(), ErrorUnion<(std::io::Error, TracedError)>> = func2();
    println!("{:?}", result.unwrap_err());
    // println!("{}", result.unwrap_err());
}

#[test]
fn bail() {
    fn func1() -> TracedResult<()> {
        eros::bail!("This is a bailing message {}", 1);
    }

    fn func2() -> eros::UnionResult<(), (TracedError,)> {
        func1().context("From func2".to_string()).union()
    }

    fn func3() -> Result<(), ErrorUnion<(TracedError, i32, bool)>> {
        return func2()
            .with_context(|| "From func3")
            .map_err(ErrorUnion::inflate);
    }

    fn func4() -> TracedResult<()> {
        let error =
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here")
                .traced_dyn();
        return Err(error);
    }

    let result: Result<(), ErrorUnion<(TracedError, i32, bool)>> = func3();
    println!("{:?}", result.unwrap_err());
    let result2: TracedResult<()> = func4();
    println!("{:?}", result2.unwrap_err());
}
