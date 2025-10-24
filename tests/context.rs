#![cfg(all(feature = "context", feature = "backtrace"))]

use std::any::Any;

use eros::{AbsentValueError, Context, ErrorUnion, TracedDyn, Union};

#[cfg(feature = "min_specialization")]
#[cfg(test)]
mod min_specialization {
    use eros::{traced, Context, ErrorUnion, ReshapeUnion, TracedError};

    #[test]
    fn error_union() {
        fn func1() -> eros::UResult<(), (std::io::Error,)> {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AddrInUse,
                "Address in use message here",
            )
            .into());
        }

        fn func2() -> Result<(), ErrorUnion<(i32, std::io::Error)>> {
            func1().widen().context("From func2".to_string())
        }

        fn func3() -> eros::UResult<(), (std::io::Error, i32, bool)> {
            return func2()
                .with_context(|| "From func3")
                .map_err(ErrorUnion::widen);
        }

        fn func4() -> eros::UResult<(), (std::io::Error, bool)> {
            return match func3().with_context(|| "From func4").narrow::<i32, _>() {
                Ok(_) => panic!("should exist"),
                Err(result) => result,
            };
        }

        fn func5() -> eros::UResult<(), (std::io::Error, bool, TracedError)> {
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
    fn func1() -> Result<(), eros::TracedError> {
        return Err(eros::traced!("This is root error message"));
    }

    fn func2() -> eros::Result<()> {
        func1().context("Generic context")
    }

    fn func3() -> Result<(), ErrorUnion<(std::io::Error, eros::TracedError)>> {
        func2().map_err(ErrorUnion::new)
    }

    let result: Result<(), ErrorUnion<(std::io::Error, eros::TracedError)>> = func3();
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
    fn func1() -> Result<(), eros::TracedError> {
        eros::bail!("This is root error message")
    }

    fn func2() -> Result<(), ErrorUnion<(std::io::Error, eros::TracedError)>> {
        func1().map_err(ErrorUnion::new)
    }

    let result: Result<(), ErrorUnion<(std::io::Error, eros::TracedError)>> = func2();
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
    fn func1() -> eros::Result<()> {
        eros::bail!("This is a bailing message {}", 1);
    }

    fn func2() -> eros::UResult<(), (eros::TracedError,)> {
        func1().context("From func2".to_string()).union()
    }

    fn func3() -> Result<(), ErrorUnion<(eros::TracedError, i32, bool)>> {
        return func2().map_err(ErrorUnion::widen);
    }

    fn func4() -> eros::Result<()> {
        let error =
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here")
                .traced_dyn();
        return Err(error);
    }

    let result: Result<(), ErrorUnion<(eros::TracedError, i32, bool)>> = func3();
    println!("{:?}", result.as_ref().unwrap_err());
    assert!(result.is_err());
    let message = result.unwrap_err().to_string();
    assert!(
        message.contains("Context:"),
        "Expected context in message:\n{}",
        message
    );
    let result: eros::Result<()> = func4();
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
fn ensure() {
    fn func1() -> eros::Result<()> {
        eros::ensure!(1 == 2, "This is a bailing message {}", 1);
        Ok(())
    }

    fn func2() -> eros::UResult<(), (eros::TracedError,)> {
        func1().context("From func2".to_string()).union()
    }

    fn func3() -> Result<(), ErrorUnion<(eros::TracedError, i32, bool)>> {
        return func2().map_err(ErrorUnion::widen);
    }

    fn func4() -> eros::Result<()> {
        let error =
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here")
                .traced_dyn();
        return Err(error);
    }

    let result: Result<(), ErrorUnion<(eros::TracedError, i32, bool)>> = func3();
    println!("{:?}", result.as_ref().unwrap_err());
    assert!(result.is_err());
    let message = result.unwrap_err().to_string();
    assert!(
        message.contains("Context:"),
        "Expected context in message:\n{}",
        message
    );
    let result: eros::Result<()> = func4();
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
    fn on_error() -> eros::Result<()> {
        let error =
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here")
                .context("This is some context");
        return Err(error);
    }

    fn on_result() -> eros::Result<()> {
        let error =
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here");
        let result: Result<(), std::io::Error> = Err(error);
        let value = result.context("This is some context")?;
        return Ok(value);
    }

    fn on_result_again() -> eros::Result<()> {
        let error =
            std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address in use message here");
        let result: Result<(), std::io::Error> = Err(error);
        let value = result.context("This is some context")?;
        return Ok(value);
    }

    let result: eros::Result<()> = on_error();
    println!("{:?}", result.as_ref().unwrap_err());
    assert!(result.is_err());
    let message = result.unwrap_err().to_string();
    assert!(
        message.contains("Context:"),
        "Expected context in message:\n{}",
        message
    );
    let result: eros::Result<()> = on_result();
    println!("{:?}", result.as_ref().unwrap_err());
    assert!(result.is_err());
    let message = result.unwrap_err().to_string();
    assert!(
        message.contains("Context:"),
        "Expected context in message:\n{}",
        message
    );
    let result: eros::Result<()> = on_result_again();
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
fn absent_value_error() {
    fn func1() -> eros::Result<()> {
        None.context("This value should be some")
    }

    let result = func1().context("Some context");
    println!("{:?}", result);
    let error = result.unwrap_err();
    let inner_error = error.into_inner() as Box<dyn Any>;
    assert!(inner_error.downcast::<AbsentValueError>().is_ok());
}

#[test]
#[cfg_attr(not(feature = "min_specialization"), should_panic)]
fn nesting_traced_dyn_calls() {
    fn func1() -> eros::Result<()> {
        eros::bail!("This is a bailing message {}", 1);
    }

    fn func2() -> eros::Result<()> {
        func1()
            .context("One")
            .traced_dyn()
            .context("Two")
            .traced_dyn()
            .context("Three")
    }

    let result: eros::Result<()> = func2();
    let message = result.unwrap_err().to_string();

    let count = message.match_indices("Context:").count();
    assert_eq!(count, 1, "Expected only one 'Context:', got:\n{}", message);
}

#[cfg(feature = "anyhow")]
#[test]
fn integration_with_anyhow() {
    fn anyhow_result() -> anyhow::Result<()> {
        use anyhow::Context;
        // Err(anyhow::anyhow!("This is the root from anyhow")).context("This is context from anyhow")
        Err(anyhow::Error::from(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            "This is the root",
        ))).context("This is some anyhow context")
    }

    // let error: Box<std::io::Error> = anyhow_result()
    //     .unwrap_err()
    //     .reallocate_into_boxed_dyn_error_without_backtrace().downcast::<std::io::Error>().unwrap();
    // let error = anyhow_result()
    //     .unwrap_err()
    //     .into_boxed_dyn_error();
    // let bind = anyhow_result().unwrap_err();
    // let error = bind.chain().next().unwrap();
    // println!("{error:?}");

    fn eros_result() -> eros::Result<()> {
        use eros::TracedError;

        anyhow_result().map_err(TracedError::anyhow)?;
        Ok(())
    }

    let result = eros_result().context("eros context");

    println!("{:?}", result.as_ref().unwrap_err());
}
