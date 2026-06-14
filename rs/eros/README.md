# eros

[<img alt="github" src="https://img.shields.io/badge/github-mcmah309/eros-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/mcmah309/eros)
[<img alt="crates.io" src="https://img.shields.io/crates/v/eros.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/eros)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-eros-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/eros)
[<img alt="test status" src="https://img.shields.io/github/actions/workflow/status/mcmah309/eros/ci.yml?branch=master&style=for-the-badge" height="20">](https://github.com/mcmah309/eros/actions/workflows/ci.yml)

Eros is the swiss army knife of error handling approaches. It fits perfectly well into libraries and binaries.

Built on the following philosophy:
1. Error types only matter when the caller cares about the type, otherwise this just hinders ergonomics and creates unnecessary noise. [Link](#optional-typed-errors)
2. There should be no boilerplate needed when handling any number of errors - no need to create an error enum for each case. [Link](#no-boilerplate)
3. Users should be able to seamlessly transition to and from fully typed errors. And handle any cases they care about. [Link](#seamless-transitions-between-error-types)
4. Errors should always provided context of the operations in the call stack that lead to the error. [Link](#errors-have-context)
5. Error constructs should performant. [Link](#optimizations)

## Philosophy In Action

### Optional Typed Errors

Error types only matter when the caller cares about the type, otherwise this just hinders ergonomics and creates unnecessary noise. Thus, it should be easy for the developer to make the type opaque for developing fast composable apis. This is where [ErrorUnion](#errorunion) helps.

```rust
use eros::bail;

// Error type is untracked
fn eros_result() -> eros::Result<()> {
    // `bail!` creates an ad hoc untyped error. Later one can get the underlying error if wanted.
    bail!("Something went wrong")
}

fn normal_result() -> Result<(), std::io::Error> {
    Err(std::io::Error::new(
        std::io::ErrorKind::AddrInUse,
        "message here",
    ))
}

// Easily convert normal `Result` to an `eros::Result`
fn using_normal_and_eros_results() -> eros::Result<()> {
    let val = normal_result()?;
    let val = eros_result()?;
    Ok(val)
}

fn main() {
    eros_result();
    normal_result();
    using_normal_and_eros_results();
}
```

### No Boilerplate

There should be no boilerplate needed when handling any number of errors (typed or untyped). This is where the magic of [ErrorUnion](#errorunion) happens.


```rust
use eros::{IntoUnion, bail};
use std::{io, sync};

fn regular_typed_result1() -> Result<(), io::Error> {
    return Err(io::Error::new(io::ErrorKind::AddrInUse, "message here"));
}

fn regular_typed_result2() -> Result<(), sync::mpsc::RecvError> {
    return Err(sync::mpsc::RecvError);
}

// `ErrorUnion` is used to track each possible error type,
// instead of creating a enum for each possible error variant.
// `eros::Result<_,(..)>` == `Result<_,ErrorUnion<(..)>>`.
fn error_union_result() -> eros::Result<(), (io::Error, sync::mpsc::RecvError)> {
    let val = regular_typed_result1().into_union()?;
    let val = regular_typed_result2().into_union()?;
    Ok(val)
}

fn main() {
    error_union_result();
}
```
The above code is precisely typed for what we care about and there was no need to create an error enum for each case. See the [ErrorUnion](#errorunion) section for more details how it works.

### Seamless Transitions Between Error Types

Users should be able to seamlessly transition to and from fully typed errors. And handle any cases they care about.

```rust
use eros::{IntoUnion, ReshapeUnion};
use std::{io, sync};

fn regular_typed_result1() -> Result<(), sync::mpsc::RecvError> {
    return Err(sync::mpsc::RecvError);
}

fn regular_typed_result2() -> Result<(), io::Error> {
    return Err(io::Error::new(io::ErrorKind::AddrInUse, "message here"));
}

fn error_union_result() -> eros::Result<(), (io::Error, sync::mpsc::RecvError)> {
    let val = regular_typed_result1().into_union()?;
    let val = regular_typed_result2().into_union()?;
    Ok(val)
}

// Error type is no longer tracked, we handled internally.
fn regular_result() -> Result<(), sync::mpsc::RecvError> {
    // Narrow the `ErrorUnion` and handle to only handle `io::Error` case!
    match error_union_result().narrow::<io::Error, _>() {
        Ok(io_error) => {
            // This statement is not needed, just to show the type explicitly for this example.
            let _: io::Error = io_error;
            todo!()
        }
        // The error type of the Result has been narrowed.
        // It is now a union with a single type (`ErrorUnion<(sync::mpsc::RecvError,)>`),
        // thus we can convert into the inner traced type.
        Err(result) => {
            // This statement is not needed, just to show the type explicitly for this example.
            let result: eros::Result<(), (sync::mpsc::RecvError,)> = result;
            result.map_err(|e| e.into_inner())
        }
    }
}

fn main() {
    regular_result();
}
```

And to expand an `ErrorUnion` just call `widen`

```rust
use eros::{IntoUnion, ReshapeUnion};
use std::{fmt, io, sync};

fn result_union1() -> eros::Result<(), (io::Error, fmt::Error)> {
    Ok(())
}

fn result_union2() -> eros::Result<(), (fmt::Error, sync::mpsc::RecvError)> {
    Ok(())
}

fn result_union3() -> Result<(), sync::mpsc::RecvError> {
    Ok(())
}

fn result_union4() -> eros::Result<(), (io::Error, fmt::Error, sync::mpsc::RecvError)> {
    result_union1().widen()?;
    result_union2().widen()?;
    result_union3().into_union()?;
    Ok(())
}

fn main() {
    result_union4().unwrap();
}
```

### Errors Have Context

Errors should always provided context of the operations in the call stack that lead to the error. Users can add context with `.context` or `.with_context`. Errors also capture a `Backtrace`.

```rust
use eros::{Context, bail};
use std::io;

fn eros_result1() -> eros::Result<()> {
    bail!("Something went wrong")
}

fn eros_result2() -> eros::Result<()> {
    Err(io::Error::new(io::ErrorKind::AddrInUse, "message here")).context("This is some context")?
}

fn adding_more_context() -> eros::Result<()> {
    let val = eros_result1().with_context(|| format!("This is some lazy context"))?;
    let val = eros_result2().context("This is some more context")?;
    Ok(val)
}

fn main() {
    let out = adding_more_context().context("final context");
    println!("{out:#?}");
}
```

```console
Something went wrong
---

Context:
        - This is some lazy context
        - final context

---

Backtrace:
... 4 lines removed for the example
   4:     0x5639a19ef19d - eros::error_union::ErrorUnionInner<dyn eros::error_union::SendSyncError>::new::h2b7d357ac3fa4dc2
                               at /workspaces/eros/rs/eros/src/error_union.rs:74:24
   5:     0x5639a19eeccc - eros::error_union::ErrorUnion::new::h7248c6cf6e9aac05
                               at /workspaces/eros/rs/eros/src/error_union.rs:322:20
   6:     0x5639a19f0cbf - example::eros_result1::h38d1b0e1c5450c1f
                               at /workspaces/eros/rs/eros/tests/example.rs:5:5
   7:     0x5639a19f0d99 - example::adding_more_context::hddf805cbc7e3056c
                               at /workspaces/eros/rs/eros/tests/example.rs:13:15
   8:     0x5639a19f0ec9 - example::main::hf13f943154472682
                               at /workspaces/eros/rs/eros/tests/example.rs:20:15
   9:     0x5639a19efb37 - example::main::{{closure}}::h4cab7eede14f8495
                               at /workspaces/eros/rs/eros/tests/example.rs:19:10
... 21 lines removed for example
...
```
#### Better Backtrace

The previous backtrace in the example was shortened for brevity, thus the "...". For a better backtrace experience while developing, enable the `better_backtrace` feature flag. Resulting in
```console
Something went wrong
---

Context:
        - This is some lazy context
        - final context

---
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ BACKTRACE ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 1: eros::error_union::ErrorUnionInner<dyn eros::error_union::SendSyncError>::new
    at ./src/error_union.rs:74
 2: eros::error_union::ErrorUnion::new
    at ./src/error_union.rs:322
 3: example::eros_result1
    at ./tests/example.rs:5
 4: example::adding_more_context
    at ./tests/example.rs:13
 5: example::main
    at ./tests/example.rs:20
 6: example::main::{{closure}}
    at ./tests/example.rs:19
                              ⋮ 21 frames hidden ⋮   
```
#### Location

The `location` feature flag adds a location at compile time for error creation and each context. This can be used with or in place of `backtrace`, as it is lighter than a full backtrace and can be used in wasm environments (backtraces do not work in wasm environments).

### Optimizations

Eros comes with the `context` and `backtrace` feature flags enabled by default. If this is disabled, backtrace and context tracking are removed from `ErrorUnion<T>` and all context methods become a no-opt. Thus it may be optimized away by the compiler. 

`ErrorUnion`'s stack size is pointer size (uses a `Box`). Boxing errors is a common trick to increase performance and decrease stack memory usage in many cases. This is because boxing may decrease the size of the return type, e.g. `Result<(),Box<u128>>` is smaller than `Result<(),u128>>`.

See the [Use In Libraries](#use-in-libraries) section as well.

## `ErrorUnion`

### Open Sum Type

`ErrorUnion` is an open sum type. An open sum type takes full advantage of rust's powerful type system. It differs from an enum in that you do not need to define any actual new type in order to hold some specific combination of variants, but rather you simply describe the ErrorUnion as holding one value out of several specific possibilities. This is declared by using a tuple of those possible variants as the generic parameter for the `ErrorUnion`. 

For example, a `ErrorUnion<(io::Error, fmt::Error)>` contains either a `io::Error` or a `fmt::Error`. The benefit of this over creating specific enums for each function become apparent in larger codebases where error handling needs to occur in different places for different errors. As such, `ErrorUnion` allows you to quickly specify a function's return value as involving a precise subset of errors that the caller can clearly reason about. Providing maximum composability with no boilerplate. E.g.

```rust
use eros::ErrorUnion;
use std::{fmt, io};

fn main() {
    type MyError = (fmt::Error, io::Error); // Optional type alias
    let error: ErrorUnion<MyError>;
}
```
vs
```rust
use std::{fmt, io};

fn main() {
    let error: CustomError;
}

#[derive(Debug)]
pub enum CustomError {
    FmtError(fmt::Error),
    IoError(io::Error),
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomError::FmtError(e) => write!(fmt, "{}", e),
            CustomError::IoError(e) => write!(fmt, "{}", e),
        }
    }
}

impl std::error::Error for CustomError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CustomError::FmtError(e) => e.source(),
            CustomError::IoError(e) => e.source(),
        }
    }
}

impl From<fmt::Error> for CustomError {
    fn from(error: fmt::Error) -> Self {
        CustomError::FmtError(error)
    }
}

impl From<io::Error> for CustomError {
    fn from(error: io::Error) -> Self {
        CustomError::IoError(error)
    }
}
```
Additionally, the complexity of the second option grow exponentially the more error enums have to be combined from different functions. That is why a lot of crates opt for not precisely defining errors for apis and instead choose a single error enum or struct for the entire crate.

When one wants the error union to encompass the set off all possible error's use `AnyError` -- `eros::Result<()> == eros::Result<(), AnyError> == Result<(), ErrorUnion<AnyError>>`

### Tracing

`ErrorUnion` also allows adding context to an error throughout the callstack with the `context` or `with_context` methods. This context may be information such as variable values or ongoing operations while the error occurred. If the error is handled higher in the stack, then this can be disregarded (no log pollution). Otherwise you can log it (or panic), capturing all the relevant information in one log. A backtrace is captured and added to the log if `RUST_BACKTRACE` is set.

## Context Macro

For some functions, one may want to attach the same context to every error that can be returned from that function. Writing `.with_context(...)` on each fallible call quickly becomes repetitive and can obscure the intent of the function. For example:

```rust
use eros::Context;

fn result1() -> eros::Result<()> {
    eros::bail!("This is an error")
}

fn context_on_each_call(param: &str) -> eros::Result<()> {
    result1().with_context(|| format!("param was {}", param))?;
    result1().with_context(|| format!("param was {}", param))?;
    result1().with_context(|| format!("param was {}", param))?;
    Ok(())
}

fn main() {
    context_on_each_call("value");
}
```

To help with this, `eros` provides the `context` attribute macro. The macro wraps the function and automatically adds the supplied context to any error returned from it:

```rust
use eros::{Context, context};

fn result1() -> eros::Result<()> {
    eros::bail!("This is an error")
}

#[context("param was {}", param)]
fn context_added_once(param: &str) -> eros::Result<()> {
    result1()?;
    result1()?;
    result1()?;
    Ok(())
}

fn main() {
    context_added_once("value");
}
```

This behaves as though each `?` in the function had been followed by the same `.with_context(...)` call, while keeping the function body focused on the actual logic.

### Automatic Context from Parameters

When the context simply consists of "which arguments was this function called with?", the format string can often be inferred automatically.

Instead of providing an explicit format string, use `#[context]` and annotate the parameters that should appear in the generated context with `#[fmt(...)]` where `...` is the desired formatting:

```rust
use eros::{context, Context};

#[derive(Debug)]
struct Flags {
    enabled: bool,
}

fn result1() -> eros::Result<()> {
    eros::bail!("This is an error")
}

#[context]
fn process(
    #[fmt("{}")] name: &str,
    count: usize,
    #[fmt("{:?}")] flags: &Flags,
) -> eros::Result<()> {
    result1()?;
    result1()?;
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

```rust,ignore
format!(
    "name: {}\nflags: {:?}\n",
    name,
    flags,
)
```

Only annotated parameters are included in the generated context. Parameters without `#[display]` or `#[debug]` are ignored, allowing sensitive values or uninteresting arguments to be omitted.

## Misc

### Use In Libraries

`eros`'s flexibility and optimizations make it a the perfect option for both libraries and binaries.

*Libraries should consider disabling default features* and allowing downstream crates to enable this. This can then be enabled for tests only in the library.

#### Suggested Route

Exposing `ErrorUnion` in a public api is perfectly fine and usually preferred. It allows multiple crates to use the power of these constructs together. see the [Optimizations](#optimizations) section for more info. Just make sure to re-export these constructs if exposed.

#### Alternatives

##### Wrapper Error Types

An alternative to exposing `ErrorUnion` is a wrapper type like a new type - `MyErrorType(ErrorUnion)`. If such a route is taken, consider implementing `Deref`/`DerefMut`. That way, a downstream can also add additional context. Additionally/alternatively, consider adding an `into_union` method as a way to to convert to the underlying `ErrorUnion`. That way, if a downstream uses Eros they can get the `ErrorUnion` rather than wrapping it in another `ErrorUnion`. 

The downside is wrapping/nesting `ErrorUnion` may still unintentionally occur, that is why exposing the `ErrorUnion` in the api is usually preferred, since `ErrorUnion` cannot be nested within itself. Additionally the `into_union` api can no longer be used across api boundaries which limits composability.

##### Non-Wrapper Error Types

If one wants to add their own custom error type for all public api's without exposing constructs like `ErrorUnion`, use the `into_inner` method at these boundaries.

<details>

<summary>Example Implementation</summary>

```rust
use eros::{SendSyncError, ErrorUnion};

#[derive(Debug)]
struct MyErrorType(Box<dyn SendSyncError>);

impl std::fmt::Display for MyErrorType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "MyErrorType: {}", self.0)
    }
}

impl std::error::Error for MyErrorType {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

fn internal_api() -> eros::Result<()> {
    Err(ErrorUnion::new(std::io::Error::new(std::io::ErrorKind::Other, "io error")))
}

pub fn public_api() -> Result<(), MyErrorType> {
    // Replace `ErrorUnion` with your custom error type
    internal_api().map_err(|e| MyErrorType(e.into_inner_dyn_error()))
}
```

</details>

### Backtrace vs Location

`eros` has two location tracking feature flags `backtrace`, which captures a backtrace at error creation if `RUST_BACKTRACE` env variable is set, and `location`, which captures the location of the code that the error and context were created from. `location` is more efficient than `backtrace` since the call location is injected at compile time. While backtrace is generally more precise and useful. Both of these can be used together. `location` becomes especially useful for wasm environments where backtraces are not supported. `location` is not enabled by default, while `backtrace` is.

### Anyhow

`eros` comes with an `anyhow` feature flag. This adds a `ErrorUnion::anyhow` function for converting an `anyhow::Error` to an `ErrorUnion`. This can help integrate with legacy code.

## Special Thanks

Special thank you to the authors and contributors of the following crates that inspired `eros`:
- [anyhow](https://github.com/dtolnay/anyhow)
- [terrors](https://github.com/komora-io/terrors)
- [error_set](https://github.com/mcmah309/error_set)
- [thiserror](https://github.com/dtolnay/thiserror)