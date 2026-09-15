use eros::{ContextValue, MsgError};
use std::{borrow::Cow, cell::Cell, error::Error};

#[test]
fn string_error_conversions_formatting_and_clone_preserve_storage() {
    const STATIC: MsgError = MsgError::from_static("static");
    for error in [
        STATIC,
        MsgError::from("static"),
        MsgError::from(Cow::Borrowed("static")),
    ] {
        let clone = error.clone();
        assert!(std::ptr::eq(error.as_str(), clone.as_str()));
        assert_eq!(error, clone);
        assert_eq!(error.to_string(), "static");
        assert_eq!(format!("{error:?}"), "static");
        assert!(error.source().is_none());
    }
    let owned = String::from("owned");
    let original_ptr = owned.as_ptr();
    let error = MsgError::from_owned(owned);
    assert_eq!(error.as_str().as_ptr(), original_ptr);
    for error in [
        error,
        MsgError::from(String::from("owned")),
        MsgError::from(Cow::Owned(String::from("owned"))),
    ] {
        let clone = error.clone();
        assert_eq!(error, clone);
        assert_eq!(error.to_string(), "owned");
        assert_eq!(format!("{error:?}"), "owned");
        assert!(error.source().is_none());
        assert!(!std::ptr::eq(error.as_str(), clone.as_str()));
        drop(error);
        assert_eq!(clone.as_str(), "owned");
    }
}

#[test]
fn string_errors_support_equality_ordering_and_hashing() {
    let mut values = std::collections::BTreeSet::new();
    values.insert(MsgError::from("b"));
    values.insert(MsgError::from("a"));
    values.insert(MsgError::from(String::from("a")));
    values.insert(MsgError::from("a"));
    assert_eq!(
        values
            .into_iter()
            .map(|value| value.as_str().to_owned())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );
    let values = std::collections::HashSet::from([
        MsgError::from("a"),
        MsgError::from(String::from("a")),
        MsgError::from(String::from("a")),
    ]);
    assert_eq!(values.len(), 1);
}

#[test]
fn error_macro_supports_literals_formatted_messages_and_error_expressions() {
    for (error, expected) in [
        (eros::error!("literal"), "literal"),
        (eros::error!("literal",), "literal"),
        (eros::error!(""), ""),
        (eros::error!("{{id}}"), "{id}"),
        (
            eros::error!(r#"{{{{"id": "日本語"}}}}"#),
            "{{\"id\": \"日本語\"}}",
        ),
        (eros::error!("\u{7b}\u{7b}id\u{7d}\u{7d}"), "{id}"),
    ] {
        let message = error.downcast_inner::<MsgError>().unwrap();
        assert_eq!(message.as_str(), expected);
        assert!(std::ptr::eq(message.as_str(), message.clone().as_str()));
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
    let message = error.downcast_inner::<MsgError>().unwrap();
    assert_eq!(message.as_str(), "value item: 7");
    assert!(!std::ptr::eq(message.as_str(), message.clone().as_str()));
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
fn error_macro_uses_static_storage_for_all_caps_names() {
    static ERROR: &str = "User {id} {{not found}}";
    const _ERROR_404: &str = "User not found";
    mod messages {
        pub static NOT_FOUND: &str = "User not found";
    }
    macro_rules! forwarded {
        ($message:expr) => {
            eros::error!($message)
        };
    }

    for (error, expected) in [
        (eros::error!(ERROR), ERROR),
        (eros::error!(ERROR,), ERROR),
        (eros::error!(r#ERROR), ERROR),
        (eros::error!(_ERROR_404), _ERROR_404),
        (eros::error!(messages::NOT_FOUND), messages::NOT_FOUND),
        (forwarded!(ERROR), ERROR),
    ] {
        let message = error.downcast_inner::<MsgError>().unwrap();
        assert_eq!(message.as_str(), expected);
        assert!(std::ptr::eq(message.as_str(), expected));
    }
}

#[test]
fn error_macro_preserves_other_error_expressions() {
    use std::fmt::Error as FormatError;
    let error = FormatError;
    let _123 = FormatError;
    const ERROR: std::fmt::Error = std::fmt::Error;

    for error in [
        eros::error!(error),
        eros::error!(_123),
        eros::error!(FormatError),
        eros::error!({ ERROR }),
    ] {
        assert!(error.is_inner::<std::fmt::Error>());
    }
}

#[test]
fn bail_and_ensure_support_all_caps_messages() {
    static ERROR: &str = "User not found";
    fn bail() -> eros::Result<()> {
        eros::bail!(ERROR)
    }
    fn bail_with_trailing_comma() -> eros::Result<()> {
        eros::bail!(ERROR,)
    }
    fn ensure(ok: bool) -> eros::Result<()> {
        eros::ensure!(ok, ERROR);
        Ok(())
    }
    fn ensure_with_trailing_comma(ok: bool) -> eros::Result<()> {
        eros::ensure!(ok, ERROR,);
        Ok(())
    }

    ensure(true).unwrap();
    ensure_with_trailing_comma(true).unwrap();
    for error in [
        bail().unwrap_err(),
        bail_with_trailing_comma().unwrap_err(),
        ensure(false).unwrap_err(),
        ensure_with_trailing_comma(false).unwrap_err(),
    ] {
        let message = error.downcast_inner::<MsgError>().unwrap();
        assert_eq!(message.as_str(), ERROR);
        assert!(std::ptr::eq(message.as_str(), ERROR));
    }
}

#[test]
fn error_macro_formats_captured_arguments_and_escaped_braces() {
    let id = 7;
    for error in [
        eros::error!("User with id {id} not found"),
        eros::error!("User with id {id} not found",),
    ] {
        assert_eq!(error.to_string(), "User with id 7 not found");
        assert!(error.is_inner::<MsgError>());
    }

    let width = 4;
    let name = "item";
    assert_eq!(
        eros::error!(r#"{name:?}: {id:0width$} {{missing}}"#).to_string(),
        "\"item\": 0007 {missing}"
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
    fn captured(value: u8) -> eros::Result<()> {
        eros::bail!("value {value}")
    }
    fn captured_with_trailing_comma(value: u8) -> eros::Result<()> {
        eros::bail!("value {value}",)
    }
    fn expression() -> eros::Result<()> {
        eros::bail!(std::fmt::Error,)
    }
    assert_eq!(literal().unwrap_err().to_string(), "literal");
    assert_eq!(formatted(7).unwrap_err().to_string(), "value 7");
    assert_eq!(captured(7).unwrap_err().to_string(), "value 7");
    assert_eq!(
        captured_with_trailing_comma(7).unwrap_err().to_string(),
        "value 7"
    );
    assert!(expression().unwrap_err().is_inner::<std::fmt::Error>());
}

#[test]
fn ensure_formats_captured_arguments_only_on_failure() {
    struct Value<'a>(&'a Cell<u8>);

    impl std::fmt::Display for Value<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.set(self.0.get() + 1);
            f.write_str("7")
        }
    }

    fn check(ok: bool, value: &Value<'_>) -> eros::Result<()> {
        eros::ensure!(ok, "value {value}");
        Ok(())
    }
    fn check_with_trailing_comma(ok: bool, value: &Value<'_>) -> eros::Result<()> {
        eros::ensure!(ok, "value {value}",);
        Ok(())
    }

    for check in [check, check_with_trailing_comma] {
        let formats = Cell::new(0);
        let value = Value(&formats);
        check(true, &value).unwrap();
        assert_eq!(formats.get(), 0);
        assert_eq!(check(false, &value).unwrap_err().to_string(), "value 7");
        assert_eq!(formats.get(), 1);
    }
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
fn latest_context_error_respects_the_context_feature() {
    let context = ContextValue::from(Box::new(std::fmt::Error) as Box<dyn eros::SendSyncError>);
    let error = eros::error!("root").context(context).context("last string");
    let latest = error.latest_context_error();
    if cfg!(feature = "context") {
        assert!(latest.unwrap().as_any().is::<std::fmt::Error>());
    } else {
        assert!(latest.is_none());
        assert!(
            latest
                .unwrap_or_else(|| error.inner())
                .as_any()
                .is::<MsgError>()
        );
    }
}
