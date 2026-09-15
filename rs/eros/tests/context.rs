#![cfg(all(feature = "context", feature = "backtrace"))]

use eros::{
    AbsentValueError, AnyError, Context, ErrorUnion, IntoAnyUnion, IntoUnion, ReshapeUnion, error,
};

#[test]
fn error_union() {
    fn can_yeet_from_regular_result() -> eros::Result<(), (std::io::Error,)> {
        Err(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            "Address in use message here",
        ))?;
        Ok(())
    }

    fn widen_then_context() -> Result<(), ErrorUnion<(std::sync::mpsc::RecvError, std::io::Error)>>
    {
        can_yeet_from_regular_result()
            .widen()
            .context("From func2".to_string())
    }

    fn map_widen() -> eros::Result<(), (std::io::Error, std::sync::mpsc::RecvError, std::fmt::Error)>
    {
        widen_then_context()
            .with_context(|| "From func3")
            .map_err(ErrorUnion::widen)?;
        Ok(())
    }

    fn narrow() -> eros::Result<(), (std::io::Error, std::sync::mpsc::RecvError)> {
        match map_widen()
            .with_context(|| "From func4")
            .narrow::<std::fmt::Error, _>()
        {
            Ok(_) => panic!("should exist"),
            Err(result) => result,
        }
    }

    fn traced_macro() -> eros::Result<()> {
        Err(error!("Error")).context("From func5")
    }

    let result: Result<
        (),
        ErrorUnion<(std::io::Error, std::sync::mpsc::RecvError, std::fmt::Error)>,
    > = map_widen();
    assert!(result.is_err());
    let error = result.unwrap_err();
    let message = format!("{:?}", error);
    assert!(
        message.contains("Context (innermost first):"),
        "Expected context in message:\n{}",
        message
    );
    let message = format!("{}", error);
    assert!(
        !message.contains("Context (innermost first):"),
        "Expected no context in message:\n{}",
        message
    );
    let result: Result<(), ErrorUnion<(std::io::Error, std::sync::mpsc::RecvError)>> = narrow();
    assert!(result.is_err());
    let message = format!("{:?}", result.unwrap_err());
    assert!(
        message.contains("Context (innermost first):"),
        "Expected context in message:\n{}",
        message
    );
    let result: Result<(), ErrorUnion<AnyError>> = traced_macro();
    assert!(result.is_err());
    let message = format!("{:?}", result.unwrap_err());
    assert!(
        message.contains("Context (innermost first):"),
        "Expected context in message:\n{}",
        message
    );
}

#[test]
fn generic_context_error_to_traced_error_union() {
    fn yeet_a_regular() -> Result<(), std::io::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            "Address in use message here",
        ))?;
        Ok(())
    }

    fn yeet_regular_into_union_any_error() -> Result<(), eros::ErrorUnion> {
        yeet_a_regular()?;
        Ok(())
    }

    fn yeet_regular_into_union_explicit() -> Result<(), eros::ErrorUnion<(std::io::Error,)>> {
        yeet_a_regular()?;
        Ok(())
    }

    fn yeet_regular_into_union_multiple_explicit()
    -> Result<(), eros::ErrorUnion<(std::sync::mpsc::RecvError, std::io::Error, std::fmt::Error)>>
    {
        // yeet_a_regular()?; // todo ideally this should work
        yeet_a_regular().union()?;
        Ok(())
    }

    fn yeet_widen_union() -> Result<(), eros::ErrorUnion<(std::fmt::Error, std::io::Error)>> {
        // yeet_regular_into_union_explicit()?; // todo ideally this should work
        yeet_regular_into_union_explicit().widen()?;
        Ok(())
    }

    yeet_a_regular().unwrap_err();
    yeet_regular_into_union_any_error().unwrap_err();
    yeet_regular_into_union_explicit().unwrap_err();
    yeet_regular_into_union_multiple_explicit().unwrap_err();
    yeet_widen_union().unwrap_err();
}

#[test]
fn bail() {
    fn func1() -> eros::Result<()> {
        eros::bail!("This is a bailing message {}", 1);
    }

    fn func2() -> eros::Result<(), AnyError> {
        func1().context("From func2".to_string())
    }

    let result: Result<(), ErrorUnion> = func2();
    println!("{:?}", result.as_ref().unwrap_err());
    assert!(result.is_err());
    let message = format!("{:?}", result.unwrap_err());
    assert!(
        message.contains("Context (innermost first):"),
        "Expected context in message:\n{}",
        message
    );
}

#[test]
fn ensure() {
    fn func1() -> eros::Result<()> {
        eros::ensure!(1 == 2, "This is a bailing message {}", 1);
        Ok(())
    }

    fn func2() -> eros::Result<(), AnyError> {
        func1().context("From func2".to_string())
    }

    let result: Result<(), ErrorUnion> = func2();
    println!("{:?}", result.as_ref().unwrap_err());
    assert!(result.is_err());
    let message = format!("{:?}", result.unwrap_err());
    assert!(
        message.contains("Context (innermost first):"),
        "Expected context in message:\n{}",
        message
    );
}

#[test]
fn absent_value_error() {
    fn func1() -> eros::Result<()> {
        // None.context("This value should be some").into() // todo ideally this should work
        None.context("This value should be some").any_union()
    }

    let result = func1().context("Some context");
    println!("{:?}", result);
    let error = result.unwrap_err();
    let inner_error = error.inner() as &dyn std::any::Any;
    assert!(inner_error.is::<AbsentValueError>());
}

#[test]
fn nesting_traced_dyn_calls() {
    fn func1() -> eros::Result<()> {
        eros::bail!("This is a bailing message {}", 1);
    }

    fn func2() -> eros::Result<()> {
        func1()
            .context("One") // creates union
            .any_union() // should not nest it
            .context("Two")
            .any_union() // should not nest it
            .context("Three")
    }

    let result: eros::Result<()> = func2();
    let message = format!("{:?}", result.unwrap_err());

    let count = message.match_indices("Context (innermost first):").count();
    assert_eq!(count, 1, "Expected only one 'Context (innermost first):', got:\n{}", message);
    println!("{}", message);
}

#[cfg(feature = "anyhow")]
#[test]
fn integration_with_anyhow() {
    fn anyhow_result() -> anyhow::Result<()> {
        use anyhow::Context;
        Err(anyhow::Error::from(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            "This is the root",
        )))
        .context("This is some anyhow context")
    }

    fn eros_result() -> eros::Result<()> {
        use eros::ErrorUnion;

        anyhow_result().map_err(ErrorUnion::from_anyhow)?;
        Ok(())
    }

    let result = eros_result().context("eros context");
    let error = result.as_ref().unwrap_err();

    assert_eq!(
        error.to_string(),
        "This is some anyhow context <- This is the root"
    );
    assert_eq!(
        format!("{error:#?}"),
        "This is some anyhow context\n  caused by: This is the root\n\n  Context (innermost first):\n    1. eros context"
    );
}
