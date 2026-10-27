use alloc::{string::ToString, vec::Vec};
use serde_json::{Value, json};

use crate::{ErrorReport, ErrorUnion, TypeSet, formatting::Report};

impl<E: TypeSet> ErrorUnion<E> {
    /// Returns root/source messages and registered child reports as structured JSON.
    ///
    /// The object contains `root` (a string) and `sources` (an array of strings
    /// in source-chain order). Registered children appear recursively in a
    /// `children` array, omitted when empty. At most 10 children are expanded
    /// across the whole report, in depth-first insertion order. Truncated batches
    /// include `children_omitted`, the number of immediate children skipped.
    /// This creates data, without logging or serializing it to text. Requires
    /// the `diagnostic` feature; works with `no_std + alloc`.
    pub fn to_display_json(&self) -> Value {
        self.report().to_display_json()
    }

    /// Returns the information in ordinary Debug as structured JSON.
    ///
    /// Adds `contexts` in attachment order (innermost first), root/context `location`
    /// objects (`file`, `line`, `column`), and `backtrace` (`status`, `text`).
    /// Each context has `message` and `user_facing`. Without captured frames,
    /// backtrace text is null; status is `disabled`, `unsupported`, or
    /// `feature_disabled`. Captured frames use `captured` and a rendered string.
    /// Locations are omitted when the `location` feature is off.
    /// Registered children appear recursively in `children`, with their own
    /// metadata. The array is omitted when empty. The same child limit and
    /// `children_omitted` field as [`Self::to_display_json`] apply.
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

    fn add_debug_fields(&self, value: &mut Value, report: &Report<'_>) {
        #[allow(unused_mut)]
        let mut contexts = Vec::<Value>::new();
        #[cfg(feature = "context")]
        for context in report.contexts {
            #[allow(unused_mut)]
            let mut frame = json!({
                "message": context.context.to_string(),
                "user_facing": false,
            });
            #[cfg(feature = "user_context")]
            {
                frame["user_facing"] = context.is_user_facing.into();
            }
            #[cfg(feature = "location")]
            {
                frame["location"] = location_value(context.location);
            }
            contexts.push(frame);
        }
        value["contexts"] = contexts.into();
        #[cfg(feature = "location")]
        {
            value["location"] = location_value(report.location);
        }
        value["backtrace"] = json!({
            "status": report.backtrace_status(),
            "text": report.backtrace_text(),
        });
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

#[cfg(feature = "location")]
fn location_value(location: &core::panic::Location<'_>) -> Value {
    json!({ "file": location.file(), "line": location.line(), "column": location.column() })
}
