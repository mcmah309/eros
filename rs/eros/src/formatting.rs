#[cfg(any(feature = "backtrace", feature = "diagnostic"))]
use alloc::string::String;
#[cfg(feature = "backtrace")]
use alloc::string::ToString;
use core::{error::Error, fmt};

use crate::SendSyncError;
use crate::{ErrorUnion, TypeSet};

/// One view of the saved data, shared by human formatting and JSON diagnostics.
pub(crate) struct Report<'a, T: SendSyncError + ?Sized = dyn SendSyncError> {
    pub(crate) root: &'a T,
    #[cfg(feature = "context")]
    pub(crate) contexts: &'a [crate::context::ContextFrame],
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
        #[cfg(feature = "context")] contexts: &'a [crate::context::ContextFrame],
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
        write_lines(f, format_args!(""), 0, self.root)?;
        #[cfg(feature = "location")]
        if detailed {
            write!(f, "\n  [{}]", self.location)?;
        }
        for source in self.sources() {
            write_lines(f, format_args!("\n  caused by: "), 13, source)?;
        }
        #[cfg(feature = "context")]
        if !self.contexts.is_empty() {
            f.write_str("\n\n  Context (innermost first):")?;
            for (index, context) in self.contexts.iter().enumerate() {
                f.write_str("\n")?;
                let number = index + 1;
                let continuation = 6 + number.ilog10() as usize + 1;
                write_lines(
                    f,
                    format_args!("    {number}. "),
                    continuation,
                    &context.context,
                )?;
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
            #[cfg(feature = "backtrace")]
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

    #[cfg(any(feature = "backtrace", feature = "diagnostic"))]
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
    prefix: fmt::Arguments<'_>,
    continuation: usize,
    value: &(impl fmt::Display + ?Sized),
) -> fmt::Result {
    f.write_fmt(prefix)?;
    let mut writer = LineWriter {
        formatter: f,
        continuation,
        pending_newline: false,
        pending_cr: false,
    };
    fmt::write(&mut writer, format_args!("{value}"))?;
    // A lone final CR is content. A trailing LF (including CRLF) is discarded,
    // matching str::lines(), even when the error writes in separate chunks.
    if writer.pending_cr {
        writer.write_content("\r")?;
    }
    Ok(())
}

struct LineWriter<'a, 'b> {
    formatter: &'a mut fmt::Formatter<'b>,
    continuation: usize,
    pending_newline: bool,
    pending_cr: bool,
}

impl LineWriter<'_, '_> {
    fn write_content(&mut self, text: &str) -> fmt::Result {
        if self.pending_newline {
            self.formatter.write_str("\n")?;
            for _ in 0..self.continuation {
                self.formatter.write_str(" ")?;
            }
            self.pending_newline = false;
        }
        self.formatter.write_str(text)
    }
}

impl fmt::Write for LineWriter<'_, '_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let mut start = 0;
        for (index, character) in text
            .char_indices()
            .filter(|(_, ch)| matches!(ch, '\r' | '\n'))
        {
            if start < index {
                if self.pending_cr {
                    self.pending_cr = false;
                    self.write_content("\r")?;
                }
                self.write_content(&text[start..index])?;
            }
            if self.pending_cr {
                self.pending_cr = false;
                if character != '\n' {
                    self.write_content("\r")?;
                }
            }
            match character {
                '\r' => self.pending_cr = true,
                '\n' => {
                    // Consecutive newlines preserve intermediate empty lines.
                    if self.pending_newline {
                        self.write_content("")?;
                    }
                    self.pending_newline = true;
                }
                _ => unreachable!(),
            }
            start = index + 1;
        }
        if start < text.len() {
            if self.pending_cr {
                self.pending_cr = false;
                self.write_content("\r")?;
            }
            self.write_content(&text[start..])?;
        }
        Ok(())
    }
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
    let mut text = String::from_utf8(output.into_inner()).ok()?;
    // Eros already prints its own backtrace heading.
    if let Some((banner, _)) = text.split_once('\n') {
        if banner.trim_matches('━').trim() == "BACKTRACE" {
            text.drain(..=banner.len());
        }
    }
    Some(text)
}

#[cfg(all(test, feature = "better_backtrace"))]
mod tests {
    use alloc::{format, string::String, vec::Vec};
    use std::backtrace::Backtrace;

    #[test]
    fn better_backtrace_removes_color_backtrace_banner() {
        // Exercise the formatter even when environment-controlled capture is disabled.
        let backtrace = Backtrace::force_capture();
        let trace = btparse::deserialize(&backtrace).expect("backtrace must deserialize");
        let mut output = color_backtrace::termcolor::NoColor::new(Vec::new());
        color_backtrace::BacktracePrinter::new()
            .print_trace(&trace, &mut output)
            .expect("color-backtrace must print the trace");
        let original = String::from_utf8(output.into_inner()).expect("trace must be UTF-8");
        let banner = format!("{:━^80}\n", " BACKTRACE ");
        assert!(
            original.starts_with(&banner),
            "color-backtrace's banner changed: {original}"
        );

        let formatted = super::better_backtrace(&backtrace)
            .expect("better_backtrace must format without falling back");
        assert!(!formatted.contains(" BACKTRACE "), "{formatted}");
        assert!(
            formatted
                .contains("formatting::tests::better_backtrace_removes_color_backtrace_banner"),
            "the application frame must remain: {formatted}"
        );
    }
}

#[cfg(test)]
mod streaming_tests {
    use super::*;

    struct Fragmented<'a> {
        text: &'a str,
        split: usize,
    }
    impl fmt::Display for Fragmented<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.text[..self.split])?;
            f.write_str("")?;
            f.write_str(&self.text[self.split..])
        }
    }
    struct Indented<'a>(Fragmented<'a>);
    impl fmt::Display for Indented<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write_lines(f, format_args!("prefix: "), 8, &self.0)
        }
    }

    #[test]
    fn multiline_output_matches_lines_for_every_chunk_boundary() {
        for text in [
            "",
            "\n",
            "\r",
            "\r\n",
            "\n\n",
            "a\n",
            "a\r\n",
            "a\rb\r",
            "a\n\nb",
            "\nfirst\nlast\n",
            "é\r\n🦀\nend",
        ] {
            let mut lines = text.lines();
            let mut expected = std::format!("prefix: {}", lines.next().unwrap_or_default());
            for line in lines {
                expected.push_str("\n        ");
                expected.push_str(line);
            }
            for split in 0..=text.len() {
                if text.is_char_boundary(split) {
                    let actual = std::format!("{}", Indented(Fragmented { text, split }));
                    assert_eq!(actual, expected, "text={text:?}, split={split}");
                }
            }
        }
    }
}
