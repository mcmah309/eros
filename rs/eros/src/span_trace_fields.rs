use core::{cell::Cell, fmt};
use tracing_subscriber::{
    field::RecordFields,
    fmt::format::{FormatFields, Writer, debug_fn},
};

/// Formats each span field on its own line for Eros reports.
///
/// Install with `tracing_error::ErrorLayer::new(eros::SpanTraceFields)`.
/// Reports display a single field inline after `with:` and multiple fields
/// beneath it. Field values use their `Debug` representation.
#[derive(Clone, Copy, Debug, Default)]
pub struct SpanTraceFields;

impl<'writer> FormatFields<'writer> for SpanTraceFields {
    fn format_fields<R: RecordFields>(&self, writer: Writer<'writer>, fields: R) -> fmt::Result {
        let first = Cell::new(true);
        debug_fn(|writer, field, value| {
            if !first.replace(false) {
                writer.write_char('\n')?;
            }
            write!(writer, "{field}={value:?}")
        })
        .format_fields(writer, fields)
    }
}
