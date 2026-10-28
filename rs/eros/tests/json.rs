#![cfg(feature = "json")]

use std::{error::Error, fmt};

use eros::{ErrorUnion, ErrorUnionVec, MsgError};
use serde_json::Value;
#[cfg(feature = "error_span_trace")]
use tracing_subscriber::prelude::*;

fn with_span<T>(f: impl FnOnce() -> T) -> T {
    let subscriber = tracing_subscriber::registry();
    #[cfg(feature = "error_span_trace")]
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

#[cfg(not(feature = "error_span_trace"))]
#[test]
fn debug_json_omits_span_trace_when_disabled_for_typed_and_erased_errors() {
    // Active tracing spans must not add JSON fields when Eros's feature is off.
    let error: ErrorUnion<(MsgError,)> = with_span(|| ErrorUnion::new(MsgError::from("failure")));
    let typed = [error.to_debug_json(), error.report().to_debug_json()];
    let erased = error.erase().adapter();
    for value in typed.into_iter().chain([
        erased.as_ref().to_debug_json(),
        erased.as_ref().report().to_debug_json(),
    ]) {
        assert_span_trace_absent(&value);
    }
}

#[cfg(not(feature = "error_span_trace"))]
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

#[cfg(feature = "error_span_trace")]
#[test]
fn debug_json_uses_null_when_enabled_without_a_capture_at_every_batch_depth() {
    let error = tracing::subscriber::with_default(tracing_subscriber::registry(), nested_batch);
    for value in [error.to_debug_json(), error.report().to_debug_json()] {
        for node in nodes(&value) {
            assert_eq!(node.get("span_trace"), Some(&Value::Null));
        }
    }
}

#[cfg(feature = "error_span_trace")]
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

#[derive(Debug)]
struct Message {
    text: &'static str,
    source: Option<Box<Message>>,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Split CRLF across writes, as real custom Display implementations can.
        for character in self.text.chars() {
            write!(f, "{character}")?;
        }
        Ok(())
    }
}

impl Error for Message {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|source| source as &dyn Error)
    }
}

#[test]
fn json_messages_match_human_line_endings_without_losing_content_or_field_boundaries() {
    for (text, expected) in [
        (
            "🦀 invalid café\r\n检查输入\r\n",
            "🦀 invalid café\n检查输入",
        ),
        (" first \r\n\r\n last \n\n", " first \n\n last \n"),
        (
            "literal\ncaused by:\n    - source\ncontext:\n    - frame\n",
            "literal\ncaused by:\n    - source\ncontext:\n    - frame",
        ),
        ("lone\rcarriage return\r", "lone\rcarriage return\r"),
        ("", ""),
        ("\r\n", ""),
        ("\n\n", "\n"),
    ] {
        let error: ErrorUnion<(Message,)> = ErrorUnion::new(Message {
            text,
            source: Some(Box::new(Message {
                text: "source\r\ncontinuation\r\n",
                source: Some(Box::new(Message {
                    text: "last source\n",
                    source: None,
                })),
            })),
        });
        #[cfg(feature = "context")]
        let error = error.context(text);
        let error = ErrorUnionVec::new(vec![error]).into_union::<(ErrorUnionVec<(Message,)>,), _>();
        if expected.contains('\n') {
            let indented = expected.replace('\n', "\n            ");
            assert!(
                format!("{error:#?}").contains(&format!(
                    "- error:\n            {indented}\n          at: [0]"
                )),
                "batch Debug must retain the same multiline message as JSON"
            );
        }
        for value in [
            error.to_display_json(),
            error.report().to_display_json(),
            error.to_debug_json(),
            error.report().to_debug_json(),
        ] {
            let child = &value["children"][0];
            assert_eq!(child["root"], expected);
            assert_eq!(
                child["sources"],
                serde_json::json!(["source\ncontinuation", "last source"])
            );
            if let Some(contexts) = child.get("contexts") {
                #[cfg(feature = "context")]
                assert_eq!(contexts[0]["message"], expected);
                #[cfg(not(feature = "context"))]
                assert_eq!(contexts, &serde_json::json!([]));
            }
        }
        error.report().visit_children(&mut |child| {
            assert_eq!(format!("{child:#}"), expected);
        });
    }
}

#[cfg(feature = "user_context")]
#[test]
fn debug_json_preserves_user_context_flags_at_every_batch_depth_after_erasure() {
    let leaf = leaf("failure")
        .context("leaf detail")
        .with_user_context(|| "leaf hint");
    let inner: ErrorUnion = ErrorUnionVec::new(vec![leaf])
        .into_union::<(ErrorUnionVec,), _>()
        .user_context("inner hint")
        .context("inner detail")
        .erase();
    let outer: ErrorUnion = ErrorUnionVec::new(vec![inner])
        .into_union::<(ErrorUnionVec,), _>()
        .context("outer detail")
        .user_context("outer hint")
        .erase();
    let adapter = outer.adapter();
    for value in [
        adapter.as_ref().to_debug_json(),
        adapter.as_ref().report().to_debug_json(),
    ] {
        for (node, flags) in [
            (&value, [false, true]),
            (&value["children"][0], [true, false]),
            (&value["children"][0]["children"][0], [false, true]),
        ] {
            let contexts = node["contexts"].as_array().unwrap();
            assert_eq!(contexts.len(), flags.len());
            for (context, flag) in contexts.iter().zip(flags) {
                assert_eq!(context.get("user_context"), Some(&Value::Bool(flag)));
                assert!(context.get("user_facing").is_none());
            }
        }
    }
}

#[cfg(all(feature = "context", not(feature = "user_context")))]
#[test]
fn debug_json_omits_user_context_flags_when_disabled_at_every_batch_depth() {
    let leaf = leaf("failure").context("leaf detail");
    let inner: ErrorUnion = ErrorUnionVec::new(vec![leaf])
        .into_union::<(ErrorUnionVec,), _>()
        .context("inner detail")
        .erase();
    let outer: ErrorUnion = ErrorUnionVec::new(vec![inner])
        .into_union::<(ErrorUnionVec,), _>()
        .context("outer detail")
        .erase();
    let adapter = outer.adapter();
    for value in [
        adapter.as_ref().to_debug_json(),
        adapter.as_ref().report().to_debug_json(),
    ] {
        for node in [
            &value,
            &value["children"][0],
            &value["children"][0]["children"][0],
        ] {
            let contexts = node["contexts"].as_array().unwrap();
            assert_eq!(contexts.len(), 1);
            assert!(contexts[0].get("user_context").is_none());
            assert!(contexts[0].get("user_facing").is_none());
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Output {
    Text(&'static str),
    Error,
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(text) => f.write_str(text),
            Self::Error => {
                f.write_str("partial message must be discarded")?;
                Err(fmt::Error)
            }
        }
    }
}

#[derive(Debug)]
struct Diagnostic {
    output: Output,
    source: Option<Box<Diagnostic>>,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.output.fmt(f)
    }
}

impl Error for Diagnostic {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|source| source as &dyn Error)
    }
}

#[test]
fn json_omits_failed_messages_and_preserves_other_diagnostics() {
    let failure = Output::Error;
    let error: ErrorUnion<(Diagnostic,)> = ErrorUnion::new(Diagnostic {
        output: failure,
        source: Some(Box::new(Diagnostic {
            output: Output::Text("first source"),
            source: Some(Box::new(Diagnostic {
                output: failure,
                source: Some(Box::new(Diagnostic {
                    output: Output::Text("last source"),
                    source: None,
                })),
            })),
        })),
    });
    #[cfg(feature = "context")]
    let error = error
        .context("first context")
        .context({
            let value: Box<dyn eros::SendSyncDebugDisplay> = Box::new(failure);
            value
        })
        .context("last context");

    let assert_node = |node: &Value, detailed: bool| {
        assert!(node.get("root").is_none(), "{node}");
        assert_eq!(
            node["sources"],
            serde_json::json!(["first source", "last source"])
        );
        if detailed {
            let contexts = node.get("contexts").expect("preserved debug contexts");
            #[cfg(feature = "context")]
            {
                let messages: Vec<_> = contexts
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|frame| frame["message"].as_str().unwrap())
                    .collect();
                assert_eq!(messages, ["first context", "last context"]);
            }
            #[cfg(not(feature = "context"))]
            assert_eq!(contexts, &serde_json::json!([]));
        }
    };

    for (value, detailed) in [
        (error.to_display_json(), false),
        (error.report().to_display_json(), false),
        (error.to_debug_json(), true),
        (error.report().to_debug_json(), true),
    ] {
        assert_node(&value, detailed);
    }
    let error: ErrorUnion =
        ErrorUnionVec::new(vec![leaf("first child"), error.erase(), leaf("last child")])
            .into_union();
    let adapter = error.adapter();
    for (value, detailed) in [
        (adapter.as_ref().to_display_json(), false),
        (adapter.as_ref().report().to_display_json(), false),
        (adapter.as_ref().to_debug_json(), true),
        (adapter.as_ref().report().to_debug_json(), true),
    ] {
        assert_eq!(value["children"].as_array().unwrap().len(), 3);
        assert_eq!(value["children"][0]["root"], "first child");
        assert_node(&value["children"][1], detailed);
        assert_eq!(value["children"][2]["root"], "last child");
    }
}
