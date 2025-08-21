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
1. [Error types only matter when the caller cares about the type, otherwise this just hinders ergonomics and creates unnecessary noise.](#optional-typed-errors).
2. [There should be no boilerplate needed when handling single or multiple typed errors.](#no-boilerplate)
3. [Users should be able to seamlessly transition to and from fully typed errors.](#seamless-transitions-between-error-types)
4. [Errors should always provided context of the operations in the call stack that lead to the error.](#errors-have-context)

## In Philosophy In Action

### Optional Typed Errors

Error types only matter when the caller cares about the type, otherwise this just hinders ergonomics and creates unnecessary noise. Thus, it should be easy for the developer to make the type opaque for developing fast composable apis. Thus improving ergonomics.

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

There should be no boilerplate needed when handling single or multiple typed error

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

Users should be able to seamlessly transition to and from fully typed errors

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
    // Deflate the `ErrorUnion` and handle the `TracedError<Error>` case.
    match func1().deflate::<TracedError<Error>, _>() {
        Ok(traced_io_error) => {
            todo!()
        }
        // The error type is now `ErrorUnion<(TracedError,)>`, thus we can convert into the inner traced type
        Err(result) => result.traced(),
    }
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
    // Can add context to `ErrorUnion` when the `min_specialization` feature is enabled
    // let out = func1().context("Last bit of context").unwrap_err();
    let out = func1();
    println!("{out:#?}");
}
```

<details>

  <summary>Output</summary>

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

</details>