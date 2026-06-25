# eros

[<img alt="github" src="https://img.shields.io/badge/github-mcmah309/eros-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/mcmah309/eros)
[<img alt="crates.io" src="https://img.shields.io/crates/v/eros.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/eros)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-eros-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/eros)
[<img alt="test status" src="https://img.shields.io/github/actions/workflow/status/mcmah309/eros/ci.yml?branch=master&style=for-the-badge" height="20">](https://github.com/mcmah309/eros/actions/workflows/ci.yml)

Eros is the swiss army knife of error handling approaches. It works well in both libraries and binaries.

Built on the following philosophy:
1. Error types only matter when the caller cares about the type, otherwise this just hinders ergonomics and creates unnecessary noise. [Link](#optional-typed-errors)
2. There should be no boilerplate needed when handling any number of errors - no need to create an error enum for each case. [Link](#no-boilerplate)
3. Users should be able to seamlessly transition to and from fully typed errors and handle any cases they care about. [Link](#seamless-transitions-between-error-types)
4. Errors should always provided context of the operations in the call stack that lead to the error. [Link](#errors-have-context)
5. Error constructs should be performant. [Link](#optimizations)

## Philosophy In Action

### Optional Typed Errors

Error types only matter when the caller cares about the type, otherwise this just hinders ergonomics and creates unnecessary noise. Thus, it should be easy for the developer to make the type opaque for developing fast composable APIs. This is where [ErrorUnion](#errorunion) helps.

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
// instead of creating an enum for each possible error variant.
// `eros::Result<_,(..)>` is shorthand for  `Result<_,ErrorUnion<(..)>>`.
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

Users should be able to seamlessly transition to and from fully typed errors and handle any cases they care about.

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
    // Narrow the `ErrorUnion` and handle the `io::Error` case!
    match error_union_result().narrow::<io::Error, _>() {
        Ok(io_error) => {
            // let _: io::Error = io_error;
            todo!()
        }
        // The error type of the Result has been narrowed.
        // It is now a union with a single type, thus we can convert into the inner traced type.
        Err(result) => {
            // let _: eros::Result<(), (sync::mpsc::RecvError,)> = result;
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

Errors should always provide context of the operations in the call stack that led to the error. Users can add context with `.context` or `.with_context`. Errors also capture a `Backtrace`.

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

Eros comes with the `context` and `backtrace` feature flags enabled by default. If this is disabled, backtrace and context tracking are removed from `ErrorUnion<T>` and all context methods become a no-op. Thus it may be optimized away by the compiler. 

`ErrorUnion`'s stack size is pointer size (uses a `Box`). Boxing errors is a common trick to increase performance and decrease stack memory usage in many cases. This is because boxing may decrease the size of the return type, e.g. `Result<(),Box<u128>>` is smaller than `Result<(),u128>>`.

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

fn result() -> eros::Result<()> {
    eros::bail!("This is an error")
}

#[context("param was {}", param)]
fn context_added_once(param: &str) -> eros::Result<()> {
    result()?;
    result()?;
    result()?;
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

```rust,ignore
format!(
    "name: {}\nflags: {:?}\n",
    name,
    flags,
)
```

Only annotated parameters are included in the generated context. Parameters without `#[fmt(...)]` are ignored, allowing sensitive values or uninteresting arguments to be omitted.

## Logging

Eros provides built-in logging integration via the `logging` feature flag. This enables `log_*` methods on `ErrorUnion` directly, as well as the `LogExt` trait for chaining log calls on `Result`.

```rust,ignore
use eros::{LogExt, bail};

fn eros_result() -> eros::Result<()> {
    bail!("Something went wrong")
}

fn main() {
    // Log directly on ErrorUnion
    if let Err(e) = eros_result() {
        e.log_error();
    }

    // Or chain logging on a Result without consuming it
    let _result = eros_result().log_warn();
}
```
The recommended pattern in practice is to use when consuming an error:
```rust,ignore
use eros::{LogExt, bail};

fn eros_result() -> eros::Result<()> {
    bail!("Something went wrong")
}

fn old_way() {
    if let Err(error) = eros_result().context("Context around function") {
        tracing::error!("{:#?}", error);
    }
}

fn recommended_way() {
    eros_result()
        .context("Context around function")
        .log_error()
        .ok();
}

fn main() {
    old_way();
    recommended_way();
}
```

### Feature Flags

The `logging` feature enables the `log*` methods and `LogExt` trait, but does not wire up a backend. Libraries can enable `logging` and let downstream crates decide on a backend.

To use `tracing` as the backend, enable the `tracing` feature. Additionally, control the format of logged messages with `log_display` (uses `Display`) or `log_debug` (uses `Debug`) feature flags. These are backend-facing flags that libraries should not set.

```toml
[dependencies]
eros = { version = "*", features = ["tracing", "log_debug"] }
```

> Libraries should enable only `logging` and leave `tracing`, `log_debug`, and `log_display` for downstream crates to decide.

## Misc

### Use In Libraries

`eros`'s flexibility and optimizations make it the perfect option for both libraries and binaries.

*Libraries should consider disabling default features* and allowing downstream crates to enable this. This can then be enabled for tests only in the library.

#### Suggested Route

Exposing `ErrorUnion` in a public API is perfectly fine and usually preferred. It allows multiple crates to use the power of these constructs together. see the [Optimizations](#optimizations) section for more info. Just make sure to re-export these constructs if exposed.

#### Alternative

If one wants to add a custom error type for all public APIs without exposing constructs like `ErrorUnion`, use the `into_inner` method at these boundaries. This is a common pattern, since most crates already define their own error type for public-facing APIs.

<details>

<summary>Example Implementation</summary>

```rust
use std::io;

use eros::{SendSyncError, ErrorUnion, bail};

#[derive(Debug)]
struct CrateError(Box<dyn SendSyncError>);

impl std::fmt::Display for CrateError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "CrateError: {}", self.0)
    }
}

impl std::error::Error for CrateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl From<ErrorUnion> for CrateError {
    fn from(e: ErrorUnion) -> Self {
        CrateError(e.into_inner_dyn_error())
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

This way the library can still use `ErrorUnion` internally for function composition, enabling features like `context` and `backtrace` for its own tests, while downstream crates only ever see a single concrete error type. `CrateError` is effectively just a thin wrapper around a boxed error, so the conversion at the boundary stays cheap regardless of how many error variants the library handles internally.

This pattern works for `AnyError` as shown above, but it isn't limited to it. When the internal `ErrorUnion` uses a typed tuple instead, `to_enum` can be used to convert into an enum, which can then be mapped into the crate's own error enum — giving callers something they can exhaustively match on.

<details>

<summary>Example Implementation</summary>

```rust
use eros::{E2, ErrorUnion, IntoUnion};
use std::{fmt, io};

#[derive(Debug)]
pub enum CrateError {
    Io(io::Error),
    Format(fmt::Error),
}

impl std::fmt::Display for CrateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrateError::Io(e) => write!(f, "{}", e),
            CrateError::Format(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for CrateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CrateError::Io(e) => e.source(),
            CrateError::Format(e) => e.source(),
        }
    }
}

impl From<ErrorUnion<(io::Error, fmt::Error)>> for CrateError {
    fn from(error: ErrorUnion<(io::Error, fmt::Error)>) -> Self {
        // `to_enum` converts the `ErrorUnion` into `E2<io::Error, fmt::Error>`,
        match error.to_enum() {
            E2::A(e) => CrateError::Io(e),
            E2::B(e) => CrateError::Format(e),
        }
    }
}

fn regular_typed_result1() -> Result<(), io::Error> {
    Err(io::Error::new(io::ErrorKind::AddrInUse, "message here"))
}

fn regular_typed_result2() -> Result<(), fmt::Error> {
    Err(fmt::Error)
}

fn internal_api() -> eros::Result<(), (io::Error, fmt::Error)> {
    regular_typed_result1().into_union()?;
    regular_typed_result2().into_union()?;
    Ok(())
}

pub fn public_api() -> Result<(), CrateError> {
    internal_api().map_err(Into::into)
}

fn main() {
    match public_api() {
        Ok(()) => println!("Success!"),
        Err(CrateError::Io(e)) => println!("IO error: {}", e),
        Err(CrateError::Format(e)) => println!("Format error: {}", e),
    }
}
```

</details>

With this, internal composing of errors can remain precise and ergonomic vs traditional enums, as outlined in [Why Traditional Enum Errors Scale Poorly](#why-traditional-enum-errors-scale-poorly) section. And downstream users are not exposed to the `ErrorUnion` type and instead see a traditional error enum. Note even if typed tuples are used internally, it is still completely valid to choose to type erase at the boundary with `into_inner`.

### Backtrace vs Location

`eros` has two location tracking feature flags `backtrace`, which captures a backtrace at error creation if `RUST_BACKTRACE` env variable is set, and `location`, which captures the location in the code that the error and context were created from. `location` is more efficient than `backtrace` since the call location is injected at compile time. While backtrace is generally more precise and useful. Both of these can be used together. `location` becomes especially useful for wasm environments where backtraces are not supported. `location` is not enabled by default, while `backtrace` is.

### Anyhow

`eros` comes with an `anyhow` feature flag. This adds a `ErrorUnion::anyhow` function for converting an `anyhow::Error` to an `ErrorUnion`. This can help integrate with legacy code.

`eros` can also quickly replace `anyhow` in any crate as simple as replacing all occurrences of:
- `anyhow!` with `error!`
- `anyhow::Error` with `eros::ErrorUnion`
- `anyhow::` with `eros::`

### Exposing Errors To Application Users

Not every error message should be shown directly to end users of an application.

Operational details such as SQL queries, filesystem paths, internal identifiers, etc. often make poor user experiences and may even leak sensitive information. Conversely, some errors are intentionally designed to be presented directly to users.

Instead, construct a user-facing message from two sources:

1. User context attached throughout the call stack. Enabled through the `user_context` feature flag
2. The underlying error itself, if the application recognizes it as safe to display.

<details>

<summary>Example Implementation</summary>

```rust,ignore
use eros::{Context, ErrorUnion, IntoDynUnion, SendSyncError, TypeSet};

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
    if let Some(error) = error.as_any().downcast_ref::<InvalidPasswordError>() {
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
    let error_message = root_error_message(error.inner_ref());
    message.push_str(&error_message);

    for context in error.user_contexts() {
        message.push('\n');
        message.push_str(&context.to_string());
    }

    message
}

fn validate_password(password: &str) -> eros::Result<()> {
    if password.len() < 8 {
        return Err(InvalidPasswordError)
            .context("Password validation failed")
            // This context is marked as "user facing" meaning it is safe to expose to the user
            .user_context("Please choose a stronger password.")
            .into_dyn_union();
    }
    Ok(())
}
fn load_configuration() -> eros::Result<()> {
    Err(SystemDiskError)
        .context("Failed to read configuration from /etc/my-app/config.toml")
        .into_dyn_union()
}
#[test]
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

### Context Placement: Two Approaches

There are two reasonable philosophies for *where* in the call stack context should be attached. Eros is flexible enough to support either, but it's worth picking one and being consistent within a codebase.

#### Approach 1: Attach Context at the Function

Under this approach, a function attaches context describing itself and its own parameters via `#[context]`. The function "owns" its own description, so every caller gets the same context for free, with no risk of forgetting it or duplicating it slightly differently at each call-site.

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

#### Approach 2: Attach Context Only at the Boundary Where Information Would Otherwise Be Lost

A function should only attach context that the caller doesn't already have. If a caller already knows the value of `param`, then a callee re-stating `param` in its own context adds no new information, just noise.

Responsibility for attaching a given piece of context passes transitively up the stack to whichever function is the last one that still has access to the information, even if that's several layers above where the error actually originated. In the example below, `func2` attaches nothing — it leaves that to its callers — and the context only gets attached at `func1` and `func1b`, the two places where `param` would otherwise be lost:

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

### Why Traditional Enum Errors Scale Poorly

Traditional enum-based error handling breaks down as soon as you compose functions, because each new layer of composition demands its own enum.

#### The Problem

Suppose three low-level functions each return a precise error enum:

```rust
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

```rust,ignore
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

```rust,ignore
match initialize_system() {
    Ok(()) => println!("Success!"),
    Err(InitError::Read(ReadError::Io(_))) => { /* handle */ }
    Err(InitError::Net(NetworkError::Io(_))) => { /* handle */ }
    Err(other) => println!("Some other error occurred: {:?}", other),
}
```

Add a fourth step that returns a `DatabaseError` and the cycle repeats: a new enum, new `From` impls, and every downstream `match` needs updating.

#### Why Crates Give Up

Faced with this growth, most crates abandon precision entirely and adopt one monolithic, crate-wide error enum:

```rust,ignore
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

#### How `ErrorUnion` Avoids This

`ErrorUnion` sidesteps the dilemma entirely. No new enum is needed to combine errors, so precision and ergonomics stop being a trade-off.

```rust,ignore
type MyError = (io::Error, ParseIntError, AddrParseError);

fn initialize_system() -> eros::Result<(), MyError> {
    let data = read_file().into_union()?;
    let port = parse_config(&data).into_union()?;
    open_socket(port).into_union()?;
    Ok(())
}
```

The signature stays exact — only the errors that can actually occur are listed — and adding a fourth fallible step just means adding one type to the tuple, not a new enum and a new set of `From` impls, or a rewritten `match`.

## Special Thanks

Special thank you to the authors and contributors of the following crates that inspired `eros`:
- [anyhow](https://github.com/dtolnay/anyhow)
- [terrors](https://github.com/komora-io/terrors)
- [error_set](https://github.com/mcmah309/error_set)
- [thiserror](https://github.com/dtolnay/thiserror)
- [tracing](https://github.com/tokio-rs/tracing)
- [color-eyre](https://github.com/eyre-rs/eyre/tree/master/color-eyre)
- [err_trail](https://github.com/mcmah309/err_trail)