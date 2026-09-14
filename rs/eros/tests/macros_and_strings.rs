use eros::{ContextSource, StrError};
use std::{borrow::Cow, cell::Cell, error::Error};

#[test]
fn string_error_conversions_formatting_and_clone_preserve_storage() {
    for error in [
        StrError::from("static"),
        StrError::from(Cow::Borrowed("static")),
    ] {
        assert!(matches!(error, StrError::Static("static")));
        let clone = error.clone();
        assert!(matches!(clone, StrError::Static("static")));
        assert!(std::ptr::eq(error.as_str(), clone.as_str()));
        assert_eq!(error, clone);
        assert_eq!(error.to_string(), "static");
        assert_eq!(format!("{error:?}"), "static");
        assert!(error.source().is_none());
    }
    for mut error in [
        StrError::from(String::from("owned")),
        StrError::from(Cow::Owned(String::from("owned"))),
    ] {
        let clone = error.clone();
        assert_eq!(error, clone);
        assert_eq!(error.to_string(), "owned");
        assert_eq!(format!("{error:?}"), "owned");
        assert!(error.source().is_none());
        let StrError::Owned(value) = &mut error else {
            panic!("expected owned storage")
        };
        value.push_str(" changed");
        assert_eq!(clone.as_str(), "owned");
        assert!(matches!(clone, StrError::Owned(_)));
    }
}

#[test]
fn string_errors_support_equality_ordering_and_hashing() {
    let mut values = std::collections::BTreeSet::new();
    values.insert(StrError::from("b"));
    values.insert(StrError::from("a"));
    values.insert(StrError::from("a"));
    assert_eq!(
        values
            .into_iter()
            .map(|value| value.as_str().to_owned())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );
    let values = std::collections::HashSet::from([
        StrError::from(String::from("a")),
        StrError::from(String::from("a")),
    ]);
    assert_eq!(values.len(), 1);
}

#[test]
fn error_macro_supports_literals_formatted_messages_and_error_expressions() {
    for error in [eros::error!("literal"), eros::error!("literal",)] {
        assert!(matches!(
            error.downcast_inner::<StrError>(),
            Some(StrError::Static("literal"))
        ));
    }
    let calls = Cell::new(0);
    let error = eros::error!(
        "value {name}: {}",
        {
            calls.set(calls.get() + 1);
            7
        },
        name = "item"
    );
    assert_eq!(calls.get(), 1);
    assert!(
        matches!(error.downcast_inner::<StrError>(), Some(StrError::Owned(value)) if value == "value item: 7")
    );
    let error = eros::error!({
        calls.set(calls.get() + 1);
        std::fmt::Error
    },);
    assert_eq!(calls.get(), 2);
    assert_eq!(
        error.downcast_inner::<std::fmt::Error>(),
        Some(std::fmt::Error)
    );
}

#[test]
fn bail_macro_returns_early_for_each_message_form() {
    fn literal() -> eros::Result<()> {
        eros::bail!("literal",)
    }
    fn formatted(value: u8) -> eros::Result<()> {
        eros::bail!("value {}", value,)
    }
    fn expression() -> eros::Result<()> {
        eros::bail!(std::fmt::Error,)
    }
    assert_eq!(literal().unwrap_err().to_string(), "literal");
    assert_eq!(formatted(7).unwrap_err().to_string(), "value 7");
    assert!(expression().unwrap_err().is_inner::<std::fmt::Error>());
}

#[test]
fn ensure_evaluates_the_condition_once_and_only_builds_errors_on_failure() {
    fn literal(ok: bool) -> eros::Result<u8> {
        eros::ensure!(ok, "literal",);
        Ok(42)
    }
    fn expression(ok: bool, conditions: &Cell<u8>, errors: &Cell<u8>) -> eros::Result<u8> {
        eros::ensure!(
            {
                conditions.set(conditions.get() + 1);
                ok
            },
            {
                errors.set(errors.get() + 1);
                std::fmt::Error
            },
        );
        Ok(42)
    }
    fn formatted(ok: bool, formats: &Cell<u8>) -> eros::Result<u8> {
        eros::ensure!(ok, "value {}", {
            formats.set(formats.get() + 1);
            7
        },);
        Ok(42)
    }
    let conditions = Cell::new(0);
    let errors = Cell::new(0);
    let formats = Cell::new(0);
    assert_eq!(literal(true).unwrap(), 42);
    assert_eq!(expression(true, &conditions, &errors).unwrap(), 42);
    assert_eq!(formatted(true, &formats).unwrap(), 42);
    assert_eq!((conditions.get(), errors.get(), formats.get()), (1, 0, 0));
    assert_eq!(literal(false).unwrap_err().to_string(), "literal");
    assert!(
        expression(false, &conditions, &errors)
            .unwrap_err()
            .is_inner::<std::fmt::Error>()
    );
    assert_eq!(
        formatted(false, &formats).unwrap_err().to_string(),
        "value 7"
    );
    assert_eq!((conditions.get(), errors.get(), formats.get()), (2, 1, 1));
}

#[eros::context("branch {}", { calls.set(calls.get() + 1); branch })]
fn attributed(branch: u8, calls: &Cell<u8>) -> eros::Result<u8> {
    let _ = calls;
    match branch {
        0 => Ok(42),
        1 => {
            Err::<(), _>(std::fmt::Error)?;
            unreachable!()
        }
        2 => return Err(eros::error!("early return")),
        _ => Err(eros::error!("tail error")),
    }
}

#[test]
fn context_attribute_handles_all_exit_paths_and_formats_lazily() {
    let calls = Cell::new(0);
    assert_eq!(attributed(0, &calls).unwrap(), 42);
    assert_eq!(calls.get(), 0);
    for branch in 1..=3 {
        let error = attributed(branch, &calls).unwrap_err();
        assert_eq!(
            format!("{error:#?}")
                .matches(&format!("branch {branch}"))
                .count(),
            usize::from(cfg!(feature = "context"))
        );
    }
    assert_eq!(calls.get(), if cfg!(feature = "context") { 3 } else { 0 });
}

#[eros::context]
fn auto_context(#[fmt("{:04}")] value: u8, ignored: u8) -> eros::Result<()> {
    let _ = (value, ignored);
    eros::bail!("root")
}

#[test]
fn auto_context_attribute_uses_only_annotated_parameters() {
    let error = auto_context(7, 99).unwrap_err();
    let report = format!("{error:#?}");
    assert_eq!(report.contains("value: 0007"), cfg!(feature = "context"));
    assert!(!report.contains("ignored"));
}

#[test]
fn disabled_context_does_not_change_the_latest_error() {
    let context = ContextSource::Error(Box::new(std::fmt::Error));
    let error = eros::error!("root").context(context).context("last string");
    if cfg!(feature = "context") {
        assert!(error.latest_error().as_any().is::<std::fmt::Error>());
    } else {
        assert!(error.latest_error().as_any().is::<StrError>());
    }
}
