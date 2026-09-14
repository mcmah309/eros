#![cfg(feature = "anyhow")]

use eros::{ErrorUnion, StrError};
use std::sync::Arc;

#[test]
fn converting_to_anyhow_preserves_root_and_context_order() {
    let error = eros::error!("root")
        .context("inner operation")
        .context("outer operation");
    let error: anyhow::Error = error.into();
    assert_eq!(
        error.to_string(),
        if cfg!(feature = "context") {
            "outer operation"
        } else {
            "root"
        }
    );
    let messages: Vec<_> = error.chain().map(ToString::to_string).collect();
    if cfg!(feature = "context") {
        assert_eq!(&messages[..2], ["outer operation", "inner operation"]);
    }
    assert_eq!(messages.last().unwrap(), "root");
    assert!(
        error
            .chain()
            .any(|cause| cause.downcast_ref::<StrError>().is_some())
    );
}

#[test]
fn owned_and_shared_anyhow_adapters_retain_chains_and_shared_ownership() {
    let shared = Arc::new(anyhow::Error::new(StrError::from("root")).context("anyhow context"));
    let shared_union = ErrorUnion::anyhow_arc(shared.clone());
    assert_eq!(Arc::strong_count(&shared), 2);
    let owned_union =
        ErrorUnion::anyhow(anyhow::Error::new(StrError::from("root")).context("anyhow context"));
    for error in [owned_union, shared_union] {
        assert_eq!(error.to_string(), "anyhow context <- root");
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<StrError>()
                .unwrap()
                .as_str(),
            "root"
        );
        assert_eq!(format!("{error:#?}"), "anyhow context\n  caused by: root");
        let error: anyhow::Error = error.context("eros context").into();
        assert_eq!(
            error.to_string(),
            if cfg!(feature = "context") {
                "eros context"
            } else {
                "anyhow context"
            }
        );
        assert_eq!(error.root_cause().to_string(), "root");
    }
    assert_eq!(Arc::strong_count(&shared), 1);
}
