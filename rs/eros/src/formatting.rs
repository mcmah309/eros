#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(any(
    feature = "backtrace",
    feature = "error_span_trace",
    all(feature = "context", feature = "context_span_trace")
))]
use alloc::string::ToString;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
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
    #[cfg(feature = "error_location")]
    pub(crate) location: &'static core::panic::Location<'static>,
    #[cfg(feature = "backtrace")]
    backtrace: &'a std::backtrace::Backtrace,
    #[cfg(feature = "error_span_trace")]
    span_trace: &'a tracing_error::SpanTrace,
}

impl<'a> Report<'a> {
    pub(crate) fn new<E: TypeSet>(error: &'a ErrorUnion<E>) -> Self {
        Self::from_inner(&error.inner)
    }

    pub(crate) fn from_inner(inner: &'a crate::storage::ErrorUnionInner) -> Self {
        let root = inner.error();
        #[cfg(feature = "backtrace")]
        let backtrace = &inner.diagnostics().backtrace;
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
            contexts: &inner.diagnostics().context,
            #[cfg(feature = "error_location")]
            location: inner.diagnostics().location,
            #[cfg(feature = "backtrace")]
            backtrace,
            #[cfg(feature = "error_span_trace")]
            span_trace: &inner.diagnostics().span_trace,
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
            // Standalone errors do not need grouping or allocated preparation.
            if !self.inner.is_collection() {
                renderer.error(f, self.root)?;
                return renderer.detail_fields(f, self);
            }
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
            #[cfg(feature = "filtered_backtrace")]
            if let Some(text) = filtered_backtrace(self.backtrace) {
                return Some(text.trim_end_matches('\n').into());
            }
            return Some(self.backtrace.to_string().trim_end_matches('\n').into());
        }
        None
    }

    #[cfg(feature = "error_span_trace")]
    pub(crate) fn span_trace_status(&self) -> &'static str {
        SpanTraceDisplay(self.span_trace).status()
    }

    #[cfg(feature = "error_span_trace")]
    pub(crate) fn span_trace_text(&self) -> Option<String> {
        SpanTraceDisplay(self.span_trace).text()
    }
}

#[cfg(any(
    feature = "error_span_trace",
    all(feature = "context", feature = "context_span_trace")
))]
pub(crate) struct SpanTraceDisplay<'a>(pub(crate) &'a tracing_error::SpanTrace);

#[cfg(any(
    feature = "error_span_trace",
    all(feature = "context", feature = "context_span_trace")
))]
impl SpanTraceDisplay<'_> {
    pub(crate) fn status(&self) -> &'static str {
        use tracing_error::SpanTraceStatus;
        match self.0.status() {
            SpanTraceStatus::CAPTURED => "captured",
            SpanTraceStatus::EMPTY => "empty",
            SpanTraceStatus::UNSUPPORTED => "unsupported",
        }
    }

    pub(crate) fn text(&self) -> Option<String> {
        (self.status() == "captured").then(|| self.to_string())
    }
}

#[cfg(any(
    feature = "error_span_trace",
    all(feature = "context", feature = "context_span_trace")
))]
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
            feature = "error_location",
            feature = "backtrace",
            feature = "error_span_trace"
        ))]
        let fields = self;
        #[cfg(feature = "context")]
        let values = fields.nested();
        #[cfg(feature = "context")]
        if !report.contexts.is_empty() {
            fields.next_line(out, &"context:")?;
            for context in report.contexts {
                values.next_bullet(out, &context.context)?;
                #[cfg(feature = "context_location")]
                if self.detailed {
                    values
                        .continued()
                        .next_line(out, &format_args!("location: {}", context.location))?;
                }
                #[cfg(feature = "context_span_trace")]
                if self.detailed {
                    let trace = SpanTraceDisplay(&context.span_trace);
                    values.continued().trace(
                        out,
                        "span trace",
                        trace.status(),
                        trace.text().as_deref(),
                    )?;
                }
            }
        }
        if self.detailed {
            #[cfg(feature = "error_location")]
            fields.next_line(out, &format_args!("location: {}", report.location))?;
            #[cfg(feature = "error_span_trace")]
            fields.trace(
                out,
                "span trace",
                report.span_trace_status(),
                report.span_trace_text().as_deref(),
            )?;
            #[cfg(feature = "backtrace")]
            fields.trace(
                out,
                "backtrace",
                report.backtrace_status(),
                report.backtrace_text().as_deref(),
            )?;
        }
        Ok(())
    }

    #[cfg(any(
        feature = "backtrace",
        feature = "error_span_trace",
        all(feature = "context", feature = "context_span_trace")
    ))]
    fn trace(
        self,
        out: &mut (impl fmt::Write + ?Sized),
        name: &str,
        status: &str,
        text: Option<&str>,
    ) -> fmt::Result {
        if let Some(text) = text {
            self.next_line(out, &format_args!("{name}:"))?;
            self.nested().next_line(out, &text)
        } else {
            let status = match status {
                "disabled" => "not captured",
                "empty" => "no active spans",
                status => status,
            };
            self.next_line(out, &format_args!("{name}: {status}"))
        }
    }

    // message_text already normalized line endings and removed one final LF.
    // Write its lines directly so intentional trailing blank lines survive.
    #[cfg(feature = "alloc")]
    fn prepared_line(
        self,
        out: &mut (impl fmt::Write + ?Sized),
        text: &str,
        bullet: bool,
    ) -> fmt::Result {
        write!(out, "{:indent$}", "", indent = self.indent)?;
        if bullet {
            out.write_str("- ")?;
        }
        let continuation = self.indent + if bullet { 2 } else { 0 };
        let mut lines = text.split('\n');
        out.write_str(lines.next().unwrap_or_default())?;
        for line in lines {
            write!(out, "\n{:continuation$}{line}", "")?;
        }
        Ok(())
    }

    #[cfg(feature = "alloc")]
    fn prepared_error(
        self,
        out: &mut (impl fmt::Write + ?Sized),
        text: &str,
        bullet: bool,
    ) -> fmt::Result {
        if text.contains('\n') {
            self.line(out, &if bullet { "- error:" } else { "error:" })?;
            out.write_char('\n')?;
            let message = if bullet {
                self.nested()
            } else {
                self.continued()
            };
            message.prepared_line(out, text, false)
        } else {
            let prefix = if bullet { "- error: " } else { "error: " };
            write!(out, "{:indent$}{prefix}{text}", "", indent = self.indent)
        }
    }

    #[cfg(feature = "alloc")]
    fn prepared_fields(
        self,
        out: &mut (impl fmt::Write + ?Sized),
        fields: &[DebugField],
    ) -> fmt::Result {
        let mut sources = false;
        #[cfg(feature = "context")]
        let mut contexts = false;
        for field in fields {
            match field {
                DebugField::Source(text) => {
                    if !sources {
                        self.next_line(out, &"caused by:")?;
                        sources = true;
                    }
                    out.write_char('\n')?;
                    self.nested().prepared_line(out, text, true)?;
                }
                #[cfg(feature = "context")]
                DebugField::Context(text) => {
                    if !contexts {
                        self.next_line(out, &"context:")?;
                        contexts = true;
                    }
                    out.write_char('\n')?;
                    self.nested().prepared_line(out, text, true)?;
                }
                #[cfg(all(feature = "context", feature = "context_location"))]
                DebugField::ContextLocation(text) => {
                    self.nested()
                        .continued()
                        .next_line(out, &format_args!("location: {text}"))?;
                }
                #[cfg(all(feature = "context", feature = "context_span_trace"))]
                DebugField::ContextSpanTrace(status, text) => {
                    self.nested()
                        .continued()
                        .trace(out, "span trace", status, text.as_deref())?;
                }
                #[cfg(feature = "error_location")]
                DebugField::Location(text) => {
                    self.next_line(out, &format_args!("location: {text}"))?
                }
                #[cfg(feature = "backtrace")]
                DebugField::Backtrace(status, text) => {
                    self.trace(out, "backtrace", status, text.as_deref())?
                }
                #[cfg(feature = "error_span_trace")]
                DebugField::SpanTrace(status, text) => {
                    self.trace(out, "span trace", status, text.as_deref())?
                }
            }
        }
        Ok(())
    }

    #[cfg(feature = "alloc")]
    fn node(self, out: &mut (impl fmt::Write + ?Sized), node: &DebugNode) -> fmt::Result {
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
            self.prepared_error(out, report.root.as_ref().expect("leaf message"), false)?;
        }
        if let Some(report) = &node.report {
            self.prepared_fields(out, &report.fields)?;
        }
        Ok(())
    }

    #[cfg(feature = "alloc")]
    fn collection(
        self,
        out: &mut (impl fmt::Write + ?Sized),
        children: &DebugChildren,
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
    fn groups(self, out: &mut (impl fmt::Write + ?Sized), groups: &[DebugGroup]) -> fmt::Result {
        for group in groups {
            out.write_char('\n')?;
            self.nested().group(out, group)?;
        }
        Ok(())
    }

    #[cfg(feature = "alloc")]
    fn group(self, out: &mut (impl fmt::Write + ?Sized), group: &DebugGroup) -> fmt::Result {
        let fields = Self {
            indent: self.indent + 2,
            ..self
        };
        if let Some(children) = &group.node.children {
            self.line(out, &"- error:")?;
            out.write_char('\n')?;
            fields.nested().collection(out, children)?;
        } else if let Some(report) = &group.node.report {
            self.prepared_error(out, report.root.as_ref().expect("leaf message"), true)?;
        }
        fields.next_line(out, &format_args!("at: [{}]", Positions(&group.positions)))?;
        if let Some(report) = &group.node.report {
            fields.prepared_fields(out, &report.fields)?;
        }
        Ok(())
    }
}

// Prepared text is both the grouping key and the rendered data. Comparing at
// most ten nodes avoids cloning recursive keys or formatting diagnostics again.
#[cfg(feature = "alloc")]
#[derive(PartialEq, Eq)]
struct DebugNode {
    report: Option<PreparedReport>,
    children: Option<DebugChildren>,
}

#[cfg(feature = "alloc")]
#[derive(PartialEq, Eq)]
struct DebugChildren {
    groups: Vec<DebugGroup>,
    total: usize,
    omitted: usize,
}

#[cfg(feature = "alloc")]
#[derive(PartialEq, Eq)]
struct DebugGroup {
    positions: Vec<usize>,
    node: DebugNode,
}

#[cfg(feature = "alloc")]
#[derive(PartialEq, Eq)]
struct PreparedReport {
    root: Option<String>,
    fields: Vec<DebugField>,
}

// Preserve field boundaries even when a message contains report-like text.
// Only visible values participate, so alternate Debug still ignores metadata.
#[cfg(feature = "alloc")]
#[derive(PartialEq, Eq)]
enum DebugField {
    Source(String),
    #[cfg(feature = "context")]
    Context(String),
    #[cfg(all(feature = "context", feature = "context_location"))]
    ContextLocation(String),
    #[cfg(all(feature = "context", feature = "context_span_trace"))]
    ContextSpanTrace(&'static str, Option<String>),
    #[cfg(feature = "error_location")]
    Location(String),
    #[cfg(feature = "backtrace")]
    Backtrace(&'static str, Option<String>),
    #[cfg(feature = "error_span_trace")]
    SpanTrace(&'static str, Option<String>),
}

#[cfg(feature = "alloc")]
pub(crate) fn message_text(value: &(impl fmt::Display + ?Sized)) -> Result<String, fmt::Error> {
    let mut message = String::new();
    write_lines(&mut message, format_args!(""), 0, value)?;
    Ok(message)
}

#[cfg(feature = "alloc")]
impl DebugNode {
    fn new(
        inner: &crate::storage::ErrorUnionInner,
        detailed: bool,
        remaining: &mut usize,
    ) -> Result<Self, fmt::Error> {
        let children = if inner.is_collection() {
            Some(group_reports(
                |limit, visitor| inner.visit_children_while(limit, visitor),
                detailed,
                remaining,
            )?)
        } else {
            None
        };
        let report = Report::from_inner(inner);
        let root = if children.is_none() {
            Some(message_text(report.root)?)
        } else {
            None
        };
        let mut fields = Vec::new();
        for source in report.sources() {
            fields.push(DebugField::Source(message_text(source)?));
        }
        #[cfg(feature = "context")]
        for context in report.contexts {
            fields.push(DebugField::Context(message_text(&context.context)?));
            #[cfg(feature = "context_location")]
            if detailed {
                fields.push(DebugField::ContextLocation(message_text(context.location)?));
            }
            #[cfg(feature = "context_span_trace")]
            if detailed {
                let trace = SpanTraceDisplay(&context.span_trace);
                fields.push(DebugField::ContextSpanTrace(trace.status(), trace.text()));
            }
        }
        if detailed {
            #[cfg(feature = "error_location")]
            fields.push(DebugField::Location(message_text(report.location)?));
            #[cfg(feature = "error_span_trace")]
            fields.push(DebugField::SpanTrace(
                report.span_trace_status(),
                report.span_trace_text(),
            ));
            #[cfg(feature = "backtrace")]
            fields.push(DebugField::Backtrace(
                report.backtrace_status(),
                report.backtrace_text(),
            ));
        }
        Ok(Self {
            report: Some(PreparedReport { root, fields }),
            children,
        })
    }
}

#[cfg(feature = "alloc")]
fn group_reports<'a>(
    visit: impl FnOnce(usize, &mut dyn FnMut(crate::ErrorReport<'a>) -> bool) -> usize,
    detailed: bool,
    remaining: &mut usize,
) -> Result<DebugChildren, fmt::Error> {
    let mut groups = Vec::<DebugGroup>::new();
    let mut result = Ok(());
    let mut visited = 0;
    let total = visit(*remaining, &mut |child| {
        let position = visited;
        visited += 1;
        *remaining -= 1;
        result = (|| {
            let node = DebugNode::new(child.inner, detailed, remaining)?;
            if let Some(group) = groups.iter_mut().find(|group| group.node == node) {
                group.positions.push(position);
            } else {
                groups.push(DebugGroup {
                    positions: alloc::vec![position],
                    node,
                });
            }
            Ok(())
        })();
        result.is_ok() && *remaining != 0
    });
    result?;
    Ok(DebugChildren {
        groups,
        total,
        omitted: total - visited,
    })
}

#[cfg(feature = "alloc")]
pub(crate) fn debug_vector<'a>(
    f: &mut fmt::Formatter<'_>,
    visit: impl FnOnce(usize, &mut dyn FnMut(crate::ErrorReport<'a>) -> bool) -> usize,
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

#[cfg(feature = "filtered_backtrace")]
fn filtered_backtrace(backtrace: &std::backtrace::Backtrace) -> Option<String> {
    use alloc::{boxed::Box, vec::Vec};
    let trace = btparse::deserialize(backtrace).ok()?;
    let printer = color_backtrace::BacktracePrinter::new().add_frame_filter(Box::new(
        |frames: &mut Vec<&color_backtrace::Frame>| {
            frames.retain(|frame| {
                !(frame.is_dependency_code()
                    || frame.is_post_panic_code()
                    || frame.is_runtime_init_code()
                    // color-backtrace's runtime heuristics miss these glibc
                    // thread entry points, including their internal aliases.
                    || matches!(
                        frame.name.as_deref(),
                        Some("__clone" | "__clone3" | "__GI___clone" | "__GI___clone3")
                    ))
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

#[cfg(all(test, feature = "filtered_backtrace"))]
mod tests {
    use crate::{ErrorUnion, ErrorUnionVec};
    use alloc::{format, string::String, vec::Vec};
    use std::backtrace::Backtrace;

    #[inline(never)]
    fn captured_error(message: &'static str) -> ErrorUnion<(std::io::Error,)> {
        ErrorUnion::new_from_parts(
            std::io::Error::other(message),
            Backtrace::force_capture(),
            #[cfg(feature = "error_span_trace")]
            tracing_error::SpanTrace::capture(),
            #[cfg(feature = "context")]
            Vec::new(),
            #[cfg(feature = "error_location")]
            core::panic::Location::caller(),
        )
    }

    fn assert_filtered_trace(text: &str) {
        assert!(text.contains("formatting::tests::captured_error"), "{text}");
        assert!(text.contains("at "), "frame locations must remain: {text}");
        assert!(
            text.contains("frames hidden"),
            "runtime frames must be filtered: {text}"
        );
        assert!(!text.contains(" BACKTRACE "), "{text}");
        assert!(!text.contains('\u{1b}'), "{text}");
        for symbol in ["__clone", "__clone3", "__GI___clone", "__GI___clone3"] {
            let suffix = format!(": {symbol}");
            assert!(
                !text.lines().any(|line| line.ends_with(&suffix)),
                "glibc thread entry frames must be filtered: {text}"
            );
        }
    }

    #[test]
    fn filtered_backtrace_single_error_formats() {
        // Example output: tests/output_formatting/filtered_backtrace_single_error/debug.txt.
        let error = captured_error("invalid port").context("configure server");

        assert_eq!(format!("{error:#}"), "invalid port");
        assert_eq!(format!("{error}"), "invalid port");
        let expected = if cfg!(feature = "context") {
            "error: invalid port\ncontext:\n    - configure server"
        } else {
            "error: invalid port"
        };
        assert_eq!(format!("{error:#?}"), expected);

        let detailed = format!("{error:?}");
        std::println!("{detailed}");
        assert!(detailed.starts_with("error: invalid port\n"));
        let (message, trace) = detailed.split_once("\nbacktrace:\n").unwrap();
        assert_eq!(detailed.matches("backtrace:").count(), 1);
        if cfg!(feature = "context") {
            assert!(message.contains("\ncontext:\n    - configure server"));
        }
        assert_filtered_trace(trace);
        assert!(trace.lines().all(|line| line.starts_with("    ")));
        assert!(!detailed.ends_with('\n'));
    }

    #[test]
    fn filtered_backtrace_collection_formats() {
        // Example output: tests/output_formatting/filtered_backtrace_collection/debug.txt.
        let first = captured_error("invalid port").context("read http port");
        let second = captured_error("invalid port").context("read https port");
        let errors = ErrorUnionVec::new(alloc::vec![first, second]);
        let mut error: ErrorUnion<(ErrorUnionVec<(std::io::Error,)>,)> = errors.union();
        error.inner.diagnostics_mut().backtrace = Backtrace::force_capture();
        let error = error.context("configure server");

        assert_eq!(format!("{error:#}"), "errors (2 total):");
        assert_eq!(format!("{error}"), "errors (2 total):");
        let alternate = format!("{error:#?}");
        assert!(!alternate.contains("backtrace"), "{alternate}");
        assert!(!alternate.contains("captured_error"), "{alternate}");

        let detailed = format!("{error:?}");
        std::println!("{detailed}");
        assert!(detailed.starts_with("error:\n    errors (2 total):\n"));
        // Distinct capture sites remain distinct detailed reports, even when
        // both children have the same error message and context is disabled.
        assert!(detailed.contains("\n          at: [0]\n"), "{detailed}");
        assert!(detailed.contains("\n          at: [1]\n"), "{detailed}");
        assert_eq!(detailed.matches("backtrace:").count(), 3);
        let children: Vec<_> = detailed.split("\n          backtrace:\n").skip(1).collect();
        assert_eq!(children.len(), 2);
        for child in children {
            let trace = child
                .lines()
                .take_while(|line| line.starts_with("              "))
                .collect::<Vec<_>>()
                .join("\n");
            assert_filtered_trace(&trace);
        }
        assert_eq!(detailed.matches("\nbacktrace:\n").count(), 1);
        if cfg!(feature = "context") {
            assert!(detailed.contains("\ncontext:\n    - configure server"));
        }
        assert!(!detailed.contains(" BACKTRACE "), "{detailed}");
        assert!(!detailed.contains('\u{1b}'), "{detailed}");
    }

    #[cfg(feature = "json")]
    #[test]
    fn filtered_backtrace_json_matches_debug() {
        let error = captured_error("invalid port").context("configure server");
        let json = error.to_debug_json();
        let trace = json["backtrace"].as_str().unwrap();
        assert_filtered_trace(trace);
        let indented = trace
            .lines()
            .map(|line| format!("    {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(format!("{error:?}").ends_with(&format!("\nbacktrace:\n{indented}")));
        assert!(error.to_display_json().get("backtrace").is_none());
    }

    #[test]
    fn filtered_backtrace_removes_color_backtrace_banner() {
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

        let formatted = super::filtered_backtrace(&backtrace)
            .expect("filtered_backtrace must format without falling back");
        assert!(!formatted.contains(" BACKTRACE "), "{formatted}");
        assert!(
            formatted
                .contains("formatting::tests::filtered_backtrace_removes_color_backtrace_banner"),
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
