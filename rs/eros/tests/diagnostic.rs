#![cfg(feature = "diagnostic")]

use eros::{ErrorUnion, ErrorUnionVec, MsgError};
use serde_json::Value;
#[cfg(feature = "error_spantrace")]
use tracing_subscriber::prelude::*;

fn with_span<T>(f: impl FnOnce() -> T) -> T {
    let subscriber = tracing_subscriber::registry();
    #[cfg(feature = "error_spantrace")]
    let subscriber = subscriber.with(tracing_error::ErrorLayer::default());
    tracing::subscriber::with_default(subscriber, || {
        tracing::info_span!("operation", request_id = 42).in_scope(f)
    })
}

fn leaf(message: &'static str) -> ErrorUnion {
    ErrorUnion::new(MsgError::from(message))
}

fn nested_batch() -> ErrorUnion {
    let inner: ErrorUnion<(ErrorUnionVec,)> =
        ErrorUnionVec::new(vec![leaf("first failure"), leaf("second failure")]).into_union();
    let outer: ErrorUnion<(ErrorUnionVec,)> =
        ErrorUnionVec::new(vec![inner.erase(), leaf("third failure")]).into_union();
    outer.erase()
}

fn nodes(value: &Value) -> [&Value; 5] {
    let nodes = [
        value,
        &value["children"][0],
        &value["children"][0]["children"][0],
        &value["children"][0]["children"][1],
        &value["children"][1],
    ];
    for node in nodes {
        assert!(
            node.get("root").is_some(),
            "expected a report object: {node}"
        );
    }
    nodes
}

fn assert_span_trace_absent(value: &Value) {
    // Indexing a missing key also returns null. Check membership to distinguish
    // an omitted field from an explicitly serialized null.
    assert!(
        !value.as_object().unwrap().contains_key("span_trace"),
        "span_trace must be omitted: {value}"
    );
}

#[cfg(not(feature = "error_spantrace"))]
#[test]
fn debug_json_omits_span_trace_when_disabled_for_typed_and_erased_errors() {
    // Active tracing spans must not add JSON fields when Eros's feature is off.
    let error: ErrorUnion<(MsgError,)> = with_span(|| ErrorUnion::new(MsgError::from("failure")));
    let typed = [error.to_debug_json(), error.report().to_debug_json()];
    let erased = error.erase().into_adapter();
    for value in typed.into_iter().chain([
        erased.as_ref().to_debug_json(),
        erased.as_ref().report().to_debug_json(),
    ]) {
        assert_span_trace_absent(&value);
    }
}

#[cfg(not(feature = "error_spantrace"))]
#[test]
fn debug_json_omits_span_trace_when_disabled_at_every_batch_depth() {
    let error = with_span(nested_batch);
    for value in [error.to_debug_json(), error.report().to_debug_json()] {
        for node in nodes(&value) {
            assert_span_trace_absent(node);
        }
    }
}

#[cfg(not(feature = "backtrace"))]
#[test]
fn debug_json_omits_backtrace_when_disabled_at_every_batch_depth() {
    let error = with_span(nested_batch);
    for value in [error.to_debug_json(), error.report().to_debug_json()] {
        for node in nodes(&value) {
            assert!(node.get("backtrace").is_none(), "{node}");
        }
    }
}

#[cfg(feature = "error_spantrace")]
#[test]
fn debug_json_uses_null_when_enabled_without_a_capture_at_every_batch_depth() {
    let error = tracing::subscriber::with_default(tracing_subscriber::registry(), nested_batch);
    for value in [error.to_debug_json(), error.report().to_debug_json()] {
        for node in nodes(&value) {
            assert_eq!(node.get("span_trace"), Some(&Value::Null));
        }
    }
}

#[cfg(feature = "error_spantrace")]
#[test]
fn debug_json_uses_trace_text_directly_when_captured_at_every_batch_depth() {
    let error = with_span(nested_batch);
    for value in [error.to_debug_json(), error.report().to_debug_json()] {
        for node in nodes(&value) {
            let trace = node["span_trace"]
                .as_str()
                .expect("captured trace must be a string");
            assert!(trace.contains("operation"), "{trace}");
            assert!(trace.contains("request_id=42"), "{trace}");
        }
    }
}

#[test]
fn display_json_always_omits_both_traces_at_every_batch_depth() {
    let error = with_span(nested_batch);
    for value in [error.to_display_json(), error.report().to_display_json()] {
        for node in nodes(&value) {
            assert_span_trace_absent(node);
            assert!(node.get("backtrace").is_none(), "{node}");
        }
    }
}
