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

Error types only matter when the caller cares about the type, otherwise this just hinders ergonomics and creates unnecessary noise. Thus, it should be easy for the developer to make the type opaque for developing fast composable apis. This is where [TracedError](#tracederror) helps.

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
    eros_result().unwrap_err();
    normal_result().unwrap_err();
    using_normal_and_eros_results().unwrap_err();
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
    error_union_result().unwrap_err();
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
            // This statement is not need, just to show the type explictly for this example.
            let _: io::Error = io_error;
            todo!()
        }
        // The error type of the Result has been narrowed.
        // It is now a union with a single type (`ErrorUnion<(sync::mpsc::RecvError,)>`),
        // thus we can convert into the inner traced type.
        Err(result) => {
            // This statement is not need, just to show the type explictly for this example.
            let result: eros::Result<(), (sync::mpsc::RecvError,)> = result;
            result.map_err(|e| e.into_inner())
        }
    }
}

fn main() {
    regular_result().unwrap_err();
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

Errors should always provided context of the operations in the call stack that lead to the error.

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
    let out = adding_more_context().unwrap_err();
    println!("{out:#?}");
}
```

```console
Something went wrong

Context:
        - This is some lazy context

Backtrace:
   0:     0x5561eb054735 - std::backtrace_rs::backtrace::libunwind::trace::hc389a5f23f39a50d
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/../../backtrace/src/backtrace/libunwind.rs:117:9
   1:     0x5561eb054735 - std::backtrace_rs::backtrace::trace_unsynchronized::h6eca87dcd6d323d8
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/../../backtrace/src/backtrace/mod.rs:66:14
   2:     0x5561eb054735 - std::backtrace::Backtrace::create::h1c21bf982658ba83
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/backtrace.rs:331:13
   3:     0x5561eb054685 - std::backtrace::Backtrace::force_capture::h09cde9fcccebf215
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/backtrace.rs:312:9
   4:     0x5561eb02e4e2 - eros::generic_error::TracedError<T>::new::h41e2123d6cf4fdd5
                               at /workspaces/eros/src/generic_error.rs:36:24
   5:     0x5561eafe8246 - x::func2::hc5bcba8eff1a9abd
                               at /workspaces/eros/tests/x.rs:17:5
   6:     0x5561eafe7f19 - x::func1::hc86226443a9fa2c0
                               at /workspaces/eros/tests/x.rs:7:15
   7:     0x5561eafe82dc - x::main::h6b82c0c63f51d406
                               at /workspaces/eros/tests/x.rs:28:15
...
```

### Optimizations

Eros comes with the `context` and `backtrace` feature flags enabled by default. If this is disabled, backtrace and context tracking are removed from `ErrorUnion<T>` and all context methods become a no-opt. Thus it may be optimized away by the compiler. 

`ErrorUnion`'s stack size is pointer size (uses a `Box`). Boxing errors is a common trick to increase performance and decrease stack memory usage in many cases. This is because boxing may decrease the size of the return type, e.g. `Result<(),Box<u128>>` is smaller than `Result<(),u128>>`.

See the [Use In Libraries](#use-in-libraries) section as well.

## `ErrorUnion`

### Open Sum Type

`ErrorUnion` is an open sum type. An open sum type takes full advantage of rust's powerful type system. It differs from an enum in that you do not need to define any actual new type in order to hold some specific combination of variants, but rather you simply describe the ErrorUnion as holding one value out of several specific possibilities. This is declared by using a tuple of those possible variants as the generic parameter for the `ErrorUnion`. 

For example, a `ErrorUnion<(String, u32)>` contains either a `String` or a `u32`. The benefit of this over creating specific enums for each function become apparent in larger codebases where error handling needs to occur in different places for different errors. As such, `ErrorUnion` allows you to quickly specify a function's return value as involving a precise subset of errors that the caller can clearly reason about. Providing maximum composability with no boilerplate. E.g.

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

## Use In Libraries

`eros`'s flexibility and optimizations make it a the perfect option for both libraries and binaries.

*Libraries should consider disabling default features* and allowing downstream crates to enable this. This can then be enabled for tests only in the library.

### Suggested Route

Exposing `ErrorUnion` in a public api is perfectly fine and usually preferred. It allows multiple crates to use the power of these constructs together. see the [Optimizations](#optimizations) section for more info. Just make sure to re-export these constructs if exposed.

### Alternatives

#### Wrapper Error Types

An alternative to exposing `ErrorUnion` is a wrapper type like a new type - `MyErrorType(ErrorUnion)`. If such a route is taken, consider implementing `Deref`/`DerefMut`. That way, a downstream can also add additional context. Additionally/alternatively, consider adding an `into_union` method as a way to to convert to the underlying `ErrorUnion`. That way, if a downstream uses Eros they can get the `ErrorUnion` rather than wrapping it in another `ErrorUnion`. 

The downside is wrapping/nesting `ErrorUnion` may still unintentionally occur, that is why exposing the `ErrorUnion` in the api is usually preferred, since `ErrorUnion` cannot be nested within itself. Additionally the `into_union` api can no longer be used across api boundaries which limits composability.

#### Non-Wrapper Error Types

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

## Special Thanks

Special thank you to the authors and contributors of the following crates that inspired `eros`:
- [anyhow](https://github.com/dtolnay/anyhow)
- [terrors](https://github.com/komora-io/terrors)
- [error_set](https://github.com/mcmah309/error_set)
- [thiserror](https://github.com/dtolnay/thiserror)