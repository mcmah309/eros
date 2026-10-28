# eros

[<img alt="github" src="https://img.shields.io/badge/github-mcmah309/eros-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/mcmah309/eros)
[<img alt="crates.io" src="https://img.shields.io/crates/v/eros.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/eros)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-eros-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/eros)
[<img alt="test status" src="https://img.shields.io/github/actions/workflow/status/mcmah309/eros/ci.yml?branch=master&style=for-the-badge" height="20">](https://github.com/mcmah309/eros/actions/workflows/ci.yml)
[<img alt="no_std" src="https://img.shields.io/badge/no__std-compatible-success?style=for-the-badge" height="20">](#no_std)

**Typed or untyped errors, with ergonomic propagation and no enum boilerplate.** Eros lets each API specify its possible error types or leave them untyped. Callers can handle specific error types and attach context as errors move through the call stack. Both approaches work in libraries and applications.

Choose the signature that fits your API:

```rust
# struct Config;
# use std::{io, num::ParseIntError as ParseError};
# {
// Propagate errors without their types.
fn load_config() -> eros::Result<Config>
# { Ok(Config) }
# assert!(load_config().is_ok());
# }

# {
// Propagate errors with their types.
fn load_config() -> eros::Result<Config, (io::Error, ParseError)>
# { Ok(Config) }
# assert!(load_config().is_ok());
# }
```

Built on the following philosophy:
1. Error types only matter when the caller cares about the type, otherwise this just hinders ergonomics and creates unnecessary noise. [Link](#optional-typed-errors)
2. There should be no boilerplate needed when handling any number of errors - no need to create an error enum for each case. [Link](#no-boilerplate)
3. Users should be able to seamlessly transition to and from fully typed errors and handle any cases they care about. [Link](#seamless-transitions-between-error-types)
4. Errors should always provide context of the operations in the call stack that lead to the error. [Link](#errors-have-context)
5. Error constructs should be performant. [Link](#optimizations)

## Philosophy In Action

### Optional Typed Errors

Error types only matter when the caller cares about the type, otherwise this just hinders ergonomics and creates unnecessary noise. Thus, it should be easy for the developer to make the type opaque for developing fast composable APIs. This is where [ErrorUnion](#errorunion) helps.

```rust
# #[cfg(feature = "alloc")]
# fn main() {
use eros::bail;

// Error type is untracked
fn validate_port(port: u16) -> eros::Result<()> {
    if port == 0 {
        bail!("Server port must be nonzero");
    }
    Ok(())
}

// Combine standard I/O and parse errors with an ad hoc validation error.
fn load_port(path: &str) -> eros::Result<u16> {
    let contents = std::fs::read_to_string(path)?;
    let port = contents.trim().parse()?;
    validate_port(port)?;
    Ok(port)
}

fn main() {
    if let Err(error) = load_port("config/port.txt") {
        println!("{error:#?}");
    }
}
# main();
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

### No Boilerplate

There should be no boilerplate needed when handling any number of errors (typed or untyped). This is where the magic of [ErrorUnion](#errorunion) happens.


```rust
# #[cfg(feature = "alloc")]
# fn main() {
use eros::IntoUnion;
use std::{io, num::ParseIntError};

// `ErrorUnion` is used to track each possible error type,
// instead of creating an enum for each possible error variant.
// `eros::Result<_,(..)>` is shorthand for  `Result<_,ErrorUnion<(..)>>`.
fn load_port(path: &str) -> eros::Result<u16, (io::Error, ParseIntError)> {
    let contents = std::fs::read_to_string(path).union()?;
    contents.trim().parse().union()
}

fn main() {
    if let Err(error) = load_port("config/port.txt") {
        println!("{error:#?}");
    }
}
# main();
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```
The above code is precisely typed for what we care about and there was no need to create an error enum for each case. One can also use a type alias like `type MyError = (io::Error, ParseIntError);` for reuse. See the [ErrorUnion](#errorunion) section for more details how it works.

### Seamless Transitions Between Error Types

Users should be able to seamlessly transition to and from fully typed errors and handle any cases they care about. One does not have to make the trade off or decide between untyped or typed code. Choose one or use each where it makes sense, either way eros composes perfectly.

#### Fully Typed

```rust
# #[cfg(feature = "alloc")]
# fn main() {
use eros::{IntoUnion, ReshapeUnion};
use std::{io, num::ParseIntError};

fn load_port(path: &str) -> eros::Result<u16, (io::Error, ParseIntError)> {
    let contents = std::fs::read_to_string(path).union()?;
    contents.trim().parse().union()
}

// I/O Error is no longer tracked, we handled internally.
fn port_or_default(path: &str) -> Result<u16, ParseIntError> {
    load_port(path)
        .recover::<io::Error, _>(|io_error| {
            // The handler retains the error's context and backtrace.
            eprintln!("Could not read {path}: {io_error:?}; using port 8080");
            8080
        })
        // Only ParseIntError remains. We *can* extract it at the API boundary if we want.
        .map_err(|error| error.into_single())
}

fn main() {
    if let Err(error) = port_or_default("config/port.txt") {
        eprintln!("Invalid server port: {error}");
    }
}
# main();
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

`recover` removes handled error types from the result's error set. Handle a group with `.recover::<(io::Error, ParseIntError), _>(|_| 8080)`. Successful values and unhandled errors pass through unchanged.

For manual branching, `narrow::<E, _>()` extracts `E` and discards its diagnostics. Use `narrow::<(E,), _>()` or `narrow::<(A, B), _>()` to retain diagnostics in a smaller union. The unmatched remainder always retains its diagnostics.

#### Untyped

Untyped APIs can still support typed recovery. For example, use a default when a configuration file is missing, while propagating permission and parse errors:

```rust
# #[cfg(feature = "alloc")]
# fn main() {
use eros::{ErrorUnion, ReshapeUnion};
use std::io;

fn load_port(path: &str) -> eros::Result<u16> {
    let contents = std::fs::read_to_string(path)?;
    Ok(contents.trim().parse()?)
}

fn port_or_default(path: &str) -> eros::Result<u16> {
    load_port(path).try_recover(|error: ErrorUnion<(io::Error,)>| {
        if error.kind() == io::ErrorKind::NotFound {
            Ok(8080)
        } else {
            Err(error.into())
        }
    })
}
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

The handler receives a typed error with its diagnostics. Other errors remain `AnyError`. You can also use `narrow` to select a concrete error or typed group from an untyped union or result.

Use `widen` to add possible error types:

```rust
# #[cfg(feature = "alloc")]
# fn main() {
use eros::{IntoUnion, ReshapeUnion};
use std::{io, net::{AddrParseError, IpAddr, TcpListener}, num::ParseIntError};

fn load_host(path: &str) -> eros::Result<IpAddr, (io::Error, AddrParseError)> {
    let contents = std::fs::read_to_string(path).union()?;
    contents.trim().parse().union()
}

fn load_port(path: &str) -> eros::Result<u16, (io::Error, ParseIntError)> {
    let contents = std::fs::read_to_string(path).union()?;
    contents.trim().parse().union()
}

fn bind_server() -> eros::Result<TcpListener, (io::Error, AddrParseError, ParseIntError)> {
    let host = load_host("config/host.txt").widen()?;
    let port = load_port("config/port.txt").widen()?;
    TcpListener::bind((host, port)).union()
}

fn main() {
    if let Err(error) = bind_server() {
        eprintln!("Could not start server: {error}");
    }
}
# main();
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

### Propagating Typed Errors Through An Untyped API

When callers no longer need the error set, `?` converts it automatically when propagating into `eros::Result<T>`. The concrete error (including any named wrapper) and all diagnostics are preserved.

```rust
use eros::ErrorUnion;

fn typed_operation() -> eros::Result<(), (std::fmt::Error,)> {
    Err(ErrorUnion::new(std::fmt::Error))
}

fn untyped_operation() -> eros::Result<()> {
    typed_operation()?;
    Ok(())
}
# assert!(untyped_operation().unwrap_err().is_inner::<std::fmt::Error>());
```

To convert the whole result directly, use `.erase()` with `use eros::Erase;`.

### Handling A Type Along An Otherwise Untyped Path

In a mostly untyped codebase, keep returning `eros::Result<T>` until a path
needs to explicitly track an error type. Use `classify` on the result to wrap
loader errors as `PortConfigError`. Downstream signatures describe the
categories callers care about without exposing implementation error types.
The callback receives an owned `InnerError`, including when the original error
already has the replacement type. Successful values pass through unchanged.

Here empty or malformed ports fail as `PortConfigError`. The typed path adds
`DynamicPortNotAllowed` for port 0, and the caller handles that error by using
port 8080:

```rust
# #[cfg(feature = "alloc")]
# fn main() {
use eros::{Erase, ReshapeUnion, SendSyncError};

#[derive(Debug, thiserror::Error)]
#[error("no port configured")]
struct MissingPort;

#[derive(Debug, thiserror::Error)]
#[error("port 0 would let the OS choose a port")]
struct DynamicPortNotAllowed;

#[derive(Debug, thiserror::Error)]
#[error("configuration failed: {0}")]
struct PortConfigError(#[source] Box<dyn SendSyncError>);

fn load_port(contents: &str) -> eros::Result<u16> {
    if contents.trim().is_empty() {
        eros::bail!(MissingPort);
    }
    Ok(contents.trim().parse()?)
}

fn checked_port(contents: &str)
    -> eros::Result<u16, (DynamicPortNotAllowed, PortConfigError)>
{
    let port = load_port(contents).classify(|inner| PortConfigError(inner.into()))?;
    eros::ensure!(port != 0, DynamicPortNotAllowed);
    Ok(port)
}

fn port_or_default(contents: &str) -> eros::Result<u16> {
    let remaining: eros::Result<u16, (PortConfigError,)> =
        checked_port(contents).recover::<DynamicPortNotAllowed, _>(|error| {
            eprintln!("No fixed port configured: {error:?}; using 8080");
            8080
        });
    remaining.erase()
}

# assert_eq!(port_or_default("9000").unwrap(), 9000);
# assert!(port_or_default("").unwrap_err().is_inner::<PortConfigError>());
# assert_eq!(port_or_default("0").unwrap(), 8080);
# assert!(port_or_default("invalid").unwrap_err().is_inner::<PortConfigError>());
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

`classify(f)` is shorthand for `map_inner(f).widen()`: it transforms every error
and infers the destination error set from the surrounding type. The destination
must contain the replacement type; it does not control whether the callback
runs. Use `map_inner` alone when a singleton error set is sufficient. The original
error becomes a source only if the callback retains it, as this wrapper does.
Saved diagnostics remain with the replacement; no new diagnostics are captured.

For a selective transformation, use `narrow` to separate the desired types,
map that branch with `map_inner`, and use `widen` to combine the possible errors.

`InnerError` and its downcasting methods are available with every feature
configuration. Enabling `alloc` adds `InnerError::into_boxed()` and conversion
into `Box<dyn SendSyncError>` through `.into()` to preserve the original concrete
error as an owned source. An already boxed root reuses its
allocation when root diagnostics are disabled; inline nonzero roots and roots
sharing an allocation with diagnostics move into a new box when that method
is called. The replacement follows the compile-time storage choice.

Mapping into domain errors also keeps failures from different operations distinct. An
`io::Error` while reading configuration and an `io::Error` while connecting to
a database have the same concrete type, but callers may need to handle them
differently. Wrap them at their respective boundaries as `PortConfigError`
and `DatabaseError` before combining the paths with `widen`. The combined set
can be `(DynamicPortNotAllowed, PortConfigError, DatabaseError)`:
recovering a configuration failure then leaves database failures untouched.
The original errors can remain inside the wrappers as sources for diagnostics.
Paths intentionally using the same category can share one variant. Use
`recover`, `try_recover`, or `narrow` where those categories are handled.

`erase()` returns to `eros::Result<T>` while preserving the concrete root,
including its boxes and domain wrappers. Later callers can still downcast or
recover `PortConfigError`. Only the static error set changes; all saved diagnostics
remain intact. The method is also available directly on
`ErrorUnion`. Plain `into()` and `?` perform the same type erasure when converting
a typed union into an untyped one.

### Adding Context As Errors Propagate

Errors should always provide context of the operations in the call stack that led to the error. Users can add context with `.context` or `.with_context`. Errors also capture a `Backtrace`.

For code already instrumented with `tracing`, [span traces](#tracing-span-traces) offer an alternative to adding operation context manually: the `error_span_trace` feature captures the active span and its parents, including their fields, when an error first enters Eros. Use `context_span_trace` to capture spans at context attachments, or `span_trace` for both.

```rust
# #[cfg(feature = "alloc")]
# fn main() {
use eros::Context;

fn parse_port(value: &str) -> eros::Result<u16> {
    Ok(value.parse::<u16>().context("Parse server port")?)
}

fn configure_server(name: &str, port: &str) -> eros::Result<u16> {
    parse_port(port).with_context(|| format!("Configure server {name}"))
}

fn main() {
    if let Err(error) = configure_server("api", "not-a-port").context("Start application") {
        println!("{error:?}");
    }
}
# main();
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

With `RUST_BACKTRACE=1`:

```console
error: invalid digit found in string
context:
    - Parse server port
    - Configure server api
    - Start application
backtrace:
    [3 frames hidden for brevity]
       3: example::parse_port
                 at ./src/main.rs:4:29
       4: example::configure_server
                 at ./src/main.rs:8:5
       5: example::main
                 at ./src/main.rs:12:25
    [17 frames hidden for brevity]
```

#### Location

The `error_location` feature records where an error first enters Eros. The `context_location` feature records where each context is attached when `context` is enabled. Enable `location` as a shortcut for both. Location capture is lighter than a full backtrace and works in wasm and `no_std` environments.

### Reducing Error Handling Overhead

Eros comes with the `context` and `backtrace` feature flags enabled by default. Disabling them removes backtrace and context tracking from `ErrorUnion<T>`, and all context methods become a no-op. Thus, it may be optimized away by the compiler.

`ErrorUnion` occupies two pointer-sized words, growing to three only when `error_location` is enabled without `alloc`. The possible error types add no storage. Storage is selected at compile time:

| Configuration | Storage | Root allocations |
| --- | --- | --- |
| `alloc`, no root diagnostics, error fits one word | Inline payload and static operations pointer | None |
| `alloc`, no root diagnostics, larger or more aligned error | Thin `Box<T>` pointer and static operations pointer | One |
| `alloc`, root diagnostics enabled | Boxed fat pointer; diagnostics and trailing payload share one allocation | One |
| No `alloc`, error fits one word | Inline payload and static operations pointer, plus optional location reference | None |

Root diagnostics are `backtrace`, `context`, `error_span_trace`, and `error_location`. When any are enabled with `alloc`, their fields live in the heap allocation and never enlarge the handle. Trace capture and context contents may allocate separately. `context_location` and `context_span_trace` describe context frames and do not by themselves add root diagnostic fields.

Inline nonzero payloads must fit both the size and alignment of `usize`. ZSTs support arbitrary alignment. Without `alloc`, oversized or over-aligned nonzero errors are rejected at compile time. The handle uses two words without root location and three with it.

Widening, narrowing, and erasing reuse storage. Typed extraction, such as `into_single`, adds no allocation. `into_inner` returns an owned `InnerError` without allocating and discards saved diagnostics. Converting that owner with `into_boxed()` reuses an existing `Box<T>` when root diagnostics are disabled; inline nonzero errors and errors stored with diagnostics move into a new box. Boxing a ZST requires no allocation. `map_single` keeps inline results allocation-free and reuses boxed storage when complete sizes and alignments match.

See the [Use In Libraries](#use-in-libraries) section as well.

## `ErrorUnion`

### Open Sum Type

`ErrorUnion` is an open sum type. An open sum type takes full advantage of rust's powerful type system. It differs from an enum in that you do not need to define any actual new type in order to hold some specific combination of variants, but rather you simply describe the ErrorUnion as holding one value out of several specific possibilities. This is declared by using a tuple of those possible variants as the generic parameter for the `ErrorUnion`. 

For example, a `ErrorUnion<(io::Error, fmt::Error)>` contains either a `io::Error` or a `fmt::Error`. The benefit of this over creating specific enums for each function becomes apparent in larger codebases where error handling needs to occur in different places for different errors. As such, `ErrorUnion` allows you to quickly specify a function's return value as involving a precise subset of errors that the caller can clearly reason about. This Provides maximum composability with no boilerplate. E.g.

```rust
# #[cfg(feature = "alloc")]
# fn main() {
use eros::ErrorUnion;
use std::{fmt, io};

fn main() {
    let error: ErrorUnion<MyError>;
    let result: eros::Result<(), MyError>;
}

type MyError = (fmt::Error, io::Error); // A type alias can be used to make statements more concise
# main();
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```
vs
```rust
# #[cfg(feature = "alloc")]
# fn main() {
use std::{fmt, io};

fn main() {
    let error: MyError;
    let result: Result<(), MyError>;
}

#[derive(Debug)]
pub enum MyError {
    FmtError(fmt::Error),
    IoError(io::Error),
}

impl std::fmt::Display for MyError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MyError::FmtError(e) => write!(fmt, "{}", e),
            MyError::IoError(e) => write!(fmt, "{}", e),
        }
    }
}

impl std::error::Error for MyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MyError::FmtError(e) => e.source(),
            MyError::IoError(e) => e.source(),
        }
    }
}

impl From<fmt::Error> for MyError {
    fn from(error: fmt::Error) -> Self {
        MyError::FmtError(error)
    }
}

impl From<io::Error> for MyError {
    fn from(error: io::Error) -> Self {
        MyError::IoError(error)
    }
}
# main();
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```
Additionally, the complexity of the second option grows exponentially the more error enums have to be combined from different functions. That is why a lot of crates opt for not precisely defining errors for APIs and instead choose a single error enum or struct for the entire crate. See the [Why Traditional Enum Errors Scale Poorly](#why-traditional-enum-errors-scale-poorly) section for a deeper dive into this.

When the error union should encompass the full set of possible errors, use `AnyError`:

`eros::Result<()>` is shorthand for `eros::Result<(), AnyError>`, which in turn is shorthand for `Result<(), ErrorUnion<AnyError>>`.

### Tracing

`ErrorUnion` also allows adding context to an error throughout the callstack with the `context` or `with_context` methods. This context may be information such as variable values or ongoing operations while the error occurred. If the error is handled higher in the stack, then this can be disregarded (no log pollution). Otherwise you can log it (or panic), capturing all the relevant information in one log. A backtrace is captured and included with the error when `RUST_BACKTRACE` is set.

## Context Macro

For some functions, one may want to attach the same context to every error that can be returned from that function. Writing `.with_context(...)` on each fallible call quickly becomes repetitive and can obscure the intent of the function. For example:

```rust
# #[cfg(feature = "alloc")]
# fn main() {
use eros::Context;

fn load_port(path: &str) -> eros::Result<u16> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to load server port from {}", path))?;
    let port = contents.trim().parse::<u16>()
        .with_context(|| format!("Failed to load server port from {}", path))?;
    Ok(port)
}

fn main() {
    if let Err(error) = load_port("config/port.txt") {
        println!("{error:#?}");
    }
}
# main();
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

To help with this, `eros` provides the `context` attribute macro. The macro wraps the function and automatically adds the supplied context to any error returned from it:

```rust
# #[cfg(feature = "alloc")]
# fn main() {
use eros::{Context, context};

// Use #[eager_context(...)] to format owned parameters before the body runs, even on success.
#[context("Failed to load server port from {}", path)]
fn load_port(path: &str) -> eros::Result<u16> {
    let contents = std::fs::read_to_string(path)?;
    let port = contents.trim().parse::<u16>()?;
    Ok(port)
}

fn main() {
    if let Err(error) = load_port("config/port.txt") {
        println!("{error:#?}");
    }
}
# main();
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

Whether reading the file or parsing its contents fails, the error includes `Failed to load server port from config/port.txt` alongside the original I/O or parse error. The macro attaches this context once, while keeping the function body focused on the actual logic.

### Automatic Context from Parameters

When the context simply consists of "which arguments was this function called with?", the format string can often be inferred automatically.

Instead of providing an explicit format string, use `#[context]` and annotate the parameters that should appear in the generated context with `#[fmt(...)]` where `...` is the desired formatting:

```rust
# #[cfg(feature = "alloc")]
# fn main() {
use eros::{context, Context};

#[derive(Debug)]
struct Flags {
    enabled: bool,
}

fn result() -> eros::Result<()> {
    eros::bail!("This is an error")
}

#[context]
fn process(
    #[fmt("{}")] name: &str,
    count: usize,
    #[fmt("{:?}")] flags: &Flags,
) -> eros::Result<()> {
    result()?;
    result()?;
    Ok(())
}

fn main() {
    process(
        "example",
        42,
        &Flags { enabled: true },
    );
}
# main();
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

The generated context is equivalent to:

```rust
# let name = "example";
# let flags = true;
# let message =
format!(
    "name: {}\nflags: {:?}\n",
    name,
    flags,
)
# ;
# assert_eq!(message, "name: example\nflags: true\n");
```

Only annotated parameters are included in the generated context. Parameters without `#[fmt(...)]` are ignored, allowing sensitive values or uninteresting arguments to be omitted.

## Formatting

`ErrorUnion` and `ErrorUnionVec` support four report formats:

| Format | `ErrorUnion` | `ErrorUnionVec` |
| --- | --- | --- |
| `{:#}` (`Display`) | Root only | `errors (N total):` heading |
| `{}` (`Display`) | Root and `Error::source()` chain | Same heading |
| `{:#?}` (`Debug`) | Root, sources, and contexts | `errors (N total):` list of grouped child reports |
| `{:?}` (`Debug`) | Same report, plus enabled locations, backtrace, and span trace | Same grouping, including metadata |

Sources and contexts use bullets; contexts appear innermost first. Empty
sections are omitted. Alternate Debug omits locations, backtraces, and span traces
at every nesting level. Formatting never captures new diagnostics.

Debug groups identical visible child reports in first-occurrence order, with
zero-based positions and half-open ranges (`0..3` means indices 0, 1, and 2):

```rust
# #[cfg(feature = "alloc")]
# fn main() {
use eros::{ErrorUnion, ErrorUnionVec};

let errors: ErrorUnionVec<(std::io::Error,)> = [
    "invalid input", "invalid input", "invalid input",
    "permission denied", "invalid input", "invalid input",
]
.into_iter()
.map(|message| ErrorUnion::new(std::io::Error::other(message)))
.collect();

println!("{errors:#?}");
# assert_eq!(format!("{errors:#?}"), "errors (6 total):\n    - error: invalid input\n      at: [0..3, 4..6]\n    - error: permission denied\n      at: [3]");
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

```text
errors (6 total):
    - error: invalid input
      at: [0..3, 4..6]
    - error: permission denied
      at: [3]
```

Different sources, contexts, or nested children form separate groups; ordinary
Debug also distinguishes locations, backtraces, and span traces.

See [Logging](#logging) for tracing fields and
[Structured JSON](#structured-json) for JSON reports.

## Best Practices

### Use In Libraries

`eros`'s flexibility and optimizations make it the perfect option for both libraries and binaries.

*Libraries should consider setting `default-features = false` on their `eros` dependency* and allowing downstream crates to enable the diagnostic features they need. Keep `features = ["alloc"]` enabled if the library stores errors exceeding one word in size or alignment. Without it, concrete errors must fit one word, or be zero-sized. The library can enable diagnostic features for its own tests.

#### Public APIs

##### Approach A: Exposing `ErrorUnion` in Public APIs

Exposing `ErrorUnion` in a public API is sometimes preferred, especially if crates are internal. It allows multiple crates to use the power of these constructs together.

Before choosing this approach ask "Will a downstream care about differentiating between these types?" e.g. exposing a `io::Error` may or may not be useful. In fact, it may be more useful for a downstream to expose a type specific to the crate e.g. `CrateError`. One could then expose an e.g.`ErrorUnion<(CrateError,)>`. But if it is desired to remove `ErrorUnion` altogether, approach B dives deeper into this.

##### Approach B: Replacing `ErrorUnion` With Concrete Crate Errors

If one wants to add a custom error type for public APIs without exposing constructs like `ErrorUnion`, use the `into_inner()` method at these boundaries. This is a common pattern, since most crates already define their own error type for public-facing APIs.

<details>

<summary>Example Implementation</summary>

```rust
# #[cfg(feature = "alloc")]
# fn main() {
use std::io;

use eros::{ErrorUnion, SendSyncError, TypeSet, bail};

#[derive(Debug)]
struct CrateError(Box<dyn SendSyncError>);

impl std::fmt::Display for CrateError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "CrateError: {}", self.0)
    }
}

impl std::error::Error for CrateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.0.as_ref())
    }
}

impl<T: TypeSet> From<ErrorUnion<T>> for CrateError {
    fn from(e: ErrorUnion<T>) -> Self {
        CrateError(e.into_inner().into_boxed())
    }
}

fn internal_api() -> eros::Result<()> {
    bail!(io::Error::new(io::ErrorKind::Other, "io error"))
}

pub fn public_api() -> Result<(), CrateError> {
    internal_api().map_err(Into::into)
}
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

</details>

This way the library can use `ErrorUnion` internally for function composition, enabling features like `context` and `backtrace` for its own tests, while downstream crates only see a single concrete error type. Converting at the boundary with `into_inner()` discards saved diagnostics. It returns an owned `InnerError` without allocating or changing the concrete root type. Call `.into_boxed()` on that owner if the public wrapper stores a boxed source, as above. The conversion cost does not depend on the number of possible error variants.

For typed tuples, the [error enum macro](#error-enum-macro) generates a concrete error enum for the public API. Convert at the boundary with `map_err(Into::into)`:

<details>

<summary>Example Implementation</summary>

```rust
# #[cfg(feature = "alloc")]
# fn main() {
use eros::IntoUnion;
use std::{fmt, io};

#[eros::error_enum(CrateError, "crate operation failed: {0}")]
pub type CrateErrors = (io::Error, fmt::Error);

fn regular_typed_result1() -> Result<(), io::Error> {
    Err(io::Error::new(io::ErrorKind::AddrInUse, "message here"))
}

fn regular_typed_result2() -> Result<(), fmt::Error> {
    Err(fmt::Error)
}

fn internal_api() -> eros::Result<(), CrateErrors> {
    regular_typed_result1().union()?;
    regular_typed_result2().union()?;
    Ok(())
}

pub fn public_api() -> Result<(), CrateError> {
    internal_api().map_err(Into::into)
}

fn main() {
    match public_api() {
        Ok(()) => println!("Success!"),
        Err(CrateError::Io(e)) => println!("IO error: {}", e),
        Err(CrateError::Fmt(e)) => println!("Format error: {}", e),
    }
}
# main();
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

</details>

With this, internal composing of errors can remain precise and ergonomic vs traditional enums, as outlined in [Why Traditional Enum Errors Scale Poorly](#why-traditional-enum-errors-scale-poorly) section. And downstream users are not exposed to the `ErrorUnion` type and instead see a traditional error enum. Note even if typed tuples are used internally, it is still completely valid to choose to type erase at the boundary with `into_inner`.

### Context Placement: Two Approaches

There are two reasonable philosophies for *where* in the call stack context should be attached. Eros is flexible enough to support either, but it's worth picking one and being consistent within a codebase.

#### Approach 1: Attach Context at the Function

Under this approach, a function attaches context describing itself and its own parameters via `#[context]`. The function "owns" its own description, so every caller gets the same context for free, with no risk of forgetting it or duplicating it slightly differently at each call-site.

```rust
# #[cfg(feature = "alloc")]
# fn main() {
use eros::{Context, context};

#[context("Failed to do some action. param was {}", param)]
fn do_some_action(param: &str) -> eros::Result<()> {
    eros::bail!("This is an error")
}

fn func1() -> eros::Result<()> {
    let param = "xyz";
    do_some_action(param)
}

fn func2() -> eros::Result<()> {
    let param = "abc";
    do_some_action(param)
}

fn main() {
    func1();
    func2();
}
# main();
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

**Why:** it removes ambiguity about whose job it is to attach context. If every function attaches context describing its own operation and inputs, nothing needs to be re-derived or duplicated by callers, and nothing is silently dropped because every caller assumed some other layer would handle it.

**Tradeoff:** if a function's parameter is itself just forwarded from its caller (and that caller already attaches it too), the same value can appear in context more than once as the error bubbles up. Note if the `context` feature is disabled `with_context` becomes a no-op, so the cost is avoidable.

#### Approach 2: Attach Context Only at the Boundary Where Information Would Otherwise Be Lost

A function should only attach context that the caller doesn't already have. If a caller already knows the value of `param`, then a callee re-stating `param` in its own context adds no new information, just noise.

Responsibility for attaching a given piece of context passes transitively up the stack to whichever function is the last one that still has access to the information, even if that's several layers above where the error actually originated. In the example below, `func2` attaches nothing — it leaves that to its callers — and the context only gets attached at `func1` and `func1b`, the two places where `param` would otherwise be lost:

```rust
# #[cfg(feature = "alloc")]
# fn main() {
use eros::Context;

fn do_some_action(param: &str) -> eros::Result<()> {
    eros::bail!("This is an error")
}

fn func2(param: &str) -> eros::Result<()> {
    do_some_action(param)
}

fn func1() -> eros::Result<()> {
    let param = "xyz";
    func2(param).with_context(|| format!("Failed to do some action. param was {}", param))
}

fn func1b() -> eros::Result<()> {
    let param = "abc";
    func2(param).with_context(|| format!("Some action failed with param {}", param))
}
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

**Why:** it keeps individual error messages lean, avoids restating the same value at every layer, and sidesteps the classic `connection failed: connection failed: connection failed: no route to host` style of redundant, nested context.

**Tradeoff:** because no single function is locally responsible for attaching a given piece of context, it takes discipline and more time has to be spent at each call-site — you have to ask "would this information otherwise be lost going up the stack from here?" If a function is called from many places, that question has to be answered (and the same context written) at each call-site rather than once at the function's own definition. It's also easy to accidentally end up restating context slightly differently at two call-sites — in the example above, `func1` and `func1b` phrase the same underlying fact as `"Failed to do some action. param was {}"` and `"Some action failed with param {}"` — since nothing enforces a single canonical phrasing the way `#[context]` on the function itself does. It is also easy to get lazy and skip attaching context altogether at a given call-site, silently losing information that would have been captured automatically under Approach 1.

### Backtrace vs Location

`backtrace` captures a backtrace at error creation when enabled by the backtrace environment settings. For lighter location tracking, enable `error_location` to capture where an error first enters Eros, `context_location` to capture where each context is attached, or `location` for both. The two location flags work independently and can be combined with `backtrace` or `span_trace`. They use compiler-provided call locations and support wasm and `no_std` environments. None of the location flags is enabled by default; `backtrace` is.

### Logging

Choose a [formatting mode](#formatting) for the information you want to log:

| Format | Tracing field |
| --- | --- |
| `{}` | `error = %error` |
| `{:#}` | `error = %format_args!("{error:#}")` |
| `{:#?}` | `error = %format_args!("{error:#?}")` |
| `{:?}` | `error = ?error` |

For JSON report values, see [Structured JSON](#structured-json).

For logging, [`err_trail`](https://github.com/mcmah309/err_trail) is recommended. Libraries can use its constructs like `error!`, `warn!`, `info!`, `debug!`, and `trace!` macros while downstream applications select the `tracing`, `log`, or `defmt` backend. If no backend is selected by a downstream, then the code is optimized away by the compiler.

The `ErrContext` trait in `err_trail` also provides methods like `.warn(())`, which logs an `Err` using `Display` and returns the result unchanged:

```rust
# #[cfg(feature = "alloc")]
# fn main() {
use eros::bail;
use err_trail::ErrContext;

fn eros_result() -> eros::Result<()> {
    bail!("Something went wrong")
}

fn main() {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_ansi(false)
        .init();

    if let Err(error) = eros_result() {
        err_trail::error!(error = %error, "Operation failed");
    }

    let _result = eros_result().warn(());
}
# main();
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

The `error!` call emits:

```text
ERROR Operation failed error=Something went wrong
```

The `.warn(())` call emits:

```text
WARN Something went wrong
```

## Additional Features

### Tracing Span Traces

The optional `error_span_trace` feature captures the current
[`tracing`](https://docs.rs/tracing) span and its parents, including fields and
source locations, when an error first enters Eros. `context_span_trace` captures
that information separately when each context is attached. Enable `span_trace`
as a shortcut for both. Both capture features require `std`; `context_span_trace`
does not enable `context` itself.

```rust
# #[cfg(feature = "error_span_trace")]
# fn main() {
use tracing_subscriber::prelude::*;

#[tracing::instrument]
fn load_settings(path: &str, attempt: u32) -> eros::Result<()> {
    eros::bail!("invalid settings")
}

tracing_subscriber::registry()
    .with(tracing_error::ErrorLayer::new(eros::SpanTraceFields))
    .init();

if let Err(error) = load_settings("config.toml", 2) {
    println!("{error:?}");
}
# }
# #[cfg(not(feature = "error_span_trace"))]
# fn main() {}
```

With backtrace capture disabled (file path shortened):

```text
error: invalid settings
span trace:
    - span_trace::load_settings
      with:
        path="config.toml"
        attempt=2
      location: span_trace.rs:4
backtrace: not captured
```

Here, `bail!` captures the `load_settings` span before the function returns.
Converting an error into an Eros error after the function returns would miss that span.

Print the saved traces with `{:?}`. With `error_span_trace`, inspect the original
trace through `.span_trace()` on an `ErrorUnion` or `ErrorReport`. With `context`
and `context_span_trace`, inspect each attachment trace through
`error.contexts()` and `ContextFrame::span_trace()`. Attaching context preserves
the original error trace, so a context attached by an instrumented caller can
record a different span from the error's origin. Detailed reports show each
context's trace beneath its message; debug JSON includes `span_trace` on each
context object. Display (`{}`) and alternate Debug (`{:#?}`) omit all traces.

### Filtered Backtrace

Other backtrace examples in this README use `[N frames hidden for brevity]` to mark frames omitted from the documentation. The default output still includes those frames. Enable `filtered_backtrace` to filter recognized dependency and runtime frames automatically.

Filtering only changes formatted output; the original trace remains available
through `.backtrace()`.

### Composing Type Sets

Use `type_set!` to share members between named error sets. Join sets and tuples
with `||`; the macro emits flat tuple aliases and includes repeated members once:

```rust
use eros::{type_set, ErrorUnion};
use core::{fmt, num::{ParseIntError, ParseFloatError}};

type_set! {
    type ParseErrors = (ParseIntError, ParseFloatError);
    pub type AppErrors = ParseErrors || (fmt::Error,);
    type Combined = AppErrors || ParseErrors;
}

let error: ErrorUnion<Combined> = ErrorUnion::new(fmt::Error);
let _: ErrorUnion<(ParseIntError, ParseFloatError, fmt::Error)> = error;
```

References must name sets in the same invocation, in any declaration order.
Members keep their first occurrence order. Deduplication compares type spellings;
use the same path for shared members, since the macro cannot resolve different
paths or external aliases for the same type. Tuples can be empty or contain one
member, written `()` and `(T,)`. Generic aliases are not supported, but members
can be generic types. Sets support up to 26 distinct members.

Visibility and attributes apply to the emitted aliases. You can place
`#[eros::error_enum(Name)]` on a composed alias inside `type_set!` to generate
an enum from its flattened tuple. A `#[cfg(...)]` attribute gates the emitted
alias; composition still uses its declared members.

### Error Enum Macro

`#[eros::error_enum(Name)]` keeps a tuple alias and generates an owned enum with `Debug`, `Display`, and `Error` implementations, useful for matching errors or [exposing a concrete public error](#approach-b-replacing-errorunion-with-concrete-crate-errors). Add `error_enum_ref` and `error_enum_mut` for borrowed views, or `error_enum_kind` for unit variants without payloads:

```rust
# #[cfg(feature = "alloc")]
# fn main() {
use eros::ErrorUnion;
use std::{fmt, io};

#[eros::error_enum(Error)]
#[eros::error_enum_ref(ErrorRef)]
#[eros::error_enum_mut(ErrorMut)]
#[eros::error_enum_kind(ErrorKind)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub type Errors = (io::Error, fmt::Error);

let mut union: ErrorUnion<(fmt::Error,)> = ErrorUnion::new(fmt::Error);
assert_eq!(ErrorKind::from(&union), ErrorKind::Fmt);
let _shared: ErrorRef<'_> = (&union).into();
let _mutable: ErrorMut<'_> = (&mut union).into();
match Error::from(union) {
    Error::Io(error) => eprintln!("I/O error: {error}"),
    Error::Fmt(error) => eprintln!("Formatting error: {error}"),
}
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

Each macro works independently. Attributes below it apply to that enum until the next enum macro or the alias. Variant names join path segments in PascalCase and strip a trailing `Error` unless that would leave an empty name. The kind enum implements `Debug`; attach other derives explicitly.

`Display` delegates to the contained error, and `Error::source()` returns it. An optional format, such as `#[eros::error_enum(Error, "operation failed: {0}")]`, customizes the message.

Conversions accept subsets and reordered tuples. Borrowing preserves diagnostics; owned conversion discards them. For erased unions (`ErrorUnion<AnyError>`), use `TryFrom`, which returns the original value or borrow on a mismatch.

### Adding Source Chains

Use `map_inner` to change the main error while keeping the original failure as its source. The closure receives an owned `InnerError`; convert it into a box to store it and expose it through `Error::source()`:

```rust
# #[cfg(feature = "alloc")]
# fn main() {
use eros::SendSyncError;
use std::{error::Error, fmt};

#[derive(Debug)]
struct UpdateError {
    source: Box<dyn SendSyncError>,
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Update preparation failed")
    }
}

impl Error for UpdateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

let error = eros::error!("TLS certificate has expired")
    .map_inner(|source| UpdateError { source: source.into() });

println!("{error}");
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

```text
Update preparation failed
    caused by:
        - TLS certificate has expired
```

For a union with a single possible error type, `as_single` and `as_single_mut` borrow the concrete error as `&T` and `&mut T`, preserving all saved diagnostics. `map_single` passes the concrete error to the closure and preserves that metadata. Or use `into_single` to extract that error and discard the metadata.


### Structured JSON

See the [Structured JSON documentation](https://github.com/mcmah309/eros/blob/master/docs/json.md).

### Exposing Errors To Application Users

Not every error message should be shown directly to end users of an application.

Operational details such as SQL queries, filesystem paths, internal identifiers, etc. often make poor user experiences and may even leak sensitive information. Conversely, some errors are intentionally designed to be presented directly to users.

Instead, construct a user-facing message from two sources:

1. User context attached throughout the call stack. Enabled through the `user_context` feature flag
2. The underlying error itself, if the application recognizes it as safe to display.

<details>

<summary>Example Implementation</summary>

```rust
# #[cfg(feature = "user_context")]
# fn main() {
use eros::{Context, ErrorUnion, Erase, SendSyncError, TypeSet};

#[derive(Debug)]
struct SystemDiskError;

impl std::fmt::Display for SystemDiskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Critical IO failure at block 0x7FA3")
    }
}

impl std::error::Error for SystemDiskError {}

#[derive(Debug)]
struct InvalidPasswordError;

impl std::fmt::Display for InvalidPasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Your password must be at least 8 characters long.")
    }
}

impl std::error::Error for InvalidPasswordError {}

// Add valid errors to display to the user here
fn root_error_message(error: &dyn SendSyncError) -> String {
    if let Some(error) = (error as &dyn core::any::Any).downcast_ref::<InvalidPasswordError>() {
        error.to_string()
    } else {
        "An internal error occurred.".to_owned()
    }
}

fn build_user_error_message<E>(error: &ErrorUnion<E>) -> String
where
    E: TypeSet,
{
    let mut message = String::new();
    let error_message = root_error_message(error.inner());
    message.push_str(&error_message);

    for frame in error.contexts().filter(|frame| frame.is_user_context()) {
        message.push('\n');
        message.push_str(&frame.to_string());
    }

    message
}

fn validate_password(password: &str) -> eros::Result<()> {
    if password.len() < 8 {
        return Err(InvalidPasswordError)
            .context("Password validation failed")
            // This context is marked as "user facing" meaning it is safe to expose to the user
            .user_context("Please choose a stronger password.")
            .erase();
    }
    Ok(())
}
fn load_configuration() -> eros::Result<()> {
    Err(SystemDiskError)
        .context("Failed to read configuration from /etc/my-app/config.toml")
        .erase()
}
fn main() {
    let password_error = validate_password("123").unwrap_err();
#     assert_eq!(build_user_error_message(&password_error), "Your password must be at least 8 characters long.\nPlease choose a stronger password.");
    println!("User message:");
    println!("{}", build_user_error_message(&password_error));
    let system_error = load_configuration().unwrap_err();
#     assert_eq!(build_user_error_message(&system_error), "An internal error occurred.");
    println!("\nUser message:");
    println!("{}", build_user_error_message(&system_error));
}
# main();
# }
# #[cfg(not(feature = "user_context"))]
# fn main() {}
```

```text
User Message:
Your password must be at least 8 characters long.
Please choose a stronger password.
User Message:
An internal error occurred.
```

</details>

This approach keeps internal diagnostics while making the user-facing experience explicit. Applications remain free to decide which information is safe to expose, while `ErrorUnion` continues to focus on error composition, tracing, and context propagation.

### Anyhow

`eros` comes with an `anyhow` feature flag. This adds a `ErrorUnion::from_anyhow` function for converting an `anyhow::Error` to an `ErrorUnion`. This can help integrate with legacy code.

`eros` can also quickly replace `anyhow` in any crate as simple as replacing all occurrences of:
- `anyhow!` with `error!`
- `anyhow::Error` with `eros::ErrorUnion`
- `anyhow::` with `eros::`

`anyhow` and `eros` give context different roles. In `anyhow`, each `.context(...)` call adds an outer layer to the error chain. The latest context becomes the main message, while earlier contexts and the original error remain as causes. As an error travels through its callers, this presents the outermost operation first.

```rust
# #[cfg(feature = "anyhow")]
# fn main() {
let error = anyhow::anyhow!("TLS certificate has expired")
    .context("fetch https://updates.example.com/manifest.json")
    .context("prepare application update to v2.4.0");

println!("{error:?}"); // Debug shows the main message and its causes.
# assert_eq!(error.to_string(), "prepare application update to v2.4.0");
# assert_eq!(error.chain().last().unwrap().to_string(), "TLS certificate has expired");
# }
# #[cfg(not(feature = "anyhow"))]
# fn main() {}
```

```text
prepare application update to v2.4.0

Caused by:
    0: fetch https://updates.example.com/manifest.json
    1: TLS certificate has expired
```

`eros` keeps the original error as the main message and presents context separately, in the order it was added. As the error travels up the call stack, context follows stack trace order: nearest the failure first, then outward through its callers:

```rust
# #[cfg(feature = "context")]
# fn main() {
# use eros::Context;
let error = eros::error!("TLS certificate has expired")
    .context("fetch https://updates.example.com/manifest.json")
    .context("prepare application update to v2.4.0");

println!("{error:#?}"); // Alternate Debug omits locations, backtraces, and span traces.
# assert_eq!(format!("{error:#?}"), "error: TLS certificate has expired\ncontext:\n    - fetch https://updates.example.com/manifest.json\n    - prepare application update to v2.4.0");
# }
# #[cfg(not(feature = "context"))]
# fn main() {}
```

With the `context` feature enabled (the default):

```text
error: TLS certificate has expired
context:
    - fetch https://updates.example.com/manifest.json
    - prepare application update to v2.4.0
```

The headline identifies the problem immediately: an expired certificate. The context then identifies the request and the application operation that encountered it.

## Why Traditional Enum Errors Scale Poorly

Traditional enum-based error handling breaks down as soon as you compose functions, because each new layer of composition demands its own enum.

### The Problem

Suppose three low-level functions each return a precise error enum. The function
bodies below are placeholders; the example is compiled but not executed:

```rust,no_run
# #[cfg(feature = "alloc")]
# fn main() {
use std::io;
use std::num::ParseIntError;
use std::net::AddrParseError;

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Format(String),
}
fn read_file() -> Result<String, ReadError> { todo!() }

#[derive(Debug)]
pub enum ParseError {
    InvalidInt(ParseIntError),
    MissingKey(String),
}
fn parse_config(_data: &str) -> Result<u16, ParseError> { todo!() }

#[derive(Debug)]
pub enum NetworkError {
    BadAddr(AddrParseError),
    Io(io::Error),
}
fn open_socket(_port: u16) -> Result<(), NetworkError> { todo!() }
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

To chain them in `initialize_system`, a precise return type requires a *fourth* enum wrapping the other three — plus a `From` impl for each, just to make `?` work:

```rust
# use std::io;
# use std::num::ParseIntError;
# use std::net::AddrParseError;
#
# #[derive(Debug)]
# pub enum ReadError {
#     Io(io::Error),
#     Format(String),
# }
# fn read_file() -> Result<String, ReadError> { Ok("8080".to_owned()) }
#
# #[derive(Debug)]
# pub enum ParseError {
#     InvalidInt(ParseIntError),
#     MissingKey(String),
# }
# fn parse_config(data: &str) -> Result<u16, ParseError> { data.parse().map_err(ParseError::InvalidInt) }
#
# #[derive(Debug)]
# pub enum NetworkError {
#     BadAddr(AddrParseError),
#     Io(io::Error),
# }
# fn open_socket(_port: u16) -> Result<(), NetworkError> { Ok(()) }
#[derive(Debug)]
pub enum InitError {
    Read(ReadError),
    Parse(ParseError),
    Net(NetworkError),
}

impl From<ReadError> for InitError {
    fn from(e: ReadError) -> Self { InitError::Read(e) }
}
impl From<ParseError> for InitError {
    fn from(e: ParseError) -> Self { InitError::Parse(e) }
}
impl From<NetworkError> for InitError {
    fn from(e: NetworkError) -> Self { InitError::Net(e) }
}

fn initialize_system() -> Result<(), InitError> {
    let data = read_file()?;
    let port = parse_config(&data)?;
    open_socket(port)?;
    Ok(())
}
# assert!(initialize_system().is_ok());
```

Note that `io::Error` is now buried two levels deep in two different places (`InitError::Read(ReadError::Io(_))` and `InitError::Net(NetworkError::Io(_))`), so callers who just want to handle IO errors have to match both paths:

```rust
# use std::io;
# use std::num::ParseIntError;
# use std::net::AddrParseError;
#
# #[derive(Debug)]
# pub enum ReadError {
#     Io(io::Error),
#     Format(String),
# }
# fn read_file() -> Result<String, ReadError> { Ok("8080".to_owned()) }
#
# #[derive(Debug)]
# pub enum ParseError {
#     InvalidInt(ParseIntError),
#     MissingKey(String),
# }
# fn parse_config(data: &str) -> Result<u16, ParseError> { data.parse().map_err(ParseError::InvalidInt) }
#
# #[derive(Debug)]
# pub enum NetworkError {
#     BadAddr(AddrParseError),
#     Io(io::Error),
# }
# fn open_socket(_port: u16) -> Result<(), NetworkError> { Ok(()) }
# #[derive(Debug)]
# pub enum InitError {
#     Read(ReadError),
#     Parse(ParseError),
#     Net(NetworkError),
# }
#
# impl From<ReadError> for InitError {
#     fn from(e: ReadError) -> Self { InitError::Read(e) }
# }
# impl From<ParseError> for InitError {
#     fn from(e: ParseError) -> Self { InitError::Parse(e) }
# }
# impl From<NetworkError> for InitError {
#     fn from(e: NetworkError) -> Self { InitError::Net(e) }
# }
#
# fn initialize_system() -> Result<(), InitError> {
#     let data = read_file()?;
#     let port = parse_config(&data)?;
#     open_socket(port)?;
#     Ok(())
# }
match initialize_system() {
    Ok(()) => println!("Success!"),
    Err(InitError::Read(ReadError::Io(_))) => { /* handle */ }
    Err(InitError::Net(NetworkError::Io(_))) => { /* handle */ }
    Err(other) => println!("Some other error occurred: {:?}", other),
}
```

Add a fourth step that returns a `DatabaseError` and the cycle repeats: a new enum, new `From` impls, and every downstream `match` needs updating.

### Why Crates Give Up

Faced with this growth, most crates abandon precision entirely and adopt one monolithic, crate-wide error enum:

```rust
# use std::{io, num::ParseIntError, net::AddrParseError};
#[derive(Debug, thiserror::Error)]
pub enum CrateError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Parse error: {0}")]
    Parse(#[from] ParseIntError),
    #[error("Network error: {0}")]
    Network(#[from] AddrParseError),
    #[error("Custom error: {0}")]
    General(String),
}
# let error = CrateError::from(io::Error::from(io::ErrorKind::PermissionDenied));
# assert!(matches!(error, CrateError::Io(_)));
# assert_eq!(CrateError::General("failed".to_owned()).to_string(), "Custom error: failed");
```

This kills the boilerplate, but it also kills accuracy: every function now claims it can return *any* crate error, even when most are impossible for that particular call path. `parse_config`'s caller has to account for a `NetworkError` that can never actually occur.

### How `ErrorUnion` Avoids This

`ErrorUnion` sidesteps the dilemma entirely. No new enum is needed to combine errors, so precision and ergonomics stop being a trade-off.

```rust
# #[cfg(feature = "alloc")]
# fn main() {
# use eros::IntoUnion;
# use std::{io, num::ParseIntError, net::AddrParseError};
# fn read_file() -> Result<String, io::Error> { Ok("8080".to_owned()) }
# fn parse_config(data: &str) -> Result<u16, ParseIntError> { data.parse() }
# fn open_socket(_port: u16) -> Result<(), AddrParseError> { Ok(()) }
type MyError = (io::Error, ParseIntError, AddrParseError);

fn initialize_system() -> eros::Result<(), MyError> {
    let data = read_file().union()?;
    let port = parse_config(&data).union()?;
    open_socket(port).union()?;
    Ok(())
}
# assert!(initialize_system().is_ok());
# }
# #[cfg(not(feature = "alloc"))]
# fn main() {}
```

The signature stays exact — only the errors that can actually occur are listed — and adding a fourth fallible step just means adding one type to the tuple, not a new enum and a new set of `From` impls, or a rewritten `match`.

## no_std

Eros supports `#![no_std]` with or without a global allocator. Disable default
features to use and enable the other features such as `alloc` as needed.

### Without an allocator

Each concrete error must implement `core::error::Error + Send + Sync + 'static`
and fit in the size and alignment of `usize`. Zero-sized errors support
arbitrary alignment. Only the active error must satisfy these storage limits;
other types may still appear in its type set. The handle occupies two words,
or three with `error_location` enabled. Enums, numeric error codes, and thin
references to static errors can be stored without allocation.

```rust
use core::{error::Error, fmt};
use eros::{ErrorUnion, Erase, IntoUnion, ReshapeUnion};

#[derive(Debug)]
struct Timeout;
impl fmt::Display for Timeout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("operation timed out")
    }
}
impl Error for Timeout {}

fn poll(ready: bool) -> eros::Result<(), (Timeout,)> {
    eros::ensure!(ready, Timeout);
    Ok(())
}

let error = poll(false).erase().unwrap_err();
assert_eq!(error.narrow::<Timeout, _>().unwrap().to_string(), "operation timed out");
let recovered = poll(false).recover::<Timeout, _>(|_| ());
assert!(recovered.is_ok());
```

### Static message errors

Literal messages in `error!`, `bail!`, and `ensure!` work without `alloc`. Each
literal expands to an ad hoc zero-sized error whose `Display` and `Debug` write
the static text. There is no string pointer in the concrete error itself.

```rust
fn validate(ready: bool) -> eros::Result<()> {
    eros::ensure!(ready, "device is not ready");
    eros::bail!("operation failed");
}

assert_eq!(validate(false).unwrap_err().to_string(), "device is not ready");
assert_eq!(validate(true).unwrap_err().to_string(), "operation failed");
let message = eros::error!("unexpected {{status}}");
assert_eq!(message.to_string(), "unexpected {status}");
```

Escape literal braces as `{{` and `}}`, as with formatting macros. Without
`alloc`, literal messages have anonymous concrete types and use erased error
sets; they cannot be downcast to `MsgError`. Use named error types for typed
`bail!` and `ensure!` results. With `alloc`, literals retain their `MsgError` type.

### What requires `alloc`

Errors exceeding one word in size or alignment require `alloc`. A direct
`&'static str` contains two words and exceeds the payload limit, while a thin
reference to a static error or `MsgError::from_static_ref` fits.

Numeric error codes work without `alloc`:

```rust
#[derive(Debug)]
struct Code(usize);
impl core::fmt::Display for Code {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "device status {}", self.0)
    }
}
impl core::error::Error for Code {}

let error: eros::ErrorUnion<(Code,)> = eros::ErrorUnion::new(Code(7));
assert_eq!(error.into_single().0, 7);
```

Formatted message macros require `alloc`, including implicit format captures.
Custom errors can format their stored fields into a formatter without allocating.
These message macro examples are rejected with `alloc` disabled:

```rust,ignore
fn operation(code: u8) -> eros::Result<()> {
    eros::bail!("device status {code}"); // Implicit capture needs formatting.
}

let _ = eros::error!("device status {}", 7); // Explicit arguments need formatting.
```

### Mapping without allocation

`map_inner` works on both unions and results without allocation. Its callback
owns the original error through `InnerError` and returns a replacement fitting
the inline payload word. Every error goes through the callback, including an
existing instance of the replacement type. The original capture location is
preserved.

```rust
use eros::ReshapeUnion;
use core::fmt;

// The owner can inspect the original concrete error to choose a domain error.
let result: eros::Result<()> = Err(eros::error!("device unavailable"));
let classified: eros::Result<(), (fmt::Error,)> =
    result.map_inner(|_| fmt::Error);
assert!(classified.unwrap_err().is_inner::<fmt::Error>());
```

Callbacks declared to take `InnerError` continue to receive it when a downstream
crate enables `alloc`. Its `is::<T>()`, `downcast_ref::<T>()`, `downcast_mut::<T>()`, and
`downcast::<T>()` methods inspect or extract the original concrete error using
its actual storage. The type remains the same whether stored inline or on the
heap. Failed owned downcasts return the original owner.

Extracting a concrete error allows a wrapper such as `DomainError(Code(7))`
to fit the inline limit. Retaining the erased `InnerError` itself requires more
than one payload word, so a replacement containing it requires `alloc`. With
`alloc`, `inner.into_boxed()` can also keep the original as a boxed source.

### With an allocator

Enable `alloc` to use arbitrary-sized errors and owned or runtime-formatted
messages in a `no_std` environment with a global allocator:

With root diagnostics enabled, a boxed fat pointer owns one allocation containing
the diagnostics and trailing payload. Otherwise, small roots stay inline and
larger roots use `Box<T>`. The union handle always occupies two words with `alloc`. Allocation behavior and diagnostic costs are described in
[Reducing Error Handling Overhead](#reducing-error-handling-overhead).

The `error_location` feature works with or without `alloc` and adds one location
reference per error without allocating. `context_location` adds one location
reference per context frame when `context` is enabled; it does not enable
`context` or `alloc` itself. Both support `no_std`. The `location` feature enables
both flags. The `std`, `context`, and `json` features enable
`alloc` automatically. `backtrace`, `error_span_trace`, and `context_span_trace`
require `std`. With `context` disabled,
static string contexts remain accepted as no-ops and lazy context closures are
not evaluated. Default features enable `alloc`, context, and backtraces, so keep
them disabled for an environment without an allocator.

## Special Thanks

Special thank you to the authors and contributors of the following crates that inspired `eros`:
- [anyhow](https://github.com/dtolnay/anyhow)
- [terrors](https://github.com/komora-io/terrors)
- [error_set](https://github.com/mcmah309/error_set)
- [thiserror](https://github.com/dtolnay/thiserror)
- [tracing](https://github.com/tokio-rs/tracing)
- [color-eyre](https://github.com/eyre-rs/eyre/tree/master/color-eyre)
- [err_trail](https://github.com/mcmah309/err_trail)
