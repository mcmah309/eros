# eros

[<img alt="github" src="https://img.shields.io/badge/github-mcmah309/eros-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/mcmah309/eros)
[<img alt="crates.io" src="https://img.shields.io/crates/v/eros.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/eros)
[<img alt="test status" src="https://img.shields.io/github/actions/workflow/status/mcmah309/eros/ci.yml?branch=master&style=for-the-badge" height="20">](https://github.com/mcmah309/eros/actions/workflows/ci.yml)

Eros is the swish army knife of error handling approaches. It fits perfectly well into libraries and applications. Eros is heavily inspired by:

- [anyhow](https://github.com/dtolnay/anyhow)
- [terrors](https://github.com/komora-io/terrors)
- [error_set](https://github.com/mcmah309/error_set)
- [thiserror](https://github.com/dtolnay/thiserror)

Eros is built on this philosophy:
1. [Error types only matter when the caller cares about the type, otherwise this just hinders ergonomics and creates unnecessary noise.](#optional-typed-errors)
2. [There should be no boilerplate needed when handling single or multiple typed errors.](#no-boilerplate)
3. [Users should be able to seamlessly transition to and from fully typed errors.](#seamless-transitions-between-error-types)
4. [Errors should always provided context of the operations in the call stack that lead to the error.](#errors-have-context)

## In Philosophy In Action

### Optional Typed Errors

Error types only matter when the caller cares about the type, otherwise this just hinders ergonomics and creates unnecessary noise. Thus, it should be easy for the developer to make the type opaque for developing fast composable apis.

```rust
use eros::{bail, IntoDynTracedError};
use std::io::{Error, ErrorKind};

// The Error type is untracked and the underlying types are different
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

There should be no boilerplate needed when handling single or multiple typed error.

```rust
use eros::{bail, IntoConcreteTracedError, IntoUnionResult, TracedError};
use std::io::{Error, ErrorKind};

// Uses `ErrorUnion` to track each type. `TracedError` remains untyped and
// `TracedError<Error>` is typed.
fn func1() -> eros::UnionResult<(), (TracedError<Error>, TracedError)> {
    // inflate the `TracedResult` type to an `UnionResult` type
    let val = func2().union()?;
    let val = func3().union()?;
    Ok(val)
}

// Error type not tracked
fn func2() -> eros::Result<()> {
    bail!("Something went wrong")
}

// Error type is tracked. Here the underlying error type is `std::io::Error`
fn func3() -> eros::Result<(), Error> {
    return Err(Error::new(ErrorKind::AddrInUse, "message here")).traced();
}

fn main() {
    func1();
}
```

`UnionResult` and the underlying `UnionError`, work with regular types as well, not just `TracedError`. Thus the error type could consist of non-traced errors as well. e.g.
```rust,ignore
fn func1() -> eros::UnionResult<(), (std::io::Error, my_crate::Error)>;
```

### Seamless Transitions Between Error Types

Users should be able to seamlessly transition to and from fully typed errors.

```rust
use eros::{bail, FlateUnionResult, IntoConcreteTracedError, IntoUnionResult, TracedError};
use std::io::{Error, ErrorKind};

fn func1() -> eros::UnionResult<(), (TracedError<Error>, TracedError)> {
    let val = func2().union()?;
    let val = func3().union()?;
    Ok(val)
}

fn func2() -> eros::Result<()> {
    bail!("Something went wrong")
}

fn func3() -> eros::Result<(), Error> {
    return Err(Error::new(ErrorKind::AddrInUse, "message here")).traced();
}

// Error type is no longer tracked, we handled internally. Otherwise we could
// have just turned the error back into a `TracedError`
fn func4() -> eros::Result<()> {
    // Deflate the `ErrorUnion` and handle to only handle `TracedError<Error>` case!
    match func1().deflate::<TracedError<Error>, _>() {
        Ok(traced_io_error) => {
            todo!("Handle `TracedError<std::io::Error>` case")
        }
        // The error type of the Result has been deflated.
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

And to expand an `ErrorUnion` just call `inflate`

```rust
use eros::{FlateUnionResult, IntoUnionResult};
use std::io::Error;

fn func1() -> eros::UnionResult<(), (Error, String)> {
    Ok(())
}

fn func2() -> eros::UnionResult<(), (i32, u16)> {
    Ok(())
}

fn func3() -> Result<(), f64> {
    Ok(())
}

fn func4() -> eros::UnionResult<(), (Error, String, i32, u16, f64)> {
    func1().inflate()?;
    func2().inflate()?;
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
    bail, Context, IntoDynTracedError, IntoUnionResult, TracedError,
};
use std::io::{Error, ErrorKind};

fn func1() -> eros::UnionResult<(), (TracedError<Error>, TracedError)> {
    let val = func2()
        .with_context(|| format!("This is some more context"))
        .union()?;
    let val = func3()
        .context(format!("This is some more context"))
        .union()?;
    Ok(val)
}

fn func2() -> eros::Result<()> {
    bail!("Something went wrong")
}

fn func3() -> eros::Result<()> {
    return Err(Error::new(ErrorKind::AddrInUse, "message here"))
        .traced_dyn()
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
   8:     0x5561eafea397 - x::main::{{closure}}::h9ec95e65e08ea0a5
                               at /workspaces/eros/tests/x.rs:27:10
   9:     0x5561eafe6bc6 - core::ops::function::FnOnce::call_once::h89665ff874f9aff0
                               at /usr/local/rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ops/function.rs:253:5
  10:     0x5561eb02945b - core::ops::function::FnOnce::call_once::he7780dbaf3819be9
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/core/src/ops/function.rs:253:5
  11:     0x5561eb02945b - test::__rust_begin_short_backtrace::he52f6244ba5ffadb
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/test/src/lib.rs:648:18
  12:     0x5561eb02862e - test::run_test_in_process::{{closure}}::h4b5580962b2f03a8
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/test/src/lib.rs:671:74
  13:     0x5561eb02862e - <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once::h19cb5d2621bd88eb
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/core/src/panic/unwind_safe.rs:272:9
  14:     0x5561eb02862e - std::panicking::catch_unwind::do_call::hea0162f6125d4c37
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/panicking.rs:589:40
  15:     0x5561eb02862e - std::panicking::catch_unwind::h58eff26629cdc5e5
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/panicking.rs:552:19
  16:     0x5561eb02862e - std::panic::catch_unwind::haee4559c8279658f
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/panic.rs:359:14
  17:     0x5561eb02862e - test::run_test_in_process::hd400bd155f277427
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/test/src/lib.rs:671:27
  18:     0x5561eb02862e - test::run_test::{{closure}}::h0d9903d185102994
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/test/src/lib.rs:592:43
  19:     0x5561eafec3a4 - test::run_test::{{closure}}::hc4b5b0598a6862e8
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/test/src/lib.rs:622:41
  20:     0x5561eafec3a4 - std::sys::backtrace::__rust_begin_short_backtrace::ha7ee3160b6c13598
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/sys/backtrace.rs:158:18
  21:     0x5561eafefc6a - std::thread::Builder::spawn_unchecked_::{{closure}}::{{closure}}::hbaba1875801144df
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/thread/mod.rs:559:17
  22:     0x5561eafefc6a - <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once::heb48e77784f0385f
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/core/src/panic/unwind_safe.rs:272:9
  23:     0x5561eafefc6a - std::panicking::catch_unwind::do_call::he0ffef791c49aaef
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/panicking.rs:589:40
  24:     0x5561eafefc6a - std::panicking::catch_unwind::h99d55591c3b90bdb
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/panicking.rs:552:19
  25:     0x5561eafefc6a - std::panic::catch_unwind::h4ea92e4fa0439888
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/panic.rs:359:14
  26:     0x5561eafefc6a - std::thread::Builder::spawn_unchecked_::{{closure}}::h03c8861180b28db2
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/thread/mod.rs:557:30
  27:     0x5561eafefc6a - core::ops::function::FnOnce::call_once{{vtable.shim}}::h00b23c1a00a0e90a
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/core/src/ops/function.rs:253:5
  28:     0x5561eb0619d7 - <alloc::boxed::Box<F,A> as core::ops::function::FnOnce<Args>>::call_once::hcd81d65010c14a3e
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/alloc/src/boxed.rs:1971:9
  29:     0x5561eb0619d7 - <alloc::boxed::Box<F,A> as core::ops::function::FnOnce<Args>>::call_once::h96a52a5b098b326a
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/alloc/src/boxed.rs:1971:9
  30:     0x5561eb0619d7 - std::sys::pal::unix::thread::Thread::new::thread_start::hd5dce28806973ef9
                               at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/sys/pal/unix/thread.rs:97:17
  31:     0x7f20d33eaaa4 - <unknown>
  32:     0x7f20d3477c3c - <unknown>
  33:                0x0 - <unknown>
  ```

## Putting It All Together

```rust
use eros::{
    traced, Context, FlateUnionResult, IntoConcreteTracedError, IntoDynTracedError,
    IntoUnionResult, TracedError,
};
use reqwest::blocking::Client;
use std::thread::sleep;
use std::time::Duration;

// Explicitly handle multiple Err types at the same time with `UnionResult`
// Add tracing to an error by wrapping it in a `TraceError`
// `UnionResult<_,_>` === `Result<_,ErrorUnion<_>>`
fn fetch_url(url: &str) -> eros::UnionResult<String, (TracedError<reqwest::Error>, TracedError)> {
    let client = Client::new();

    let res = client
        .get(url)
        .send()
        // Explicitly trace the `Err` with the type (`TracedError<reqwest::Error>`)
        .traced()
        // Add context to the traced error if an `Err`
        .with_context(|| format!("Url: {url}"))
        // Convert the `TracedError<reqwest::Error>` into a `UnionError<_>`.
        // If this type was already and `UnionError` we would call `inflate` instead.
        .union()?;

    if !res.status().is_success() {
        // `traced!` create a `TraceError`. See also `bail!`.
        Err(traced!("Bad response: {}", res.status())).union()?;
    }

    let body = res
        .text()
        // Trace the `Err` without the type (`TracedError`)
        .traced_dyn()
        .context("while reading response body")
        .union()?;

    Ok(body)
}

// `eros::Result<_>` === `Result<_,TracedError>` === `TracedResult<_>`
fn fetch_with_retry(url: &str, retries: usize) -> eros::Result<String> {
    let mut attempts = 0;

    loop {
        attempts += 1;

        // Handle one of the error types explicitly with `deflate`!
        match fetch_url(url).deflate::<TracedError<reqwest::Error>, _>() {
            Ok(request_error) => {
                if attempts < retries {
                    sleep(Duration::from_millis(200));
                    continue;
                } else {
                    return Err(request_error.into_dyn().context("Retries exceeded"));
                }
            }
            // `result` is now `UnionResult<String,(TracedError,)>`, so we convert the `Err` type
            // into `TracedError`. Thus `Result<String,TracedError>`
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
   1: <E as eros::generic_error::IntoConcreteTracedError<eros::generic_error::TracedError<E>>>::traced
             at ./src/generic_error.rs:211:9
   2: <core::result::Result<S,E> as eros::generic_error::IntoConcreteTracedError<core::result::Result<S,eros::generic_error::TracedError<E>>>>::traced::{{closure}}
             at ./src/generic_error.rs:235:28
   3: core::result::Result<T,E>::map_err
             at /usr/local/rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/result.rs:914:27
   4: <core::result::Result<S,E> as eros::generic_error::IntoConcreteTracedError<core::result::Result<S,eros::generic_error::TracedError<E>>>>::traced
             at ./src/generic_error.rs:235:14
   5: x::fetch_url
             at ./tests/x.rs:15:10
   6: x::fetch_with_retry
             at ./tests/x.rs:38:15
   7: x::main
             at ./tests/x.rs:54:11
   8: x::main::{{closure}}
             at ./tests/x.rs:53:10
   9: core::ops::function::FnOnce::call_once
             at /usr/local/rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/ops/function.rs:253:5
  10: core::ops::function::FnOnce::call_once
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/core/src/ops/function.rs:253:5
  11: test::__rust_begin_short_backtrace
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/test/src/lib.rs:648:18
  12: test::run_test_in_process::{{closure}}
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/test/src/lib.rs:671:74
  13: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/core/src/panic/unwind_safe.rs:272:9
  14: std::panicking::catch_unwind::do_call
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/panicking.rs:589:40
  15: std::panicking::catch_unwind
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/panicking.rs:552:19
  16: std::panic::catch_unwind
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/panic.rs:359:14
  17: test::run_test_in_process
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/test/src/lib.rs:671:27
  18: test::run_test::{{closure}}
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/test/src/lib.rs:592:43
  19: test::run_test::{{closure}}
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/test/src/lib.rs:622:41
  20: std::sys::backtrace::__rust_begin_short_backtrace
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/sys/backtrace.rs:158:18
  21: std::thread::Builder::spawn_unchecked_::{{closure}}::{{closure}}
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/thread/mod.rs:559:17
  22: <core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/core/src/panic/unwind_safe.rs:272:9
  23: std::panicking::catch_unwind::do_call
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/panicking.rs:589:40
  24: std::panicking::catch_unwind
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/panicking.rs:552:19
  25: std::panic::catch_unwind
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/panic.rs:359:14
  26: std::thread::Builder::spawn_unchecked_::{{closure}}
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/thread/mod.rs:557:30
  27: core::ops::function::FnOnce::call_once{{vtable.shim}}
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/core/src/ops/function.rs:253:5
  28: <alloc::boxed::Box<F,A> as core::ops::function::FnOnce<Args>>::call_once
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/alloc/src/boxed.rs:1971:9
  29: <alloc::boxed::Box<F,A> as core::ops::function::FnOnce<Args>>::call_once
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/alloc/src/boxed.rs:1971:9
  30: std::sys::pal::unix::thread::Thread::new::thread_start
             at /rustc/8f08b3a32478b8d0507732800ecb548a76e0fd0c/library/std/src/sys/pal/unix/thread.rs:97:17
  31: <unknown>
  32: <unknown>
```