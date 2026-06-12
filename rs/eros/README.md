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
use eros::{bail, TracedDyn};
use std::io::{Error, ErrorKind};

// The Error type is untracked and the underlying types are different.
// If one wanted to track the error this can be done - `eros::Result<(), _>`,
// since `eros::Result<()>` == `eros::Result<(), TracedError>`.
fn func1() -> eros::Result<()> {
    let val = func2()?;
    let val = func3()?;
    Ok(val)
}

fn func2() -> eros::Result<()> {
    bail!("Something went wrong")
}

fn func3() -> eros::Result<()> {
    return Err(Error::new(ErrorKind::AddrInUse, "message here")).traced_dyn();
}

fn main() {
    func1();
}
```

### No Boilerplate

There should be no boilerplate needed when handling any number of errors (typed or untyped). This is where [ErrorUnion](#errorunion) helps in addition to [TracedError](#tracederror).


```rust
use eros::{bail, Traced, Union, TE};
use std::io;

// `ErrorUnion` is used to track each possible error type,
// instead of creating a enum for each possible error variant.
// `UResult<_,(..)>` == `Result<_,ErrorUnion<(..)>>`.
// Here `TracedError` remains untyped and `TracedError<io::Error>` is typed.
// `TE<T>` is a type alias for `TracedError<T>`.
fn func1() -> eros::UResult<(), (TE<io::Error>, TE)> {
    // Change the `eros::Result` type to an `UResult` type
    let val = func2().union()?; // TracedError
    let val = func3().union()?; // TracedError<Error>
    Ok(val)
}

// The error type not tracked
fn func2() -> eros::Result<()> {
    bail!("Something went wrong")
}

// The error type can be tracked with `TracedError<T>` as well.
// Here the underlying error type is `io::Error`.
// `eros::Result<(), _>` == `Result<(), TracedError<_>>`
fn func3() -> eros::Result<(), io::Error> {
    return Err(io::Error::new(io::ErrorKind::AddrInUse, "message here")).traced();
}

fn main() {
    func1();
}
```
The above code is precisely typed for what we care about and there was no need to create an error enum for each case.

`UResult` and the underlying `ErrorUnion`, work with regular types as well, not just `TracedError`. Thus the error type could consist of non-traced errors as well. e.g.
```rust,ignore
fn func1() -> eros::UResult<(), (io::Error, my_crate::Error)> { todo!() }
```

### Seamless Transitions Between Error Types

Users should be able to seamlessly transition to and from fully typed errors. And handle any cases they care about.

```rust
use eros::{bail, ReshapeUnion, Traced, Union, TE};
use std::io;

fn func1() -> eros::UResult<(), (TE<io::Error>, TE)> {
    let val = func2().union()?;
    let val = func3().union()?;
    Ok(val)
}

fn func2() -> eros::Result<()> {
    bail!("Something went wrong")
}

fn func3() -> eros::Result<(), io::Error> {
    return Err(io::Error::new(io::ErrorKind::AddrInUse, "message here")).traced();
}

// Error type is no longer tracked, we handled internally.
fn func4() -> eros::Result<()> {
    // Narrow the `ErrorUnion` and handle to only handle `TracedError<Error>` case!
    match func1().narrow::<TE<io::Error>, _>() {
        Ok(traced_io_error) => {
            todo!("Handle `TracedError<std::io::Error>` case")
        }
        // The error type of the Result has been narrowed.
        // It is now a union with a single type (`ErrorUnion<(TracedError,)>`), 
        // thus we can convert into the inner traced type.
        // Note: Alternatively, we could just call `traced` on `result` to accomplish the same thing
        Err(result) => result.map_err(|e| e.into_inner()),
    }
}

fn main() {
    func4();
}
```

And to expand an `ErrorUnion` just call `widen`

```rust
use eros::{ReshapeUnion, Union};
use std::io;

fn func1() -> eros::UResult<(), (io::Error, String)> {
    Ok(())
}

fn func2() -> eros::UResult<(), (i32, u16)> {
    Ok(())
}

fn func3() -> Result<(), f64> {
    Ok(())
}

fn func4() -> eros::UResult<(), (io::Error, String, i32, u16, f64)> {
    func1().widen()?;
    func2().widen()?;
    func3().union()?;
    Ok(())
}

fn main() {
    func4();
}
```

### Errors Have Context

Errors should always provided context of the operations in the call stack that lead to the error.

```rust
use eros::{
    bail, Context, Union, TE,
};
use std::io;

fn func1() -> eros::UResult<(), (TE<io::Error>, TE)> {
    let val = func2()
        .with_context(|| format!("This is some more context"))
        .union()?;
    let val = func3()
        .context("This is some more context")
        .union()?;
    Ok(val)
}

fn func2() -> eros::Result<()> {
    bail!("Something went wrong")
}

fn func3() -> eros::Result<()> {
    return Err(io::Error::new(io::ErrorKind::AddrInUse, "message here"))
        // Trace the `Err` without the type (`TracedError`)
        // Note: Calling `.traced_dyn()` not needed. we can call `context` directly
        // .traced_dyn()
        .context("This is some context");
}

fn main() {
    // Can add context to `ErrorUnion` when the `min_specialization` feature flag is enabled
    // let out = func1().context("Last bit of context").unwrap_err();
    let out = func1();
    println!("{out:#?}");
}
```

```console
Something went wrong

Context:
        - This is some more context
        - Last bit of context

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

Eros comes with the `context` and `backtrace` feature flags enabled by default. If this is disabled, backtrace and context tracking are removed from `TracedError<T>` and all context methods become a no-opt. Thus, `TracedError<T>` becomes a new type and may be optimized away by the compiler. 

Additionally, in this case for the untyped version, `TracedError`, the new type is just a wrapper around a `Box`. Boxing errors is a common trick to increase performance and decrease stack memory usage in many cases. This is because boxing may decrease the size of the return type, e.g. `Result<(),Box<u128>>` is smaller than `Result<(),u128>>`.

See the [Use In Libraries](#use-in-libraries) section as well.

## Putting It All Together

```rust
use eros::{
    bail, Context, ReshapeUnion, Traced, TracedDyn, Union,
    TE,
};
use reqwest::blocking::{Client, Response};
use std::thread::sleep;
use std::time::Duration;

// Add tracing to an error by wrapping it in a `TracedError`.
// When we don't care about the error type we can use `eros::Result<_>` which has tracing.
// `eros::Result<_>` == `Result<_,TracedError>`
// When we *do* care about the error type we can use `eros::Result<_,_>` which also has tracing but preserves the error type.
// `eros::Result<_,_>` == `Result<_,TracedError<_>>`
// In the below example we don't preserve the error type.
fn handle_response(res: Response) -> eros::Result<String> {
    if !res.status().is_success() {
        // `bail!` to directly bail with the error message.
        // See `traced!` to create a `TracedError` without bailing.
        bail!("Bad response: {}", res.status());
    }

    let body = res
        .text()
        // Trace the `Err` without the type (`TracedError`)
        // Note: Calling `.traced_dyn()` not needed. we can call `context` directly
        // .traced_dyn()
        // Add context to the traced error if an `Err`
        .context("while reading response body")?;
    Ok(body)
}

// Explicitly handle multiple Err types at the same time with `UResult`.
// No new error enum creation is needed or nesting of errors.
// `UResult<_,_>` == `Result<_,ErrorUnion<_>>`
fn fetch_url(url: &str) -> eros::UResult<String, (TE<reqwest::Error>, TE)> {
    let client = Client::new();

    let res = client
        .get(url)
        .send()
        // Explicitly trace the `Err` with the type (`TracedError<reqwest::Error>`)
        .traced()
        // Add lazy context to the traced error if an `Err`
        .with_context(|| format!("Url: {url}"))
        // Convert the `TracedError<reqwest::Error>` into a `UnionError<_>`.
        // If this type was already a `UnionError`, we would call `widen` instead.
        .union()?;

    handle_response(res).union()
}

fn fetch_with_retry(url: &str, retries: usize) -> eros::Result<String> {
    let mut attempts = 0;

    loop {
        attempts += 1;

        // Handle one of the error types explicitly with `narrow`!
        match fetch_url(url).narrow::<TE<reqwest::Error>, _>() {
            Ok(request_error) => {
                if attempts < retries {
                    sleep(Duration::from_millis(200));
                    continue;
                } else {
                    return Err(request_error.traced_dyn().context("Retries exceeded"));
                }
            }
            // `result` is now `UResult<String,(TracedError,)>`, so we convert the `Err` type
            // into `TracedError`. Thus, we now have a `Result<String,TracedError>`.
            Err(result) => return result.map_err(|e| e.into_inner()),
        }
    }
}

fn main() {
    match fetch_with_retry("https://badurl214651523152316hng.com", 3).context("Fetch failed") {
        Ok(body) => println!("Ok Body:\n{body}"),
        Err(err) => eprintln!("Error:\n{err:?}"),
    }
}
```
Output:
```console
Error:
error sending request

Context:
        - Url: https://badurl214651523152316hng.com
        - Retries exceeded
        - Fetch failed

Backtrace:
   0: eros::generic_error::TracedError<T>::new
             at ./src/generic_error.rs:47:24
   1: <E as eros::generic_error::Traced<eros::generic_error::TracedError<E>>>::traced
             at ./src/generic_error.rs:211:9
   2: <core::result::Result<S,E> as eros::generic_error::Traced<core::result::Result<S,eros::generic_error::TracedError<E>>>>::traced::{{closure}}
             at ./src/generic_error.rs:235:28
   3: core::result::Result<T,E>::map_err
             at /usr/local/rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:914:27
   4: <core::result::Result<S,E> as eros::generic_error::Traced<core::result::Result<S,eros::generic_error::TracedError<E>>>>::traced
             at ./src/generic_error.rs:235:14
   5: x::fetch_url
             at ./tests/x.rs:39:10
   6: x::fetch_with_retry
             at ./tests/x.rs:56:15
   7: x::main
             at ./tests/x.rs:74:11
...
```

## `TracedError`

`TracedError` (type alias `TE`) allows adding context to an error throughout the callstack with the `context` or `with_context` methods. This context may be information such as variable values or ongoing operations while the error occurred. If the error is handled higher in the stack, then this can be disregarded (no log pollution). Otherwise you can log it (or panic), capturing all the relevant information in one log. A backtrace is captured and added to the log if `RUST_BACKTRACE` is set. Use `TracedError` if the underlying error type does not matter. Otherwise, the type can be specified with `TracedError<T>`.

## `ErrorUnion`

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

## Use In Libraries

`eros`'s flexibility and optimizations make it a the perfect option for both libraries and binaries.

*Libraries should consider disabling default features* and allowing downstream crates to enable this. This can then be enabled for tests only in the library.

### Suggested Route

Exposing `TracedError`, or `ErrorUnion` in a public api is perfectly fine and usually preferred. It allows multiple crates to use the power of these constructs together. see the [Optimizations](#optimizations) section for more info. Just make sure to re-export these constructs if exposed.

### Alternatives

#### Wrapper Error Types

An alternative to exposing `TracedError` is a wrapper type like a new type - `MyErrorType(TracedError)`. If such a route is taken, consider implementing `Deref`/`DerefMut`. That way, a downstream can also add additional context. Additionally/alternatively, consider adding an `into_traced` method as a way to to convert to the underlying `TracedError`. That way, if a downstream uses Eros they can get the `TracedError` rather than wrapping it in another `TracedError`. 

The downside is wrapping/nesting `TracedError` may still unintentionally occur, that is why exposing the `TracedError` in the api is usually preferred, since `TracedError` cannot be nested within itself. Additionally the `into_traced` api can no longer be used across api boundaries ([example](https://github.com/mcmah309/error_set?tab=readme-ov-file#eros)) which limits composability.

#### Non-Wrapper Error Types

If one wants to add their own custom error type for all public api's without exposing constructs like `TracedError`, use the `into_inner` method at these boundaries.

<details>

<summary>Example Implementation</summary>

```rust
use eros::{AnyError, TracedError};

#[derive(Debug)]
struct MyErrorType(Box<dyn AnyError>);

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
    Err(TracedError::boxed(std::io::Error::new(std::io::ErrorKind::Other, "io error")))
}

pub fn public_api() -> Result<(), MyErrorType> {
    // Replace `TracedError` with your custom error type
    internal_api().map_err(|e| MyErrorType(e.into_inner()))
}
```

</details>

## Special Thanks

Special thank you to the authors and contributors of the following crates that inspired `eros`:
- [anyhow](https://github.com/dtolnay/anyhow)
- [terrors](https://github.com/komora-io/terrors)
- [error_set](https://github.com/mcmah309/error_set)
- [thiserror](https://github.com/dtolnay/thiserror)