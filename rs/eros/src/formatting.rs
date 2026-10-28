#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(any(feature = "backtrace", feature = "spantrace"))]
use alloc::string::ToString;
#[cfg(feature = "alloc")]
use alloc::{collections::BTreeMap, vec::Vec};
use core::{error::Error, fmt};

use crate::SendSyncError;
use crate::{ErrorUnion, TypeSet};

// Shared by human and JSON reporting. Charge before descending into a child so
// both wide batches and deeply nested batches have bounded child expansion.
#[cfg(feature = "alloc")]
pub(crate) const MAX_CHILD_REPORTS: usize = 10;

/// One view of the saved data, shared by human formatting and JSON diagnostics.
pub(crate) struct Report<'a> {
    #[cfg(feature = "alloc")]
    inner: &'a crate::storage::ErrorUnionInner,
    pub(crate) root: &'a dyn SendSyncError,
    #[cfg(feature = "context")]
    pub(crate) contexts: &'a [crate::context::ContextFrame],
    #[cfg(feature = "location")]
    pub(crate) location: &'static core::panic::Location<'static>,
    #[cfg(feature = "backtrace")]
    backtrace: &'a std::backtrace::Backtrace,
    #[cfg(feature = "spantrace")]
    span_trace: &'a tracing_error::SpanTrace,
}

impl<'a> Report<'a> {
    pub(crate) fn new<E: TypeSet>(error: &'a ErrorUnion<E>) -> Self {
        Self::from_inner(&error.inner)
    }

    pub(crate) fn from_inner(inner: &'a crate::storage::ErrorUnionInner) -> Self {
        let root = inner.error();
        #[cfg(feature = "backtrace")]
        let backtrace = &inner.backtrace;
        // Preserve the formatter's existing preference for an anyhow trace.
        #[cfg(all(feature = "anyhow", feature = "backtrace"))]
        let backtrace = {
            use crate::error_union::{AnyhowError, AnyhowErrorArc};
            let root = root as &dyn core::any::Any;
            let original = if let Some(error) = root.downcast_ref::<AnyhowError>() {
                Some(error.0.backtrace())
            } else {
                root.downcast_ref::<AnyhowErrorArc>()
                    .map(|error| error.0.backtrace())
            };
            original
                .filter(|trace| trace.status() == std::backtrace::BacktraceStatus::Captured)
                .unwrap_or(backtrace)
        };
        Self {
            #[cfg(feature = "alloc")]
            inner,
            root,
            #[cfg(feature = "context")]
            contexts: &inner.context,
            #[cfg(feature = "location")]
            location: inner.location,
            #[cfg(feature = "backtrace")]
            backtrace,
            #[cfg(feature = "spantrace")]
            span_trace: &inner.span_trace,
        }
    }

    pub(crate) fn sources(&self) -> impl Iterator<Item = &'a (dyn Error + 'static)> {
        core::iter::successors(self.root.source(), |error| (*error).source())
    }

    pub(crate) fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let renderer = ReportRenderer {
            indent: 0,
            detailed: false,
        };
        renderer.line(f, self.root)?;
        if !f.alternate() {
            renderer.sources(f, self)?;
        }
        Ok(())
    }

    pub(crate) fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let renderer = ReportRenderer {
            indent: 0,
            detailed: !f.alternate(),
        };
        #[cfg(feature = "alloc")]
        {
            let mut remaining = MAX_CHILD_REPORTS;
            let node = DebugNode::new(self.inner, renderer.detailed, &mut remaining)?;
            renderer.node(f, &node)
        }
        #[cfg(not(feature = "alloc"))]
        {
            renderer.error(f, self.root)?;
            renderer.detail_fields(f, self)
        }
    }

    #[cfg(feature = "backtrace")]
    pub(crate) fn backtrace_status(&self) -> &'static str {
        use std::backtrace::BacktraceStatus;
        match self.backtrace.status() {
            BacktraceStatus::Captured => "captured",
            BacktraceStatus::Disabled => "disabled",
            BacktraceStatus::Unsupported => "unsupported",
            _ => "unknown",
        }
    }

    #[cfg(feature = "backtrace")]
    pub(crate) fn backtrace_text(&self) -> Option<String> {
        if self.backtrace_status() == "captured" {
            #[cfg(feature = "better_backtrace")]
            if let Some(text) = better_backtrace(self.backtrace) {
                return Some(text.trim_end_matches('\n').into());
            }
            return Some(self.backtrace.to_string().trim_end_matches('\n').into());
        }
        None
    }

    #[cfg(feature = "spantrace")]
    pub(crate) fn span_trace_status(&self) -> &'static str {
        use tracing_error::SpanTraceStatus;
        match self.span_trace.status() {
            SpanTraceStatus::CAPTURED => "captured",
            SpanTraceStatus::EMPTY => "empty",
            SpanTraceStatus::UNSUPPORTED => "unsupported",
        }
    }

    #[cfg(feature = "spantrace")]
    pub(crate) fn span_trace_text(&self) -> Option<String> {
        (self.span_trace_status() == "captured")
            .then(|| SpanTraceDisplay(self.span_trace).to_string())
    }
}

#[cfg(feature = "spantrace")]
struct SpanTraceDisplay<'a>(&'a tracing_error::SpanTrace);

#[cfg(feature = "spantrace")]
impl fmt::Display for SpanTraceDisplay<'_> {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        let renderer = ReportRenderer {
            indent: 0,
            detailed: false,
        };
        let mut first = true;
        let mut result = Ok(());
        self.0.with_spans(|metadata, fields| {
            result = (|| {
                let name = format_args!("{}::{}", metadata.target(), metadata.name());
                if first {
                    renderer.bullet(out, &name)?;
                    first = false;
                } else {
                    renderer.next_bullet(out, &name)?;
                }
                let continued = renderer.continued();
                if !fields.is_empty() {
                    if fields.lines().nth(1).is_some() {
                        continued.next_line(out, &"with:")?;
                        continued.continued().next_line(out, &fields)?;
                    } else {
                        continued.next_line(out, &format_args!("with: {fields}"))?;
                    }
                }
                if let (Some(file), Some(line)) = (metadata.file(), metadata.line()) {
                    continued.next_line(out, &format_args!("location: {file}:{line}"))?;
                }
                Ok(())
            })();
            result.is_ok()
        });
        result
    }
}

/// Every report uses this layout, at its own nesting level. Nested reports do
/// not re-enter fmt::Debug or infer their depth from already formatted strings.
#[derive(Clone, Copy)]
struct ReportRenderer {
    indent: usize,
    detailed: bool,
}

impl ReportRenderer {
    // Structural nesting and text continuation have distinct indentation.
    fn nested(self) -> Self {
        Self {
            indent: self.indent + 4,
            ..self
        }
    }

    fn continued(self) -> Self {
        Self {
            indent: self.indent + 2,
            ..self
        }
    }

    fn line(
        self,
        out: &mut (impl fmt::Write + ?Sized),
        value: &(impl fmt::Display + ?Sized),
    ) -> fmt::Result {
        let indent = self.indent;
        write_lines(out, format_args!("{:indent$}", ""), indent, value)
    }

    fn next_line(
        self,
        out: &mut (impl fmt::Write + ?Sized),
        value: &(impl fmt::Display + ?Sized),
    ) -> fmt::Result {
        out.write_char('\n')?;
        self.line(out, value)
    }

    fn next_bullet(
        self,
        out: &mut (impl fmt::Write + ?Sized),
        value: &(impl fmt::Display + ?Sized),
    ) -> fmt::Result {
        out.write_char('\n')?;
        self.bullet(out, value)
    }

    fn bullet(
        self,
        out: &mut (impl fmt::Write + ?Sized),
        value: &(impl fmt::Display + ?Sized),
    ) -> fmt::Result {
        let indent = self.indent;
        write_lines(out, format_args!("{:indent$}- ", ""), indent + 2, value)
    }

    fn sources(self, out: &mut (impl fmt::Write + ?Sized), report: &Report<'_>) -> fmt::Result {
        self.nested().source_fields(out, report)
    }

    fn source_fields(
        self,
        out: &mut (impl fmt::Write + ?Sized),
        report: &Report<'_>,
    ) -> fmt::Result {
        let fields = self;
        let values = fields.nested();
        let mut sources = report.sources().peekable();
        if sources.peek().is_some() {
            fields.next_line(out, &"caused by:")?;
            for source in sources {
                values.next_bullet(out, source)?;
            }
        }
        Ok(())
    }

    fn error(
        self,
        out: &mut (impl fmt::Write + ?Sized),
        value: &(impl fmt::Display + ?Sized),
    ) -> fmt::Result {
        // Probe normalized lines without allocating, including when CRLF is
        // split across writes. A final newline does not make a multiline error.
        let multiline = {
            struct Probe(bool);
            impl fmt::Write for Probe {
                fn write_str(&mut self, text: &str) -> fmt::Result {
                    self.0 |= text.contains('\n');
                    Ok(())
                }
            }
            let mut probe = Probe(false);
            write_lines(&mut probe, format_args!(""), 0, value)?;
            probe.0
        };
        if multiline {
            self.line(out, &"error:")?;
            self.continued().next_line(out, value)
        } else {
            self.line(out, &format_args!("error: {value}"))
        }
    }

    fn detail_fields(
        self,
        out: &mut (impl fmt::Write + ?Sized),
        report: &Report<'_>,
    ) -> fmt::Result {
        self.source_fields(out, report)?;
        #[cfg(any(
            feature = "context",
            feature = "location",
            feature = "backtrace",
            feature = "spantrace"
        ))]
        let fields = self;
        #[cfg(feature = "context")]
        let values = fields.nested();
        #[cfg(feature = "context")]
        if !report.contexts.is_empty() {
            fields.next_line(out, &"context:")?;
            for context in report.contexts {
                values.next_bullet(out, &context.context)?;
                #[cfg(feature = "location")]
                if self.detailed {
                    values
                        .continued()
                        .next_line(out, &format_args!("location: {}", context.location))?;
                }
            }
        }
        if self.detailed {
            #[cfg(feature = "location")]
            fields.next_line(out, &format_args!("location: {}", report.location))?;
            #[cfg(feature = "spantrace")]
            fields.trace(
                out,
                "span trace",
                report.span_trace_status(),
                report.span_trace_text(),
            )?;
            #[cfg(feature = "backtrace")]
            fields.trace(
                out,
                "backtrace",
                report.backtrace_status(),
                report.backtrace_text(),
            )?;
        }
        Ok(())
    }

    #[cfg(any(feature = "backtrace", feature = "spantrace"))]
    fn trace(
        self,
        out: &mut (impl fmt::Write + ?Sized),
        name: &str,
        status: &str,
        text: Option<String>,
    ) -> fmt::Result {
        if let Some(text) = text {
            self.next_line(out, &format_args!("{name}:"))?;
            self.nested().next_line(out, &text)
        } else {
            let status = match status {
                "disabled" | "empty" => "not captured",
                status => status,
            };
            self.next_line(out, &format_args!("{name}: {status}"))
        }
    }

    #[cfg(feature = "alloc")]
    fn node(self, out: &mut (impl fmt::Write + ?Sized), node: &DebugNode<'_>) -> fmt::Result {
        if let Some(groups) = &node.children {
            let collection = if node.report.is_some() {
                self.line(out, &"error:")?;
                out.write_char('\n')?;
                self.nested()
            } else {
                self
            };
            collection.collection(out, groups)?;
        } else if let Some(report) = &node.report {
            self.error(out, report.root)?;
        }
        if let Some(report) = &node.report {
            self.detail_fields(out, report)?;
        }
        Ok(())
    }

    #[cfg(feature = "alloc")]
    fn collection(
        self,
        out: &mut (impl fmt::Write + ?Sized),
        children: &DebugChildren<'_>,
    ) -> fmt::Result {
        let count = children.total;
        let empty = if count == 0 { " []" } else { "" };
        self.line(out, &format_args!("errors ({count} total):{empty}"))?;
        self.groups(out, &children.groups)?;
        if children.omitted != 0 {
            self.nested().next_bullet(
                out,
                &format_args!(
                    "... {} errors omitted (report limit reached)",
                    children.omitted
                ),
            )?;
        }
        Ok(())
    }

    #[cfg(feature = "alloc")]
    fn groups(
        self,
        out: &mut (impl fmt::Write + ?Sized),
        groups: &[DebugGroup<'_>],
    ) -> fmt::Result {
        for group in groups {
            out.write_char('\n')?;
            self.nested().group(out, group)?;
        }
        Ok(())
    }

    #[cfg(feature = "alloc")]
    fn group(self, out: &mut (impl fmt::Write + ?Sized), group: &DebugGroup<'_>) -> fmt::Result {
        let fields = Self {
            indent: self.indent + 2,
            ..self
        };
        if let Some(children) = &group.node.children {
            self.line(out, &"- error:")?;
            out.write_char('\n')?;
            fields.nested().collection(out, children)?;
        } else {
            let mut message = String::new();
            if let Some(report) = &group.node.report {
                write_lines(&mut message, format_args!(""), 0, report.root)?;
            }
            if message.contains('\n') {
                self.line(out, &"- error:")?;
                fields.continued().next_line(out, &message)?;
            } else {
                self.line(out, &format_args!("- error: {message}"))?;
            }
        }
        fields.next_line(out, &format_args!("at: [{}]", Positions(&group.positions)))?;
        if let Some(report) = &group.node.report {
            fields.detail_fields(out, report)?;
        }
        Ok(())
    }
}

/// Prepared grouping borrows the saved reports. Preparing a child once avoids
/// recursively rebuilding its groups each time it is rendered at a new depth.
#[cfg(feature = "alloc")]
struct DebugNode<'a> {
    report: Option<Report<'a>>,
    children: Option<DebugChildren<'a>>,
}

#[cfg(feature = "alloc")]
struct DebugChildren<'a> {
    groups: Vec<DebugGroup<'a>>,
    total: usize,
    omitted: usize,
}

#[cfg(feature = "alloc")]
struct DebugGroup<'a> {
    positions: Vec<usize>,
    node: DebugNode<'a>,
}

/// Preserve field boundaries even when a message contains report-like text.
/// Only visible values participate, so alternate Debug still ignores metadata.
#[cfg(feature = "alloc")]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum GroupKeyPart {
    Root(String),
    Source(String),
    #[cfg(feature = "context")]
    Context(String),
    #[cfg(all(feature = "context", feature = "location"))]
    ContextLocation(String),
    #[cfg(feature = "location")]
    Location(String),
    #[cfg(feature = "backtrace")]
    Backtrace(&'static str, Option<String>),
    #[cfg(feature = "spantrace")]
    SpanTrace(&'static str, Option<String>),
    Children {
        total: usize,
        omitted: usize,
        groups: Vec<(Vec<usize>, Vec<GroupKeyPart>)>,
    },
}

#[cfg(feature = "alloc")]
fn message_key(value: &(impl fmt::Display + ?Sized)) -> Result<String, fmt::Error> {
    let mut message = String::new();
    write_lines(&mut message, format_args!(""), 0, value)?;
    Ok(message)
}

#[cfg(feature = "alloc")]
impl<'a> DebugNode<'a> {
    fn new(
        inner: &'a crate::storage::ErrorUnionInner,
        detailed: bool,
        remaining: &mut usize,
    ) -> Result<Self, fmt::Error> {
        let children = if inner.is_collection() {
            Some(group_reports(
                |visitor| inner.visit_children(visitor),
                detailed,
                remaining,
            )?)
        } else {
            None
        };
        Ok(Self {
            report: Some(Report::from_inner(inner)),
            children,
        })
    }

    fn grouping_key(&self, detailed: bool) -> Result<Vec<GroupKeyPart>, fmt::Error> {
        let mut key = Vec::new();
        if let Some(children) = &self.children {
            let groups = children
                .groups
                .iter()
                .map(|group| Ok((group.positions.clone(), group.node.grouping_key(detailed)?)))
                .collect::<Result<_, fmt::Error>>()?;
            key.push(GroupKeyPart::Children {
                total: children.total,
                omitted: children.omitted,
                groups,
            });
        }
        if let Some(report) = &self.report {
            if self.children.is_none() {
                key.push(GroupKeyPart::Root(message_key(report.root)?));
            }
            for source in report.sources() {
                key.push(GroupKeyPart::Source(message_key(source)?));
            }
            #[cfg(feature = "context")]
            for context in report.contexts {
                key.push(GroupKeyPart::Context(message_key(&context.context)?));
                #[cfg(feature = "location")]
                if detailed {
                    key.push(GroupKeyPart::ContextLocation(message_key(
                        context.location,
                    )?));
                }
            }
            if detailed {
                #[cfg(feature = "location")]
                key.push(GroupKeyPart::Location(message_key(report.location)?));
                #[cfg(feature = "backtrace")]
                key.push(GroupKeyPart::Backtrace(
                    report.backtrace_status(),
                    report.backtrace_text(),
                ));
                #[cfg(feature = "spantrace")]
                key.push(GroupKeyPart::SpanTrace(
                    report.span_trace_status(),
                    report.span_trace_text(),
                ));
            }
        }
        Ok(key)
    }
}

#[cfg(feature = "alloc")]
fn group_reports<'a>(
    visit: impl FnOnce(&mut dyn FnMut(crate::ErrorReport<'a>)),
    detailed: bool,
    remaining: &mut usize,
) -> Result<DebugChildren<'a>, fmt::Error> {
    let mut groups = Vec::<DebugGroup<'a>>::new();
    let mut lookup = BTreeMap::<Vec<GroupKeyPart>, usize>::new();
    let mut result = Ok(());
    let mut total = 0;
    let mut omitted = 0;
    visit(&mut |child| {
        let position = total;
        total += 1;
        if result.is_ok() {
            if *remaining == 0 {
                omitted += 1;
                return;
            }
            *remaining -= 1;
            result = (|| {
                let node = DebugNode::new(child.inner, detailed, remaining)?;
                let key = node.grouping_key(detailed)?;
                let index = *lookup.entry(key).or_insert(groups.len());
                if index == groups.len() {
                    groups.push(DebugGroup {
                        positions: Vec::new(),
                        node,
                    });
                }
                groups[index].positions.push(position);
                Ok(())
            })();
        }
    });
    result?;
    Ok(DebugChildren {
        groups,
        total,
        omitted,
    })
}

#[cfg(feature = "alloc")]
pub(crate) fn debug_vector<'a>(
    f: &mut fmt::Formatter<'_>,
    visit: impl FnOnce(&mut dyn FnMut(crate::ErrorReport<'a>)),
) -> fmt::Result {
    let renderer = ReportRenderer {
        indent: 0,
        detailed: !f.alternate(),
    };
    let mut remaining = MAX_CHILD_REPORTS;
    let node = DebugNode {
        report: None,
        children: Some(group_reports(visit, renderer.detailed, &mut remaining)?),
    };
    renderer.node(f, &node)
}

/// Zero-based vector positions, compacted into half-open Rust ranges.
#[cfg(feature = "alloc")]
struct Positions<'a>(&'a [usize]);

#[cfg(feature = "alloc")]
impl fmt::Display for Positions<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut positions = self.0.iter().copied().peekable();
        let mut first = true;
        while let Some(start) = positions.next() {
            let mut end = start;
            while positions.peek() == Some(&(end + 1)) {
                end = positions.next().unwrap();
            }
            if !first {
                f.write_str(", ")?;
            }
            first = false;
            write!(f, "{start}")?;
            if end != start {
                write!(f, "..{}", end + 1)?;
            }
        }
        Ok(())
    }
}

fn write_lines(
    f: &mut (impl fmt::Write + ?Sized),
    prefix: fmt::Arguments<'_>,
    continuation: usize,
    value: &(impl fmt::Display + ?Sized),
) -> fmt::Result {
    f.write_fmt(prefix)?;
    let mut writer = LineWriter {
        writer: f,
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

struct LineWriter<'a, W: fmt::Write + ?Sized> {
    writer: &'a mut W,
    continuation: usize,
    pending_newline: bool,
    pending_cr: bool,
}

impl<W: fmt::Write + ?Sized> LineWriter<'_, W> {
    fn write_content(&mut self, text: &str) -> fmt::Result {
        if self.pending_newline {
            self.writer.write_str("\n")?;
            for _ in 0..self.continuation {
                self.writer.write_str(" ")?;
            }
            self.pending_newline = false;
        }
        self.writer.write_str(text)
    }
}

impl<W: fmt::Write + ?Sized> fmt::Write for LineWriter<'_, W> {
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
    struct ErrorField<'a>(Fragmented<'a>);
    impl fmt::Display for ErrorField<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            ReportRenderer {
                indent: 0,
                detailed: false,
            }
            .error(f, &self.0)
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
            let expected_error = if text.lines().count() > 1 {
                let mut output = std::string::String::from("error:");
                for line in text.lines() {
                    output.push_str("\n  ");
                    output.push_str(line);
                }
                output
            } else {
                std::format!("error: {}", text.lines().next().unwrap_or_default())
            };
            for split in 0..=text.len() {
                if text.is_char_boundary(split) {
                    let actual = std::format!("{}", Indented(Fragmented { text, split }));
                    assert_eq!(actual, expected, "text={text:?}, split={split}");
                    let actual = std::format!("{}", ErrorField(Fragmented { text, split }));
                    assert_eq!(actual, expected_error, "text={text:?}, split={split}");
                }
            }
        }
    }
}
