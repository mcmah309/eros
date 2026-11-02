<!-- Generated from rs/eros/README.src.md by scripts/update-readme.sh. Edit the source, then run the script. -->

# eros

[<img alt="github" src="https://img.shields.io/badge/github-mcmah309/eros-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/mcmah309/eros) [<img alt="crates.io" src="https://img.shields.io/crates/v/eros.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/eros) [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-eros-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/eros) [<img alt="test status" src="https://img.shields.io/github/actions/workflow/status/mcmah309/eros/ci.yml?branch=master&style=for-the-badge" height="20">](https://github.com/mcmah309/eros/actions/workflows/ci.yml) [<img alt="no_std" src="https://img.shields.io/badge/no__std-compatible-success?style=for-the-badge" height="20">](#no_std)

**Typed or untyped errors, with ergonomic propagation and no enum boilerplate.** Eros lets each API specify its possible error types or leave them untyped. Callers can handle specific error types and attach context as errors move through the call stack. Both approaches work in libraries and applications.

Choose the signature that fits your API:

```rust
// Propagate errors without their types.
fn load_config() -> eros::Result<Config>

// Propagate errors with their types.
fn load_config() -> eros::Result<Config, (io::Error, ParseError)>
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
```

### No Boilerplate

There should be no boilerplate needed when handling any number of errors (typed or untyped). This is where the magic of [ErrorUnion](#errorunion) happens.

```rust
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
```
The above code is precisely typed for what we care about and there was no need to create an error enum for each case. One can also use a type alias like `type MyError = (io::Error, ParseIntError);` for reuse. See the [ErrorUnion](#errorunion) section for more details how it works.

### Seamless Transitions Between Error Types

Users should be able to seamlessly transition to and from fully typed errors and handle any cases they care about. Use typed or untyped errors wherever each makes sense.

#### Fully Typed

```rust
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
```

`recover` removes handled error types from the result's error set. Handle a group with `.recover::<(io::Error, ParseIntError), _>(|_| 8080)`. Successful values and unhandled errors pass through unchanged.

For manual branching, `narrow::<E, _>()` extracts `E` and discards its diagnostics. Use `narrow::<(E,), _>()` or `narrow::<(A, B), _>()` to retain diagnostics in a smaller union. The unmatched remainder always retains its diagnostics.

#### Untyped

Untyped APIs can still support typed recovery. For example, use a default when a configuration file is missing, while propagating permission and parse errors:

```rust
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
```

The handler receives a typed error with its diagnostics. Other errors remain `AnyError`. You can also use `narrow` to select a concrete error or typed group from an untyped union or result.

Use `widen` to add possible error types:

```rust
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
```

When callers no longer need the error set, `?` converts typed errors into `eros::Result<T>` automatically. Use `.erase()` with `use eros::Erase;` to convert a whole result or union. Both preserve the concrete error and its diagnostics, so callers can still downcast or recover specific types.

### Classifying Errors

Use `classify` to wrap errors in a type callers care about. Here, loader errors become `PortConfigError`, while `DynamicPortNotAllowed` is handled separately:

```rust
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

```

`classify(f)` is shorthand for `map_inner(f).widen()`. The callback receives an owned `InnerError` for every error, even one already of the replacement type. The destination error set is inferred. Use `map_inner` alone for a single-type set, or `narrow` first to transform only selected types. Saved diagnostics are preserved; the original error remains a source only if the callback keeps it, as above.

Wrappers also distinguish errors of the same type from different operations. For example, classify configuration and database I/O errors as `PortConfigError` and `DatabaseError` before combining them with `widen`. Recovering one then leaves the other untouched.

### Errors Have Context

Errors should always provide context of the operations in the call stack that led to the error. Users can add context with `.context` or `.with_context`. Errors also capture a `Backtrace`.

For code already instrumented with `tracing`, [span traces](#tracing-span-traces) can capture operation context automatically.

```rust
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

### Optimizations

Eros comes with the `context` and `backtrace` feature flags enabled by default. Disabling them removes backtrace and context tracking from `ErrorUnion<T>`, and all context methods become a no-op. Thus, it may be optimized away by the compiler.

`ErrorUnion` occupies two pointer-sized words. The number of possible error types does not affect its size.

| Configuration | Error storage |
| --- | --- |
| No root diagnostics, error fits one word | Inline, no allocation |
| No root diagnostics, larger error | One `Box<T>`; requires `alloc` |
| Root diagnostics enabled with `alloc` | Error and diagnostics share one allocation |

Root diagnostics are `backtrace`, `context`, `error_span_trace`, and `error_location`. Traces and context contents may allocate separately. Without `alloc`, errors must fit the size and alignment of `usize`; zero-sized errors may have any alignment.

Widening, narrowing, erasing, and typed extraction do not allocate. `into_inner()` returns an owned `InnerError` and discards diagnostics. Its `.into_boxed()` method reuses an existing `Box<T>` where possible, allocating when moving an inline nonzero error or separating an error from its diagnostics.

See the [Use In Libraries](#use-in-libraries) section as well.

## `ErrorUnion`

### Open Sum Type

`ErrorUnion` is an open sum type. An open sum type takes full advantage of rust's powerful type system. It differs from an enum in that you do not need to define any actual new type in order to hold some specific combination of variants, but rather you simply describe the ErrorUnion as holding one value out of several specific possibilities. This is declared by using a tuple of those possible variants as the generic parameter for the `ErrorUnion`. 

For example, a `ErrorUnion<(io::Error, fmt::Error)>` contains either a `io::Error` or a `fmt::Error`. The benefit of this over creating specific enums for each function becomes apparent in larger codebases where error handling needs to occur in different places for different errors. As such, `ErrorUnion` allows you to quickly specify a function's return value as involving a precise subset of errors that the caller can clearly reason about. This Provides maximum composability with no boilerplate. E.g.

```rust
use eros::ErrorUnion;
use std::{fmt, io};

fn main() {
    let error: ErrorUnion<MyError>;
    let result: eros::Result<(), MyError>;
}

type MyError = (fmt::Error, io::Error); // A type alias can be used to make statements more concise
```
vs
```rust
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
```
Additionally, the complexity of the second option grows exponentially the more error enums have to be combined from different functions. That is why a lot of crates opt for not precisely defining errors for APIs and instead choose a single error enum or struct for the entire crate. See the [Why Traditional Enum Errors Scale Poorly](#why-traditional-enum-errors-scale-poorly) section for a deeper dive into this.

When the error union should encompass the full set of possible errors, use `AnyError`:

`eros::Result<()>` is shorthand for `eros::Result<(), AnyError>`, which in turn is shorthand for `Result<(), ErrorUnion<AnyError>>`.

### Tracing

`ErrorUnion` also allows adding context to an error throughout the callstack with the `context` or `with_context` methods. This context may be information such as variable values or ongoing operations while the error occurred. If the error is handled higher in the stack, then this can be disregarded (no log pollution). Otherwise you can log it (or panic), capturing all the relevant information in one log. A backtrace is captured and included with the error when `RUST_BACKTRACE` is set.

## Context Macro

For some functions, one may want to attach the same context to every error that can be returned from that function. Writing `.with_context(...)` on each fallible call quickly becomes repetitive and can obscure the intent of the function. For example:

```rust
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
```

To help with this, `eros` provides the `context` attribute macro. The macro wraps the function and automatically adds the supplied context to any error returned from it:

```rust
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
```

Whether reading the file or parsing its contents fails, the error includes `Failed to load server port from config/port.txt` alongside the original I/O or parse error. The macro attaches this context once, while keeping the function body focused on the actual logic.

### Automatic Context from Parameters

When the context simply consists of "which arguments was this function called with?", the format string can often be inferred automatically.

Instead of providing an explicit format string, use `#[context]` and annotate the parameters that should appear in the generated context with `#[fmt(...)]` where `...` is the desired formatting:

```rust
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
```

The generated context is equivalent to:

```rust
format!(
    "name: {}\nflags: {:?}\n",
    name,
    flags,
)
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

Contexts appear innermost first, and empty sections are omitted. Formatting uses saved diagnostics; it does not capture new ones.

Debug groups identical reports and lists their zero-based positions (`0..3` means indices 0, 1, and 2):

```rust
use eros::{ErrorUnion, ErrorUnionVec};

let errors: ErrorUnionVec<(std::io::Error,)> = [
    "invalid input", "invalid input", "invalid input",
    "permission denied", "invalid input", "invalid input",
]
.into_iter()
.map(|message| ErrorUnion::new(std::io::Error::other(message)))
.collect();

println!("{errors:#?}");
```

```text
errors (6 total):
    - error: invalid input
      at: [0..3, 4..6]
    - error: permission denied
      at: [3]
```

Reports with different visible diagnostics form separate groups.

See [Logging](#logging) for tracing fields and [Structured JSON](#structured-json) for JSON reports.

## Best Practices

### Use In Libraries

*Libraries should consider setting `default-features = false` on their `eros` dependency*, allowing downstream crates to choose diagnostics. Keep `features = ["alloc"]` if errors exceed one word in size or alignment. Enable diagnostics separately for the library's own tests.

#### Public APIs

##### Approach A: Exposing `ErrorUnion` in Public APIs

Exposing `ErrorUnion` lets multiple crates compose errors together, especially for internal APIs. Choose types callers care about: a crate-specific error may be more useful than an underlying `io::Error`.

##### Approach B: Replacing `ErrorUnion` With Concrete Crate Errors

To expose a concrete crate error, use `into_inner()` at the API boundary and box the underlying error as a source:

<details>

<summary>Example Implementation</summary>

```rust
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
```

</details>

The library uses `ErrorUnion` internally while callers see `CrateError`. `into_inner()` discards saved diagnostics and returns the concrete error inside an owned `InnerError`; `.into_boxed()` boxes it for the wrapper.

For typed tuples, the [error enum macro](#error-enum-macro) generates a concrete error enum for the public API. Convert at the boundary with `map_err(Into::into)`:

<details>

<summary>Example Implementation</summary>

```rust
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
```

</details>

Callers see a traditional enum while internal functions keep precise error sets. Typed unions can also use `into_inner()` if a single opaque crate error is preferred.

### Context Placement: Two Approaches

Context can describe each function or be attached where information would otherwise be lost.

#### Approach 1: Attach Context at the Function

A function uses `#[context]` to describe its operation and inputs. Every caller gets the same context automatically:

```rust
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
```

**Why:** it removes ambiguity about whose job it is to attach context. If every function attaches context describing its own operation and inputs, nothing needs to be re-derived or duplicated by callers, and nothing is silently dropped because every caller assumed some other layer would handle it.

**Tradeoff:** if a function's parameter is itself just forwarded from its caller (and that caller already attaches it too), the same value can appear in context more than once as the error bubbles up. Note if the `context` feature is disabled `with_context` becomes a no-op, so the cost is avoidable.

#### Approach 2: Attach Context Where Information Would Be Lost

Attach context at the last caller that has the relevant information. Here, `func2` forwards `param`, so `func1` and `func1b` attach it before returning:

```rust
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
```

**Why:** it keeps individual error messages lean, avoids restating the same value at every layer, and sidesteps the classic `connection failed: connection failed: connection failed: no route to host` style of redundant, nested context.

**Tradeoff:** because no single function is locally responsible for attaching a given piece of context, it takes discipline and more time has to be spent at each call-site — you have to ask "would this information otherwise be lost going up the stack from here?" If a function is called from many places, that question has to be answered (and the same context written) at each call-site rather than once at the function's own definition. It's also easy to accidentally end up restating context slightly differently at two call-sites — in the example above, `func1` and `func1b` phrase the same underlying fact as `"Failed to do some action. param was {}"` and `"Some action failed with param {}"` — since nothing enforces a single canonical phrasing the way `#[context]` on the function itself does. It is also easy to get lazy and skip attaching context altogether at a given call-site, silently losing information that would have been captured automatically under Approach 1.

### Backtrace vs Location

`backtrace` is enabled by default and captures the call stack when the backtrace environment settings request it. [Location tracking](#location) is lighter, works in wasm and `no_std`, and can be used alongside backtraces and span traces.

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

`error_span_trace` captures the current [`tracing`](https://docs.rs/tracing) span and its parents when an error first enters Eros, including fields and locations. `context_span_trace` captures spans at context attachments; enable `context` separately. `span_trace` enables both trace features. All require `std`.

```rust
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

Convert errors into Eros before leaving the instrumented function so its span is still active. Print traces with `{:?}` or inspect them through `.span_trace()` on the error or its context frames. `{}` and `{:#?}` omit traces.

### Filtered Backtrace

Enable `filtered_backtrace` to hide recognized dependency and runtime frames in formatted output. The original trace remains available through `.backtrace()`.

### Composing Type Sets

Use `type_set!` to share members between named error sets. Join sets and tuples with `||`; the macro emits flat tuple aliases and includes repeated members once:

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

Named sets must be declared in the same invocation. Members keep their first occurrence order; use the same type paths so duplicates are recognized. Sets support up to 26 distinct members and cannot have generic parameters.

Attributes and visibility apply to the emitted aliases. Add `#[eros::error_enum(Name)]` to generate an enum from a composed set. `#[cfg(...)]` gates the alias, but its members still participate in composition.

### Error Enum Macro

`#[eros::error_enum(Name)]` keeps a tuple alias and generates an owned enum with `Debug`, `Display`, and `Error` implementations, useful for matching errors or [exposing a concrete public error](#approach-b-replacing-errorunion-with-concrete-crate-errors). Add `error_enum_ref` and `error_enum_mut` for borrowed views, or `error_enum_kind` for unit variants without payloads:

```rust
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
```

Each macro works independently. Attributes below it apply to that enum until the next enum macro or the alias. Variant names join path segments in PascalCase and strip a trailing `Error` unless that would leave an empty name. The kind enum implements `Debug`; attach other derives explicitly.

`Display` delegates to the contained error, and `Error::source()` returns it. An optional format, such as `#[eros::error_enum(Error, "operation failed: {0}")]`, customizes the message.

Conversions accept subsets and reordered tuples. Borrowing preserves diagnostics; owned conversion discards them. For erased unions (`ErrorUnion<AnyError>`), use `TryFrom`, which returns the original value or borrow on a mismatch.

### Adding Source Chains

Use `map_inner` to change the main error while keeping the original failure as its source. The closure receives an owned `InnerError`; convert it into a box to store it and expose it through `Error::source()`:

```rust
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
    println!("User message:");
    println!("{}", build_user_error_message(&password_error));
    let system_error = load_configuration().unwrap_err();
    println!("\nUser message:");
    println!("{}", build_user_error_message(&system_error));
}
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
let error = anyhow::anyhow!("TLS certificate has expired")
    .context("fetch https://updates.example.com/manifest.json")
    .context("prepare application update to v2.4.0");

println!("{error:?}"); // Debug shows the main message and its causes.
```

```text
prepare application update to v2.4.0

Caused by:
    0: fetch https://updates.example.com/manifest.json
    1: TLS certificate has expired
```

`eros` keeps the original error as the main message and presents context separately, in the order it was added. As the error travels up the call stack, context follows stack trace order: nearest the failure first, then outward through its callers:

```rust
let error = eros::error!("TLS certificate has expired")
    .context("fetch https://updates.example.com/manifest.json")
    .context("prepare application update to v2.4.0");

println!("{error:#?}"); // Alternate Debug omits locations, backtraces, and span traces.
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

Suppose three low-level functions each return a precise error enum. The function bodies below are placeholders; the example is compiled but not executed:

```rust,no_run
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
```

To chain them in `initialize_system`, a precise return type requires a *fourth* enum wrapping the other three — plus a `From` impl for each, just to make `?` work:

```rust
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
```

Note that `io::Error` is now buried two levels deep in two different places (`InitError::Read(ReadError::Io(_))` and `InitError::Net(NetworkError::Io(_))`), so callers who just want to handle IO errors have to match both paths:

```rust
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
```

This kills the boilerplate, but it also kills accuracy: every function now claims it can return *any* crate error, even when most are impossible for that particular call path. `parse_config`'s caller has to account for a `NetworkError` that can never actually occur.

### How `ErrorUnion` Avoids This

`ErrorUnion` sidesteps the dilemma entirely. No new enum is needed to combine errors, so precision and ergonomics stop being a trade-off.

```rust
type MyError = (io::Error, ParseIntError, AddrParseError);

fn initialize_system() -> eros::Result<(), MyError> {
    let data = read_file().union()?;
    let port = parse_config(&data).union()?;
    open_socket(port).union()?;
    Ok(())
}
```

The signature stays exact — only the errors that can actually occur are listed — and adding a fourth fallible step just means adding one type to the tuple, not a new enum and a new set of `From` impls, or a rewritten `match`.

## no_std

Eros supports `#![no_std]` with or without a global allocator. Disable default features, then enable `alloc` and other features as needed.

### Without an allocator

Errors must implement `core::error::Error + Send + Sync + 'static` and fit the size and alignment of `usize`. Zero-sized errors may have any alignment. These limits apply to the stored error, not every type in its error set. Typed recovery, erasure, and downcasting still work.

### Inline errors

`#[eros::inline_error]` generates error implementations for nongeneric structs and enums, checking the size and alignment limits at compile time, even with `alloc` enabled:

```rust
#[eros::inline_error("operation timed out")]
pub struct Timeout;

#[eros::inline_error("device code {code:04x}", code = self.0)]
pub struct DeviceFailure(pub usize);

#[eros::inline_error("{self:?}")]
#[repr(u8)]
pub enum DeviceStatus { Disconnected, InvalidCode(u16) }

let erased = eros::error!(DeviceFailure(7));
assert_eq!(erased.to_string(), "device code 0007");
assert_eq!(erased.downcast_inner::<DeviceFailure>().unwrap().0, 7);
```

The macro generates `Debug`, `Error`, and `Display`. Messages use `format!` syntax without allocating; `"{self:?}"` uses debug formatting. Omit the message to implement `Display` yourself. Add derives such as `Clone` or `Copy` explicitly. Actual allocation depends on the [enabled diagnostics](#optimizations).

### Static message errors

Literal messages in `error!`, `bail!`, and `ensure!` work without `alloc`, producing anonymous zero-sized errors:

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

Escape braces as `{{` and `}}`. Without `alloc`, these messages use erased error sets and cannot be downcast to `MsgError`; use named errors for typed results. Runtime formatting, including `bail!("status {code}")`, requires `alloc`.

### Mapping without allocation

`map_inner` and `classify` work without allocation when the replacement fits the inline limits. Their callbacks receive an owned `InnerError`, which supports `is`, `downcast_ref`, `downcast_mut`, and `downcast`. The original capture location is preserved.

Extract a concrete error to keep it in an inline wrapper. Storing the whole `InnerError` exceeds the one-word limit and requires `alloc`.

### With an allocator

Enable `alloc` for larger errors and owned or runtime-formatted messages. A global allocator is required; see [Optimizations](#optimizations) for storage costs. `&'static str` is two words and requires `alloc`, while thin references to static errors fit inline.

`context` and `json` enable `alloc` automatically. Location tracking works without `std`; `backtrace` and span traces require `std`. With `context` disabled, static string contexts are accepted as no-ops and lazy context closures are not evaluated.

## Special Thanks

Special thank you to the authors and contributors of the following crates that inspired `eros`:
- [anyhow](https://github.com/dtolnay/anyhow)
- [terrors](https://github.com/komora-io/terrors)
- [error_set](https://github.com/mcmah309/error_set)
- [thiserror](https://github.com/dtolnay/thiserror)
- [tracing](https://github.com/tokio-rs/tracing)
- [color-eyre](https://github.com/eyre-rs/eyre/tree/master/color-eyre)
- [err_trail](https://github.com/mcmah309/err_trail)
