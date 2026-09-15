use alloc::{string::ToString, vec::Vec};
use serde_json::{Value, json};

use crate::{ErrorUnion, TypeSet, formatting::Report};

impl<E: TypeSet> ErrorUnion<E> {
    /// Returns the information in ordinary Display as structured JSON.
    ///
    /// The object contains `root` (a string) and `sources` (an array of strings
    /// in source-chain order). This creates data, without logging or serializing
    /// it to text. Requires the `diagnostic` feature; works with `no_std + alloc`.
    pub fn to_display_json(&self) -> Value {
        display_value(&Report::new(self))
    }

    /// Returns the information in ordinary Debug as structured JSON.
    ///
    /// Adds `contexts` in attachment order (innermost first), root/context `location`
    /// objects (`file`, `line`, `column`), and `backtrace` (`status`, `text`).
    /// Each context has `message` and `user_facing`. Without captured frames,
    /// backtrace text is null; status is `disabled`, `unsupported`, or
    /// `feature_disabled`. Captured frames use `captured` and a rendered string.
    /// Locations are omitted when the `location` feature is off.
    pub fn to_debug_json(&self) -> Value {
        let report = Report::new(self);
        let mut value = display_value(&report);
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
        value
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
