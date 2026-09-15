#[cfg(feature = "backtrace")]
use alloc::string::ToString;
use alloc::{format, string::String};
use core::{error::Error, fmt};

use crate::SendSyncError;
use crate::{ErrorUnion, TypeSet};

/// One view of the saved data, shared by human formatting and JSON diagnostics.
pub(crate) struct Report<'a, T: SendSyncError + ?Sized = dyn SendSyncError> {
    pub(crate) root: &'a T,
    #[cfg(feature = "context")]
    pub(crate) contexts: &'a [crate::context::ErosContext],
    #[cfg(feature = "location")]
    pub(crate) location: &'static core::panic::Location<'static>,
    #[cfg(feature = "backtrace")]
    backtrace: &'a std::backtrace::Backtrace,
}

impl<'a> Report<'a> {
    pub(crate) fn new<E: TypeSet>(error: &'a ErrorUnion<E>) -> Self {
        Self::from_parts(
            error.inner(),
            #[cfg(feature = "context")]
            &error.inner.context,
            #[cfg(feature = "location")]
            error.inner.location,
            #[cfg(feature = "backtrace")]
            &error.inner.backtrace,
        )
    }
}

impl<'a, T: SendSyncError + ?Sized> Report<'a, T> {
    pub(crate) fn from_parts(
        root: &'a T,
        #[cfg(feature = "context")] contexts: &'a [crate::context::ErosContext],
        #[cfg(feature = "location")] location: &'static core::panic::Location<'static>,
        #[cfg(feature = "backtrace")] backtrace: &'a std::backtrace::Backtrace,
    ) -> Self {
        // Preserve the formatter's existing preference for an anyhow trace.
        #[cfg(all(feature = "anyhow", feature = "backtrace"))]
        let backtrace = {
            use crate::error_union::{AnyhowError, AnyhowErrorArc};
            let original = if let Some(error) = root.as_any().downcast_ref::<AnyhowError>() {
                Some(error.0.backtrace())
            } else {
                root.as_any()
                    .downcast_ref::<AnyhowErrorArc>()
                    .map(|error| error.0.backtrace())
            };
            original
                .filter(|trace| trace.status() == std::backtrace::BacktraceStatus::Captured)
                .unwrap_or(backtrace)
        };
        Self {
            root,
            #[cfg(feature = "context")]
            contexts,
            #[cfg(feature = "location")]
            location,
            #[cfg(feature = "backtrace")]
            backtrace,
        }
    }

    pub(crate) fn sources(&self) -> impl Iterator<Item = &'a (dyn Error + 'static)> {
        core::iter::successors(self.root.source(), |error| (*error).source())
    }

    pub(crate) fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.root)?;
        if !f.alternate() {
            for source in self.sources() {
                write!(f, " <- {source}")?;
            }
        }
        Ok(())
    }

    pub(crate) fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let detailed = !f.alternate();
        write_lines(f, "", "", &format!("{}", self.root))?;
        #[cfg(feature = "location")]
        if detailed {
            write!(f, "\n  [{}]", self.location)?;
        }
        for source in self.sources() {
            write_lines(f, "\n  caused by: ", "             ", &format!("{source}"))?;
        }
        #[cfg(feature = "context")]
        if !self.contexts.is_empty() {
            f.write_str("\n\n  Context (innermost first):")?;
            for (index, context) in self.contexts.iter().enumerate() {
                let prefix = format!("    {}. ", index + 1);
                let continuation = " ".repeat(prefix.len());
                f.write_str("\n")?;
                write_lines(f, &prefix, &continuation, &format!("{}", context.context))?;
                #[cfg(feature = "location")]
                if detailed {
                    write!(f, "\n    [{}]", context.location)?;
                }
            }
        }
        if detailed {
            let status = self.backtrace_status();
            let label = if status == "feature_disabled" {
                "feature disabled"
            } else {
                status
            };
            write!(f, "\n\nBacktrace ({label}):")?;
            if let Some(text) = self.backtrace_text() {
                write!(f, "\n{text}")?;
            }
        }
        Ok(())
    }

    pub(crate) fn backtrace_status(&self) -> &'static str {
        #[cfg(feature = "backtrace")]
        {
            use std::backtrace::BacktraceStatus;
            match self.backtrace.status() {
                BacktraceStatus::Captured => "captured",
                BacktraceStatus::Disabled => "disabled",
                BacktraceStatus::Unsupported => "unsupported",
                _ => "unknown",
            }
        }
        #[cfg(not(feature = "backtrace"))]
        "feature_disabled"
    }

    pub(crate) fn backtrace_text(&self) -> Option<String> {
        #[cfg(feature = "backtrace")]
        if self.backtrace_status() == "captured" {
            #[cfg(feature = "better_backtrace")]
            if let Some(text) = better_backtrace(self.backtrace) {
                return Some(text.trim_end_matches('\n').into());
            }
            return Some(self.backtrace.to_string().trim_end_matches('\n').into());
        }
        None
    }
}

fn write_lines(
    f: &mut fmt::Formatter<'_>,
    prefix: &str,
    continuation: &str,
    text: &str,
) -> fmt::Result {
    let mut lines = text.lines();
    write!(f, "{prefix}{}", lines.next().unwrap_or_default())?;
    for line in lines {
        write!(f, "\n{continuation}{line}")?;
    }
    Ok(())
}

#[cfg(feature = "better_backtrace")]
fn better_backtrace(backtrace: &std::backtrace::Backtrace) -> Option<String> {
    use alloc::{boxed::Box, vec::Vec};
    let trace = btparse::deserialize(backtrace).ok()?;
    let printer = color_backtrace::BacktracePrinter::new().add_frame_filter(Box::new(
        |frames: &mut Vec<&color_backtrace::Frame>| {
            frames.retain(|frame| {
                !(frame.is_dependency_code()
                    || frame.is_post_panic_code()
                    || frame.is_runtime_init_code())
            });
        },
    ));
    // Formatting an error must not inject terminal escapes into tracing/JSON.
    let mut output = color_backtrace::termcolor::NoColor::new(Vec::new());
    printer.print_trace(&trace, &mut output).ok()?;
    String::from_utf8(output.into_inner()).ok()
}
