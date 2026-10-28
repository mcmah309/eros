use alloc::{string::String, vec::Vec};
use core::fmt;
use serde_json::{Value, json};

use crate::{ErrorReport, ErrorUnion, TypeSet, formatting::Report};

impl<E: TypeSet> ErrorUnion<E> {
    /// Returns a structured JSON report without diagnostics.
    ///
    /// See the [Structured JSON documentation](https://github.com/mcmah309/eros/blob/master/docs/json.md).
    pub fn to_display_json(&self) -> Value {
        self.report().to_display_json()
    }

    /// Returns a structured JSON report with diagnostics.
    ///
    /// See the [Structured JSON documentation](https://github.com/mcmah309/eros/blob/master/docs/json.md).
    pub fn to_debug_json(&self) -> Value {
        self.report().to_debug_json()
    }
}

impl ErrorReport<'_> {
    /// Returns a structured JSON report without diagnostics.
    ///
    /// See the [Structured JSON documentation](https://github.com/mcmah309/eros/blob/master/docs/json.md).
    pub fn to_display_json(&self) -> Value {
        let mut remaining = crate::formatting::MAX_CHILD_REPORTS;
        self.to_json(false, &mut remaining)
    }

    /// Returns a structured JSON report with diagnostics.
    ///
    /// See the [Structured JSON documentation](https://github.com/mcmah309/eros/blob/master/docs/json.md).
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
            let Some(message) = message_text(&context.context) else {
                continue;
            };
            #[allow(unused_mut)]
            let mut frame = json!({
                "message": message,
            });
            #[cfg(feature = "user_context")]
            {
                frame["user_context"] = context.is_user_context.into();
            }
            #[cfg(feature = "context_location")]
            {
                frame["location"] = location_value(context.location);
            }
            #[cfg(feature = "context_span_trace")]
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
        #[cfg(feature = "error_span_trace")]
        {
            value["span_trace"] = _report.span_trace_text().into();
        }
    }

    fn add_children(&self, value: &mut Value, detailed: bool, remaining: &mut usize) {
        let mut children = Vec::new();
        let total = self.inner.visit_children_while(*remaining, &mut |child| {
            *remaining -= 1;
            children.push(child.to_json(detailed, remaining));
            *remaining != 0
        });
        let omitted = total - children.len();
        if !children.is_empty() {
            value["children"] = children.into();
        }
        if omitted != 0 {
            value["children_omitted"] = omitted.into();
        }
    }
}

fn display_value(report: &Report<'_>) -> Value {
    let root = message_text(report.root);
    let sources: Vec<_> = report.sources().filter_map(message_text).collect();
    let mut value = json!({ "sources": sources });
    if let Some(root) = root {
        value["root"] = root.into();
    }
    value
}

fn message_text(value: &(impl fmt::Display + ?Sized)) -> Option<String> {
    crate::formatting::message_text(value).ok()
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
        error.inner.diagnostics_mut().backtrace = Backtrace::disabled();
        for value in [error.to_debug_json(), error.report().to_debug_json()] {
            assert_eq!(value.get("backtrace"), Some(&Value::Null));
        }
        assert!(format!("{error:?}").ends_with("\nbacktrace: not captured"));
    }

    #[test]
    fn captured_backtrace_is_a_json_string_and_has_a_plain_text_header() {
        let mut error: ErrorUnion = ErrorUnion::new(core::fmt::Error);
        error.inner.diagnostics_mut().backtrace = Backtrace::force_capture();
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
