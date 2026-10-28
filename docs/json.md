# Structured JSON

Enable `json` to get reports as `serde_json::Value` with
`.to_display_json()` or `.to_debug_json()`.

This format is not covered by SemVer guarantees and may change in any release.

Each object describes one `ErrorUnion`. Display JSON contains `root` and
`sources`; Debug JSON adds diagnostics. Only `ErrorUnionVec::union()` adds
batch fields (`children` and `children_omitted`), with each child using the same
format.

| Field | Value |
| --- | --- |
| `root` | Root message string |
| `sources` | Message strings in source-chain order; empty array without sources |
| `children` | Batch child objects; omitted without expanded children |
| `children_omitted` | Number of immediate children skipped; omitted when zero |
| `contexts` | Context objects, innermost first; empty array without context |
| `contexts[].message` | Context message string |
| `contexts[].user_context` | Boolean: true for user context, false for regular context; omitted without the `user_context` feature |
| `location` | Root location; requires `error_location` |
| `contexts[].location` | Attachment location; requires `context_location` |
| `span_trace` | Root trace; requires `error_span_trace` |
| `contexts[].span_trace` | Context trace; requires `context_span_trace` |
| `backtrace` | Root backtrace; requires `backtrace` |

Children retain insertion order and duplicates, with **at most 10 expanded across
the entire tree**, visited depth-first. Diagnostics belong to their own union.

Messages normalize CRLF to LF and remove one final line ending, preserving other
whitespace and Unicode.

Locations contain `file` (string), `line`, and `column` (integers). Traces are
rendered strings, or `null` without a capture; disabled fields are omitted.

Messages and context frames that return formatting errors are omitted.

## Example

```rust
// ConfigError displays "cannot open configuration"; StartupError displays "startup failed".
// Both expose their boxed `source` through Error::source().

let error = eros::error!("permission denied")
    .map_inner(|source| ConfigError { source })
    .context("read /etc/app.toml")
    .context("start service");

println!("to_display_json: {}", error.to_display_json());
println!("to_debug_json: {}", error.to_debug_json());
```

With default Eros features, `json`, and `RUST_LIB_BACKTRACE=0`, the output is:

```text
to_display_json: {"root":"cannot open configuration","sources":["permission denied"]}
```
```text
to_debug_json: {"backtrace":null,"contexts":[{"message":"read /etc/app.toml"},{"message":"start service"}],"root":"cannot open configuration","sources":["permission denied"]}
```
