#![cfg(feature = "traced")]

use eros::{
    bail, traced, Context, ErrorUnion, IntoTracedDyn, IntoUnionResult, TracedError,
    TracedResult,
};

#[cfg(feature = "min_specialization")]
#[cfg(test)]
mod min_specialization {
    use eros::{traced, Context, ErrorUnion, ReshapeUnionResult, TracedError};

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
            func1().widen().context("From func2".to_string())
        }

        fn func3() -> eros::UnionResult<(), (std::io::Error, i32, bool)> {
            return func2()
                .with_context(|| "From func3")
                .map_err(ErrorUnion::widen);
        }

        fn func4() -> eros::UnionResult<(), (std::io::Error, bool)> {
            return match func3().with_context(|| "From func4").narrow::<i32, _>() {
                Ok(_) => panic!("should exist"),
                Err(result) => result,
            };
        }

        fn func5() -> eros::UnionResult<(), (std::io::Error, bool, TracedError)> {
            return Err(ErrorUnion::new(traced!("Error"))).context("From func5");
        }

        let result: Result<(), ErrorUnion<(std::io::Error, i32, bool)>> = func3();
        println!("{:?}", result.as_ref().unwrap_err());
        assert!(result.is_err());
        let message = result.unwrap_err().to_string();
        assert!(
            !message.contains("Context:"),
            "Expected no context in message:\n{}",
            message
        );
        let result: Result<(), ErrorUnion<(std::io::Error, bool)>> = func4();
        println!("{:?}", result.as_ref().unwrap_err());
        assert!(result.is_err());
        let message = result.unwrap_err().to_string();
        assert!(
            !message.contains("Context:"),
            "Expected no context in message:\n{}",
            message
        );
        let result: Result<(), ErrorUnion<(std::io::Error, bool, TracedError)>> = func5();
        println!("{:?}", result.as_ref().unwrap_err());
        assert!(result.is_err());
        let message = result.unwrap_err().to_string();
        assert!(
            message.contains("Context:"),
            "Expected context in message:\n{}",
            message
        );
    }
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
        func2().map_err(ErrorUnion::new)
    }

    let result: Result<(), ErrorUnion<(std::io::Error, TracedError)>> = func3();
    println!("{:?}", result.as_ref().unwrap_err());
    assert!(result.is_err());
    let message = result.unwrap_err().to_string();
    assert!(
        message.contains("Context:"),
        "Expected context in message:\n{}",
        message
    );
}

#[test]
fn generic_error_to_error_union() {
    fn func1() -> Result<(), TracedError> {
        bail!("This is root error message")
    }

    fn func2() -> Result<(), ErrorUnion<(std::io::Error, TracedError)>> {
        func1().map_err(ErrorUnion::new)
    }

    let result: Result<(), ErrorUnion<(std::io::Error, TracedError)>> = func2();
    println!("{:?}", result.as_ref().unwrap_err());
    assert!(result.is_err());
    let message = result.unwrap_err().to_string();
    assert!(
        !message.contains("Context:"),
        "Expected no context in message:\n{}",
        message
    );
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
        return func2().map_err(ErrorUnion::widen);
    }

    fn func4() -> TracedResult<()> {
        let error =
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here")
                .traced_dyn();
        return Err(error);
    }

    let result: Result<(), ErrorUnion<(TracedError, i32, bool)>> = func3();
    println!("{:?}", result.as_ref().unwrap_err());
    assert!(result.is_err());
    let message = result.unwrap_err().to_string();
    assert!(
        message.contains("Context:"),
        "Expected context in message:\n{}",
        message
    );
    let result: TracedResult<()> = func4();
    println!("{:?}", result.as_ref().unwrap_err());
    assert!(result.is_err());
    let message = result.unwrap_err().to_string();
    assert!(
        !message.contains("Context:"),
        "Expected no context in message:\n{}",
        message
    );
}

#[test]
fn context_directly_on_error() {
    fn on_error() -> TracedResult<()> {
        let error =
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here")
                .context("This is some context");
        return Err(error);
    }

    fn on_result() -> TracedResult<()> {
        let error =
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here");
        let result: Result<(), std::io::Error> = Err(error);
        let value = result.context("This is some context")?;
        return Ok(value);
    }

    fn on_result_again() -> TracedResult<()> {
        let error =
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here");
        let result: Result<(), std::io::Error> = Err(error);
        let value = result.context("This is some context")?;
        return Ok(value);
    }

    let result: TracedResult<()> = on_error();
    println!("{:?}", result.as_ref().unwrap_err());
    assert!(result.is_err());
    let message = result.unwrap_err().to_string();
    assert!(
        message.contains("Context:"),
        "Expected context in message:\n{}",
        message
    );
    let result: TracedResult<()> = on_result();
    println!("{:?}", result.as_ref().unwrap_err());
    assert!(result.is_err());
    let message = result.unwrap_err().to_string();
    assert!(
        message.contains("Context:"),
        "Expected context in message:\n{}",
        message
    );
    let result: TracedResult<()> = on_result_again();
    println!("{:?}", result.as_ref().unwrap_err());
    assert!(result.is_err());
    let message = result.unwrap_err().to_string();
    assert!(
        message.contains("Context:"),
        "Expected context in message:\n{}",
        message
    );
}

#[test]
#[cfg_attr(not(feature = "min_specialization"), should_panic)]
fn nesting_traced_dyn_calls() {
    fn func1() -> TracedResult<()> {
        eros::bail!("This is a bailing message {}", 1);
    }

    fn func2() -> TracedResult<()> {
        func1()
            .context("One")
            .traced_dyn()
            .context("Two")
            .traced_dyn()
            .context("Three")
    }

    let result: TracedResult<()> = func2();
    let message = result.unwrap_err().to_string();

    let count = message.match_indices("Context:").count();
    assert_eq!(count, 1, "Expected only one 'Context:', got:\n{}", message);
}
