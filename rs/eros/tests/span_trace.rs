#![cfg(feature = "error_span_trace")]

use eros::{ErrorUnion, ErrorUnionAdapter, ErrorUnionVec, IntoUnion, MsgError, ReshapeUnion};
use std::{fmt, future::Future, sync::Arc, task::Poll};
use tracing::Instrument;
use tracing_error::{ErrorLayer, SpanTrace, SpanTraceStatus};
use tracing_subscriber::prelude::*;

fn with_traces<T>(f: impl FnOnce() -> T) -> T {
    let subscriber = tracing_subscriber::registry().with(ErrorLayer::new(eros::SpanTraceFields));
    tracing::subscriber::with_default(subscriber, f)
}

fn names(trace: &SpanTrace) -> Vec<&'static str> {
    let mut names = Vec::new();
    trace.with_spans(|metadata, _fields| {
        names.push(metadata.name());
        true
    });
    names
}

#[tracing::instrument]
fn read_configuration() -> ErrorUnion<(MsgError,)> {
    ErrorUnion::new(MsgError::from("invalid configuration"))
}

fn captured_error() -> ErrorUnion<(MsgError,)> {
    with_traces(|| tracing::info_span!("request", request_id = 42).in_scope(read_configuration))
}

#[test]
fn captures_at_creation_and_retains_ancestry_after_leaving_the_subscriber() {
    let error = captured_error();
    assert_eq!(error.span_trace().status(), SpanTraceStatus::CAPTURED);
    assert_eq!(names(error.span_trace()), ["read_configuration", "request"]);
    assert!(error.span_trace().to_string().contains("request_id=42"));
    assert!(std::ptr::eq(
        error.span_trace(),
        error.report().span_trace()
    ));

    // Formatting in a different span must use the saved ancestry.
    with_traces(|| {
        tracing::info_span!("report_error").in_scope(|| {
            let report = format!("{error:?}");
            assert!(
                report.contains(&format!(
                    "\nspan trace:\n    - {}::read_configuration\n      location: ",
                    module_path!()
                )),
                "{report}"
            );
            assert!(
                report.contains(&format!(
                    "\n    - {}::request\n      with: request_id=42\n      location: ",
                    module_path!()
                )),
                "{report}"
            );
            assert!(!report.contains("::report_error"));
            assert_eq!(format!("{:?}", error.report()), report);
            assert_eq!(format!("{error:#?}"), "error: invalid configuration");
            assert_eq!(format!("{:#?}", error.report()), format!("{error:#?}"));
            assert_eq!(format!("{error}"), "invalid configuration");
            assert_eq!(format!("{error:#}"), "invalid configuration");
        });
    });
}

fn assert_field_layout(multiple_fields: &str, multiple_report_fields: &str) {
    let empty: ErrorUnion<(fmt::Error,)> =
        tracing::info_span!("empty", unset = tracing::field::Empty)
            .in_scope(|| ErrorUnion::new(fmt::Error));
    empty.span_trace().with_spans(|_, fields| {
        assert!(fields.is_empty(), "{fields}");
        true
    });
    assert!(!format!("{empty:?}").contains("with:"));

    let single: ErrorUnion<(fmt::Error,)> = tracing::info_span!(
        "single",
        path = "config file attempt=99",
        unset = tracing::field::Empty
    )
    .in_scope(|| ErrorUnion::new(fmt::Error));
    let report = format!("{single:?}");
    assert!(
        report.contains("\n      with: path=\"config file attempt=99\"\n      location: "),
        "{report}"
    );

    let multiple: ErrorUnion<(fmt::Error,)> = tracing::info_span!(
        "multiple",
        path = "config file attempt=99",
        attempt = 2,
        unset = tracing::field::Empty
    )
    .in_scope(|| ErrorUnion::new(fmt::Error));
    let report = format!("{multiple:?}");
    assert!(report.contains(multiple_report_fields), "{report}");
    assert_eq!(format!("{:?}", multiple.report()), report);
    assert_eq!(format!("{multiple:#?}"), format!("{empty:#?}"));
    multiple.span_trace().with_spans(|_, fields| {
        assert_eq!(fields, multiple_fields);
        true
    });
}

#[test]
fn eros_error_layer_separates_fields_without_splitting_values() {
    with_traces(|| {
        assert_field_layout(
            "path=\"config file attempt=99\"\nattempt=2",
            "\n      with:\n        path=\"config file attempt=99\"\n        attempt=2\n      location: ",
        );
    });
}

#[test]
fn default_error_layer_keeps_multiple_fields_inline() {
    let subscriber = tracing_subscriber::registry().with(ErrorLayer::default());
    tracing::subscriber::with_default(subscriber, || {
        assert_field_layout(
            "path=\"config file attempt=99\" attempt=2",
            "\n      with: path=\"config file attempt=99\" attempt=2\n      location: ",
        );
    });
}

#[test]
fn eros_error_layer_preserves_typed_and_formatted_values() {
    with_traces(|| {
        let error: ErrorUnion<(fmt::Error,)> = tracing::info_span!(
            "values",
            message = "quoted \"text\"\nnext line",
            count = 42_u64,
            enabled = true,
            ratio = 1.5_f64,
            display = %"two words attempt=99",
            debug = ?[1, 2],
        )
        .in_scope(|| ErrorUnion::new(fmt::Error));
        error.span_trace().with_spans(|_, fields| {
            assert_eq!(
                fields,
                "message=\"quoted \\\"text\\\"\\nnext line\"\ncount=42\nenabled=true\nratio=1.5\ndisplay=two words attempt=99\ndebug=[1, 2]"
            );
            true
        });
        let report = format!("{error:?}");
        assert!(
            report.contains(
                "\n        display=two words attempt=99\n        debug=[1, 2]\n      location: "
            ),
            "{report}"
        );
    });
}

#[test]
fn multiline_span_fields_align_under_with_heading() {
    use tracing_subscriber::{
        field::RecordFields,
        fmt::format::{FormatFields, Writer},
    };

    struct MultilineFields;
    impl<'writer> FormatFields<'writer> for MultilineFields {
        fn format_fields<R: RecordFields>(
            &self,
            mut writer: Writer<'writer>,
            _fields: R,
        ) -> fmt::Result {
            fmt::Write::write_str(&mut writer, "first=1\r\nsecond=2\n")
        }
    }

    let subscriber = tracing_subscriber::registry().with(ErrorLayer::new(MultilineFields));
    tracing::subscriber::with_default(subscriber, || {
        let error: ErrorUnion<(fmt::Error,)> =
            tracing::info_span!("multiline").in_scope(|| ErrorUnion::new(fmt::Error));
        let report = format!("{error:?}");
        assert!(
            report.contains(&format!(
                "\nspan trace:\n    - {}::multiline\n      with:\n        first=1\n        second=2\n      location: ",
                module_path!()
            )),
            "{report}"
        );
        assert!(!report.contains('\r'), "{report}");
        #[cfg(feature = "json")]
        {
            let diagnostic = error.to_debug_json();
            let trace = diagnostic["span_trace"].as_str().unwrap();
            assert!(
                trace.starts_with(&format!(
                    "- {}::multiline\n  with:\n    first=1\n    second=2\n  location: ",
                    module_path!()
                )),
                "{trace}"
            );
            assert!(!trace.ends_with('\n'));
        }
    });
}

#[test]
fn uncaptured_traces_have_single_status_lines() {
    let empty: ErrorUnion<(fmt::Error,)> = with_traces(|| ErrorUnion::new(fmt::Error));
    let unsupported: ErrorUnion<(fmt::Error,)> =
        tracing::subscriber::with_default(tracing_subscriber::registry(), || {
            tracing::info_span!("without_error_layer").in_scope(|| ErrorUnion::new(fmt::Error))
        });
    for (error, status, expected) in [
        (empty, SpanTraceStatus::EMPTY, "no active spans"),
        (unsupported, SpanTraceStatus::UNSUPPORTED, "unsupported"),
    ] {
        assert_eq!(error.span_trace().status(), status);
        let report = format!("{error:?}");
        assert!(
            report.contains(&format!("\nspan trace: {expected}")),
            "{report}"
        );
        assert_eq!(format!("{:?}", error.report()), report);
        assert!(!format!("{error:#?}").contains("span trace"));
        #[cfg(feature = "json")]
        for diagnostic in [error.to_debug_json(), error.report().to_debug_json()] {
            assert_eq!(diagnostic.get("span_trace"), Some(&serde_json::Value::Null));
        }
    }
}

#[test]
fn conversions_and_macros_capture_where_the_plain_error_enters_eros() {
    fn question_mark() -> eros::Result<(), (fmt::Error,)> {
        Err(fmt::Error)?;
        Ok(())
    }
    fn bail() -> eros::Result<()> {
        eros::bail!("failure")
    }
    fn ensure() -> eros::Result<()> {
        eros::ensure!(false, "failure");
        Ok(())
    }
    with_traces(|| {
        let plain = fmt::Error;
        tracing::info_span!("conversion").in_scope(|| {
            let result: eros::Result<(), (fmt::Error,)> = Err(plain).union();
            let errors = [
                result.unwrap_err().erase(),
                question_mark().unwrap_err().erase(),
                eros::error!("failure"),
                bail().unwrap_err(),
                ensure().unwrap_err(),
            ];
            for error in errors {
                assert_eq!(names(error.span_trace()), ["conversion"]);
            }
        });
    });
}

#[test]
fn reshaping_mapping_classification_and_adapters_preserve_the_original_trace() {
    with_traces(|| {
        let error = tracing::info_span!("request").in_scope(read_configuration);
        let original = error.span_trace().to_string();
        tracing::info_span!("transform").in_scope(|| {
            let error: ErrorUnion<(fmt::Error, MsgError)> = error.widen();
            assert_eq!(error.span_trace().to_string(), original);
            let error = error.narrow::<fmt::Error, _>().unwrap_err();
            assert_eq!(error.span_trace().to_string(), original);
            let error = error.narrow::<(MsgError,), _>().unwrap();
            let error = error.map_single(|_| fmt::Error);
            assert_eq!(error.span_trace().to_string(), original);
            let error = error.map_inner(|_| MsgError::from("replacement"));
            assert_eq!(error.span_trace().to_string(), original);
            let error: ErrorUnion<(fmt::Error,)> = error.classify(|_: eros::InnerError| fmt::Error);
            assert_eq!(error.span_trace().to_string(), original);
            let error = error.erase().narrow::<(fmt::Error,), _>().unwrap();
            let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(error.adapter());
            let error = boxed
                .downcast::<ErrorUnionAdapter<(fmt::Error,)>>()
                .unwrap()
                .union();
            assert_eq!(error.span_trace().to_string(), original);
        });
    });
}

#[test]
fn recovery_preserves_selected_and_remaining_traces_and_uses_the_fallbacks_own_trace() {
    let error = captured_error();
    let original = error.span_trace().to_string();
    let result: eros::Result<(), (fmt::Error, MsgError)> = Err(error.widen());
    let result = result.recover::<fmt::Error, _>(|_| panic!("must not handle this error"));
    assert_eq!(
        result.as_ref().unwrap_err().span_trace().to_string(),
        original
    );
    let result: eros::Result<(), (MsgError,)> =
        result.try_recover(|error: ErrorUnion<(MsgError,)>| {
            assert_eq!(error.span_trace().to_string(), original);
            Err(error)
        });
    let result: eros::Result<(), (MsgError,)> =
        result.try_recover(|error: ErrorUnion<(MsgError,)>| {
            assert_eq!(error.span_trace().to_string(), original);
            with_traces(|| {
                tracing::info_span!("fallback")
                    .in_scope(|| Err(ErrorUnion::new(MsgError::from("fallback failure"))))
            })
        });
    let result = result.recover(|error: ErrorUnion<(MsgError,)>| {
        assert_eq!(names(error.span_trace()), ["fallback"]);
    });
    result.into_value();
}

#[test]
fn instrumented_async_operations_capture_the_logical_ancestry_across_polls() {
    struct NoopWake;
    impl std::task::Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }
    with_traces(|| {
        let request = tracing::info_span!("async_request");
        let mut first_poll = true;
        let operation = async {
            std::future::poll_fn(|_| {
                if std::mem::take(&mut first_poll) {
                    Poll::Pending
                } else {
                    Poll::Ready(())
                }
            })
            .await;
            let error: ErrorUnion<(fmt::Error,)> = ErrorUnion::new(fmt::Error);
            error
        }
        .instrument(tracing::info_span!(parent: &request, "async_operation"));
        let mut operation = std::pin::pin!(operation);
        let waker = Arc::new(NoopWake).into();
        let mut context = std::task::Context::from_waker(&waker);
        assert!(operation.as_mut().poll(&mut context).is_pending());
        let Poll::Ready(error) = operation.as_mut().poll(&mut context) else {
            panic!("second poll must complete");
        };
        assert_eq!(
            names(error.span_trace()),
            ["async_operation", "async_request"]
        );
    });
}

#[test]
fn batch_grouping_distinguishes_span_fields_only_in_detailed_reports() {
    let error: ErrorUnion<(ErrorUnionVec<(MsgError,)>,)> = with_traces(|| {
        let children =
            [1, 2, 1].map(|id| tracing::info_span!("item", id).in_scope(read_configuration));
        tracing::info_span!("batch").in_scope(|| ErrorUnionVec::new(Vec::from(children)).union())
    });
    let detailed = format!("{error:?}");
    assert_eq!(
        detailed.matches("- error: invalid configuration").count(),
        2
    );
    assert!(detailed.contains("at: [0, 2]"), "{detailed}");
    assert!(detailed.contains("at: [1]"), "{detailed}");
    assert_eq!(detailed.matches("span trace:").count(), 3);
    assert!(
        detailed.contains(&format!(
            "\n          span trace:\n              - {}::read_configuration\n                location: ",
            module_path!()
        )),
        "{detailed}"
    );
    let compact = format!("{error:#?}");
    assert_eq!(compact.matches("- error: invalid configuration").count(), 1);
    assert!(compact.contains("at: [0..3]"), "{compact}");
    assert!(!compact.contains("span trace"));
    assert_eq!(names(error.span_trace()), ["batch"]);
    error.report().visit_children(&mut |child| {
        assert_eq!(names(child.span_trace()), ["read_configuration", "item"]);
    });
    #[cfg(feature = "json")]
    {
        let diagnostic = error.to_debug_json();
        assert_eq!(diagnostic["children"].as_array().unwrap().len(), 3);
        for (index, child) in error.as_single().iter().enumerate() {
            assert_eq!(
                diagnostic["children"][index]["span_trace"],
                child.to_debug_json()["span_trace"]
            );
        }
    }
}

#[cfg(feature = "json")]
#[test]
fn debug_json_contains_the_saved_trace_and_display_json_omits_it() {
    let error = captured_error();
    let diagnostic = error.to_debug_json();
    let trace = diagnostic["span_trace"]
        .as_str()
        .expect("captured span trace text");
    assert!(
        trace.starts_with(&format!(
            "- {}::read_configuration\n  location: ",
            module_path!()
        )),
        "{trace}"
    );
    assert!(
        trace.contains(&format!(
            "\n- {}::request\n  with: request_id=42\n  location: ",
            module_path!()
        )),
        "{trace}"
    );
    let indented = trace
        .lines()
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(format!("{error:?}").contains(&format!("\nspan trace:\n{indented}")));
    assert_eq!(error.report().to_debug_json(), diagnostic);
    assert!(error.to_display_json().get("span_trace").is_none());
    assert!(error.report().to_display_json().get("span_trace").is_none());
}

#[cfg(feature = "context")]
#[test]
fn attaching_context_does_not_recapture_the_trace() {
    let error = captured_error();
    let original = error.span_trace().to_string();
    let error = with_traces(|| {
        tracing::info_span!("attach_context").in_scope(|| {
            error.context("load settings").with_context(|| {
                Box::new(SpanTrace::capture()) as Box<dyn eros::SendSyncDebugDisplay>
            })
        })
    });
    assert_eq!(error.span_trace().to_string(), original);
    let value = error.contexts().last().unwrap().value().as_boxed().unwrap();
    let manual = (value as &dyn std::any::Any)
        .downcast_ref::<SpanTrace>()
        .unwrap();
    assert_eq!(names(manual), ["attach_context"]);
}

#[cfg(feature = "anyhow")]
#[test]
fn anyhow_imports_capture_the_conversion_span() {
    let original = anyhow::anyhow!("original failure");
    let shared = Arc::new(anyhow::anyhow!("shared failure"));
    let errors = with_traces(|| {
        tracing::info_span!("import_anyhow").in_scope(|| {
            [
                ErrorUnion::from_anyhow(original),
                ErrorUnion::from_anyhow_arc(shared),
            ]
        })
    });
    for error in errors {
        assert_eq!(names(error.span_trace()), ["import_anyhow"]);
    }
}
