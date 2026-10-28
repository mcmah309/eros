use alloc::{string::ToString, vec::Vec};
use serde_json::{Value, json};

use crate::{ErrorReport, ErrorUnion, TypeSet, formatting::Report};

impl<E: TypeSet> ErrorUnion<E> {
    /// Returns root/source messages as structured JSON.
    ///
    /// The object contains `root` (a string) and `sources` (an array of strings
    /// in source-chain order). See [`crate::ErrorUnionVec`] for batch reporting.
    /// This creates data, without logging or serializing it to text. Requires
    /// the `json` feature; works with `no_std + alloc`.
    pub fn to_display_json(&self) -> Value {
        self.report().to_display_json()
    }

    /// Returns the information in ordinary Debug as structured JSON.
    ///
    /// Adds `contexts` in attachment order (innermost first), root/context `location`
    /// objects (`file`, `line`, `column`), and enabled traces.
    /// Each context has `message` and `user_facing`. With `backtrace`, adds
    /// `backtrace` as rendered text when captured, or null otherwise.
    /// Root locations require `error_location`; context locations require
    /// `context_location`. The `location` feature enables both.
    /// `error_spantrace` adds root `span_trace` text, and `context_spantrace`
    /// adds it to each context. Enabled traces are null when not captured;
    /// disabled fields are omitted. The `spantrace` feature enables both.
    /// See [`crate::ErrorUnionVec`] for batch reporting.
    pub fn to_debug_json(&self) -> Value {
        self.report().to_debug_json()
    }
}

impl ErrorReport<'_> {
    /// Returns root/source messages and any child reports as structured JSON.
    ///
    /// Uses the same schema as [`ErrorUnion::to_display_json`].
    pub fn to_display_json(&self) -> Value {
        let mut remaining = crate::formatting::MAX_CHILD_REPORTS;
        self.to_json(false, &mut remaining)
    }

    /// Returns this report and its children with their saved diagnostic metadata.
    ///
    /// Uses the same schema as [`ErrorUnion::to_debug_json`].
    pub fn to_debug_json(&self) -> Value {
        let mut remaining = crate::formatting::MAX_CHILD_REPORTS;
        self.to_json(true, &mut remaining)
    }

    fn to_json(&self, detailed: bool, remaining: &mut usize) -> Value {
        let report = Report::from_inner(self.inner);
        let mut value = display_value(&report);
        if detailed {
            self.add_debug_fields(&mut value, &report);
        }
        self.add_children(&mut value, detailed, remaining);
        value
    }

    fn add_debug_fields(&self, value: &mut Value, _report: &Report<'_>) {
        #[allow(unused_mut)]
        let mut contexts = Vec::<Value>::new();
        #[cfg(feature = "context")]
        for context in _report.contexts {
            #[allow(unused_mut)]
            let mut frame = json!({
                "message": context.context.to_string(),
                "user_facing": false,
            });
            #[cfg(feature = "user_context")]
            {
                frame["user_facing"] = context.is_user_context.into();
            }
            #[cfg(feature = "context_location")]
            {
                frame["location"] = location_value(context.location);
            }
            #[cfg(feature = "context_spantrace")]
            {
                frame["span_trace"] = crate::formatting::SpanTraceDisplay(&context.span_trace)
                    .text()
                    .into();
            }
            contexts.push(frame);
        }
        value["contexts"] = contexts.into();
        #[cfg(feature = "error_location")]
        {
            value["location"] = location_value(_report.location);
        }
        #[cfg(feature = "backtrace")]
        {
            value["backtrace"] = _report.backtrace_text().into();
        }
        #[cfg(feature = "error_spantrace")]
        {
            value["span_trace"] = _report.span_trace_text().into();
        }
    }

    fn add_children(&self, value: &mut Value, detailed: bool, remaining: &mut usize) {
        let mut children = Vec::new();
        let mut omitted = 0usize;
        self.visit_children(&mut |child| {
            if *remaining == 0 {
                omitted += 1;
                return;
            }
            *remaining -= 1;
            children.push(child.to_json(detailed, remaining));
        });
        if !children.is_empty() {
            value["children"] = children.into();
        }
        if omitted != 0 {
            value["children_omitted"] = omitted.into();
        }
    }
}

fn display_value(report: &Report<'_>) -> Value {
    json!({
        "root": report.root.to_string(),
        "sources": report.sources().map(ToString::to_string).collect::<Vec<_>>(),
    })
}

#[cfg(any(
    feature = "error_location",
    all(feature = "context", feature = "context_location")
))]
fn location_value(location: &core::panic::Location<'_>) -> Value {
    json!({ "file": location.file(), "line": location.line(), "column": location.column() })
}

#[cfg(all(test, feature = "backtrace"))]
mod tests {
    use super::*;
    use std::backtrace::{Backtrace, BacktraceStatus};

    #[test]
    fn uncaptured_backtrace_is_json_null_and_has_a_single_text_status_line() {
        let mut error: ErrorUnion = ErrorUnion::new(core::fmt::Error);
        error.inner.backtrace = Backtrace::disabled();
        for value in [error.to_debug_json(), error.report().to_debug_json()] {
            assert_eq!(value.get("backtrace"), Some(&Value::Null));
        }
        assert!(format!("{error:?}").ends_with("\nbacktrace: not captured"));
    }

    #[test]
    fn captured_backtrace_is_a_json_string_and_has_a_plain_text_header() {
        let mut error: ErrorUnion = ErrorUnion::new(core::fmt::Error);
        error.inner.backtrace = Backtrace::force_capture();
        assert_eq!(error.backtrace().status(), BacktraceStatus::Captured);
        for value in [error.to_debug_json(), error.report().to_debug_json()] {
            let text = value["backtrace"]
                .as_str()
                .expect("captured backtrace text");
            assert!(
                text.contains("captured_backtrace_is_a_json_string"),
                "{text}"
            );
        }
        let text = format!("{error:?}");
        assert!(text.contains("\nbacktrace:\n"), "{text}");
        assert!(!text.contains("(captured)"), "{text}");
    }
}
