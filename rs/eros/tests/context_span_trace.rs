#![cfg(all(feature = "context", feature = "context_span_trace"))]

use eros::{Context, ErrorUnion, ErrorUnionAdapter, ErrorUnionVec, MsgError, ReshapeUnion};
use std::fmt;
use tracing_error::{ErrorLayer, SpanTrace, SpanTraceStatus};
use tracing_subscriber::prelude::*;

fn with_traces<T>(f: impl FnOnce() -> T) -> T {
    let subscriber = tracing_subscriber::registry().with(ErrorLayer::new(eros::SpanTraceFields));
    tracing::subscriber::with_default(subscriber, f)
}

fn root() -> ErrorUnion<(MsgError,)> {
    ErrorUnion::new(MsgError::from("failure"))
}

fn names(trace: &SpanTrace) -> Vec<&'static str> {
    let mut names = Vec::new();
    trace.with_spans(|metadata, _| {
        names.push(metadata.name());
        true
    });
    names
}

#[test]
fn each_context_retains_its_attachment_span_after_the_subscriber_exits() {
    let error = with_traces(|| {
        let error = tracing::info_span!("origin").in_scope(root);
        let error = tracing::info_span!("request", id = 42).in_scope(|| {
            tracing::info_span!("first_attachment", attempt = 1).in_scope(|| error.context("first"))
        });
        tracing::info_span!("second_attachment").in_scope(|| error.with_context(|| "second"))
    });
    let frames: Vec<_> = error.contexts().collect();
    assert_eq!(
        names(frames[0].span_trace()),
        ["first_attachment", "request"]
    );
    assert_eq!(names(frames[1].span_trace()), ["second_attachment"]);
    assert!(frames[0].span_trace().to_string().contains("id=42"));
    #[cfg(feature = "error_span_trace")]
    assert_eq!(names(error.span_trace()), ["origin"]);
    for (frame, borrowed) in frames.iter().zip(error.report().contexts()) {
        assert!(std::ptr::eq(frame.span_trace(), borrowed.span_trace()));
        assert_eq!(frame.span_trace().status(), SpanTraceStatus::CAPTURED);
    }

    with_traces(|| {
        tracing::info_span!("reporting").in_scope(|| {
            #[cfg(feature = "context_location")]
            let location = format!("      location: {}\n", frames[0].location());
            #[cfg(not(feature = "context_location"))]
            let location = String::new();
            let text = format!("{error:?}");
            assert_eq!(format!("{:?}", error.report()), text);
            assert!(
                text.contains(&format!(
                    "    - first\n{}      span trace:\n          - {}::first_attachment",
                    location,
                    module_path!()
                )),
                "{text}"
            );
            assert!(text.contains("attempt=1"), "{text}");
            assert!(text.contains("id=42"), "{text}");
            assert!(!text.contains("::reporting"), "{text}");
            #[cfg(not(feature = "error_span_trace"))]
            assert!(!text.contains("::origin"), "{text}");
            for compact in [
                format!("{error}"),
                format!("{error:#}"),
                format!("{error:#?}"),
            ] {
                assert!(!compact.contains("span trace"), "{compact}");
            }
        });
    });
    #[cfg(feature = "json")]
    {
        let json = error.to_debug_json();
        assert_eq!(error.report().to_debug_json(), json);
        assert!(
            json["contexts"][0]["span_trace"]
                .as_str()
                .unwrap()
                .contains("::first_attachment")
        );
        assert!(
            json["contexts"][1]["span_trace"]
                .as_str()
                .unwrap()
                .contains("::second_attachment")
        );
        assert_eq!(
            json.get("span_trace").is_some(),
            cfg!(feature = "error_span_trace")
        );
        assert!(error.to_display_json().get("contexts").is_none());
    }
}

#[test]
fn direct_result_and_option_context_methods_capture_the_active_span() {
    let errors = with_traces(|| {
        tracing::info_span!("adapters").in_scope(|| {
            #[allow(unused_mut)]
            let mut errors = vec![
                root().context("eager").erase(),
                root().with_context(|| "lazy").erase(),
                Err::<(), _>(root()).context("eager").unwrap_err().erase(),
                Err::<(), _>(root())
                    .with_context(|| "lazy")
                    .unwrap_err()
                    .erase(),
                Err::<(), _>(fmt::Error)
                    .context("eager")
                    .unwrap_err()
                    .erase(),
                Err::<(), _>(fmt::Error)
                    .with_context(|| "lazy")
                    .unwrap_err()
                    .erase(),
                None::<()>.context("eager").unwrap_err().erase(),
                None::<()>.with_context(|| "lazy").unwrap_err().erase(),
            ];
            #[cfg(feature = "user_context")]
            errors.extend([
                root().user_context("eager").erase(),
                root().with_user_context(|| "lazy").erase(),
                Err::<(), _>(root())
                    .user_context("eager")
                    .unwrap_err()
                    .erase(),
                Err::<(), _>(root())
                    .with_user_context(|| "lazy")
                    .unwrap_err()
                    .erase(),
                Err::<(), _>(fmt::Error)
                    .user_context("eager")
                    .unwrap_err()
                    .erase(),
                Err::<(), _>(fmt::Error)
                    .with_user_context(|| "lazy")
                    .unwrap_err()
                    .erase(),
                None::<()>.user_context("eager").unwrap_err().erase(),
                None::<()>.with_user_context(|| "lazy").unwrap_err().erase(),
            ]);
            errors
        })
    });
    for error in errors {
        assert_eq!(error.contexts().len(), 1);
        assert_eq!(
            names(error.contexts().next().unwrap().span_trace()),
            ["adapters"]
        );
    }
}

#[test]
fn reshaping_mapping_recovery_and_adapters_preserve_context_traces() {
    let error =
        with_traces(|| tracing::info_span!("attachment").in_scope(|| root().context("saved")));
    let original = error.contexts().next().unwrap().span_trace().to_string();
    let assert_saved = |error: &ErrorUnion| {
        assert_eq!(
            error.contexts().next().unwrap().span_trace().to_string(),
            original
        );
    };
    with_traces(|| {
        tracing::info_span!("transform").in_scope(|| {
            let error: ErrorUnion<(fmt::Error, MsgError)> = error.widen();
            let error = error.narrow::<fmt::Error, _>().unwrap_err();
            let error = error.narrow::<(MsgError,), _>().unwrap();
            let error = error
                .map_single(|_| fmt::Error)
                .map_inner(|_| MsgError::from("replacement"));
            let error: ErrorUnion<(fmt::Error,)> = error.map_inner(|_| fmt::Error);
            let error = error.erase();
            assert_saved(&error);
            let error = error.narrow::<(fmt::Error,), _>().unwrap();
            let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(error.adapter());
            let error = boxed
                .downcast::<ErrorUnionAdapter<(fmt::Error,)>>()
                .unwrap()
                .union();
            let result: eros::Result<(), (fmt::Error, MsgError)> = Err(error.widen());
            let result = result.recover::<MsgError, _>(|_| panic!("must not handle this error"));
            let result: eros::Result<(), (fmt::Error,)> = result
                .try_recover(|error: ErrorUnion<(fmt::Error,)>| {
                    assert_saved(&error.erase());
                    Err::<(), _>(with_traces(|| {
                        tracing::info_span!("fallback")
                            .in_scope(|| root().context("fallback context"))
                    }))
                })
                .map_err(|error| error.map_single(|_| fmt::Error));
            assert_eq!(
                names(result.unwrap_err().contexts().next().unwrap().span_trace()),
                ["fallback"]
            );
        })
    });
}

#[test]
fn context_span_fields_distinguish_batch_groups_only_in_detailed_reports() {
    let batch = with_traces(|| {
        let children = [1, 2, 1].map(|id| {
            let error = root();
            tracing::info_span!("attachment", id).in_scope(|| error.context("same context"))
        });
        ErrorUnionVec::new(Vec::from(children))
    });
    let detailed = format!("{batch:?}");
    assert_eq!(
        detailed.matches("- error: failure").count(),
        2,
        "{detailed}"
    );
    assert!(detailed.contains("at: [0, 2]"), "{detailed}");
    assert!(detailed.contains("at: [1]"), "{detailed}");
    assert!(detailed.contains("with: id=1"), "{detailed}");
    assert!(detailed.contains("with: id=2"), "{detailed}");
    let compact = format!("{batch:#?}");
    assert_eq!(compact.matches("- error: failure").count(), 1, "{compact}");
    assert!(compact.contains("at: [0..3]"), "{compact}");
    assert!(!compact.contains("span trace"));
    #[cfg(feature = "json")]
    {
        let error: ErrorUnion<(ErrorUnionVec<(MsgError,)>,)> = batch.union();
        let json = error.to_debug_json();
        let children = json["children"].as_array().unwrap();
        assert_eq!(children.len(), 3);
        assert!(
            children[0]["contexts"][0]["span_trace"]
                .as_str()
                .unwrap()
                .contains("id=1")
        );
        assert!(
            children[1]["contexts"][0]["span_trace"]
                .as_str()
                .unwrap()
                .contains("id=2")
        );
    }
}

#[test]
fn uncaptured_context_traces_keep_their_status_and_serialize_as_null() {
    let empty = with_traces(|| root().context("empty"));
    let unsupported = tracing::subscriber::with_default(tracing_subscriber::registry(), || {
        tracing::info_span!("without_error_layer").in_scope(|| root().context("unsupported"))
    });
    for (error, status, text) in [
        (empty, SpanTraceStatus::EMPTY, "no active spans"),
        (unsupported, SpanTraceStatus::UNSUPPORTED, "unsupported"),
    ] {
        assert_eq!(
            error.contexts().next().unwrap().span_trace().status(),
            status
        );
        assert!(format!("{error:?}").contains(&format!("\n      span trace: {text}")));
        #[cfg(feature = "json")]
        for json in [error.to_debug_json(), error.report().to_debug_json()] {
            assert_eq!(
                json["contexts"][0].get("span_trace"),
                Some(&serde_json::Value::Null)
            );
        }
    }
}
