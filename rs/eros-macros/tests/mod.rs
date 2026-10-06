#![allow(unused)]

//************************************************************************//

#[eros_macros::context("arg1 is {}", arg1)]
fn test_function(arg1: &str, arg2: String, arg3: i32) -> eros::Result<String> {
    eros::bail!("This is the error")
}

#[test]
fn test() {
    let error = test_function("test", "arg2".to_owned(), 42).unwrap_err();
    let inner_error = error.inner();
    assert_eq!(inner_error.to_string(), "This is the error");
    assert!(format!("{:?}", error).contains("    1. arg1 is test\n"));
}

//************************************************************************//

#[eros_macros::context("async arg is {}", value)]
async fn async_function(value: u32) -> eros::Result<u32> {
    eros::bail!("async error")
}

#[tokio::test]
async fn test_async_function_context_is_attached() {
    let error = async_function(99).await.unwrap_err();
    assert_eq!(error.inner().to_string(), "async error");
    assert!(format!("{:?}", error).contains("    1. async arg is 99\n"));
}

#[eros_macros::context("fetching {}", url)]
async fn async_success(url: &str) -> eros::Result<String> {
    Ok(format!("ok: {}", url))
}

#[tokio::test]
async fn test_async_function_ok_passes_through() {
    let result = async_success("https://example.com").await.unwrap();
    assert_eq!(result, "ok: https://example.com");
}

//************************************************************************//

struct Fetcher {
    base_url: String,
}

impl Fetcher {
    #[eros_macros::context("fetching from {}", self.base_url)]
    fn fetch(&self, path: &str) -> eros::Result<String> {
        eros::bail!("connection refused")
    }

    #[eros_macros::context("fetching from {} path {}", self.base_url, path)]
    fn fetch_multi_arg(&self, path: &str) -> eros::Result<String> {
        eros::bail!("not found")
    }

    #[eros_macros::context("shared ref returns ok")]
    fn fetch_ok(&self) -> eros::Result<String> {
        Ok(self.base_url.clone())
    }
}

#[test]
fn test_shared_ref_context_is_attached() {
    let f = Fetcher {
        base_url: "http://localhost".to_owned(),
    };
    let error = f.fetch("/api").unwrap_err();
    assert_eq!(error.inner().to_string(), "connection refused");
    assert!(format!("{:?}", error).contains("    1. fetching from http://localhost\n"));
}

#[test]
fn test_shared_ref_multiple_format_args() {
    let f = Fetcher {
        base_url: "http://localhost".to_owned(),
    };
    let error = f.fetch_multi_arg("/api/v1").unwrap_err();
    let debug = format!("{:?}", error);
    assert!(debug.contains("http://localhost"));
    assert!(debug.contains("/api/v1"));
}

#[test]
fn test_shared_ref_ok_passes_through() {
    let f = Fetcher {
        base_url: "http://localhost".to_owned(),
    };
    assert_eq!(f.fetch_ok().unwrap(), "http://localhost");
}

//************************************************************************//

struct Counter {
    count: u32,
}

impl Counter {
    #[eros_macros::context("increment failed at count {}", self.count)]
    fn increment(&mut self, by: u32) -> eros::Result<()> {
        if self.count + by > 100 {
            eros::bail!("overflow");
        }
        self.count += by;
        Ok(())
    }

    #[eros_macros::context("reset failed")]
    fn reset(&mut self) -> eros::Result<()> {
        self.count = 0;
        Ok(())
    }
}

#[test]
fn test_mut_ref_context_is_attached() {
    let mut c = Counter { count: 95 };
    let error = c.increment(10).unwrap_err();
    assert_eq!(error.inner().to_string(), "overflow");
    assert!(format!("{:?}", error).contains("    1. increment failed at count 95\n"));
}

#[test]
fn test_mut_ref_ok_mutates_state() {
    let mut c = Counter { count: 0 };
    c.increment(42).unwrap();
    assert_eq!(c.count, 42);
}

#[test]
fn test_mut_ref_ok_no_context_noise() {
    let mut c = Counter { count: 10 };
    c.reset().unwrap();
    assert_eq!(c.count, 0);
}

//************************************************************************//

struct Wrapper(String);

impl Wrapper {
    #[eros_macros::context("consuming wrapper")]
    fn consume(self) -> eros::Result<String> {
        eros::bail!("consumed and failed")
    }

    #[eros_macros::context("consuming wrapper ok")]
    fn consume_ok(self) -> eros::Result<String> {
        Ok(self.0)
    }
}

#[test]
fn test_value_receiver_context_is_attached() {
    let w = Wrapper("hello".to_owned());
    let error = w.consume().unwrap_err();
    assert_eq!(error.inner().to_string(), "consumed and failed");
    assert!(format!("{:?}", error).contains("    1. consuming wrapper\n"));
}

#[test]
fn test_value_receiver_ok_passes_through() {
    let w = Wrapper("hello".to_owned());
    assert_eq!(w.consume_ok().unwrap(), "hello");
}

//************************************************************************//

struct AsyncClient {
    host: String,
}

impl AsyncClient {
    #[eros_macros::context("async fetch from {}", self.host)]
    async fn fetch(&self, endpoint: &str) -> eros::Result<String> {
        eros::bail!("timeout")
    }

    #[eros_macros::context("async fetch ok from {}", self.host)]
    async fn fetch_ok(&self) -> eros::Result<String> {
        Ok(self.host.clone())
    }
}

#[tokio::test]
async fn test_async_shared_ref_context_is_attached() {
    let client = AsyncClient {
        host: "api.example.com".to_owned(),
    };
    let error = client.fetch("/v1/items").await.unwrap_err();
    assert_eq!(error.inner().to_string(), "timeout");
    assert!(format!("{:?}", error).contains("    1. async fetch from api.example.com\n"));
}

#[tokio::test]
async fn test_async_shared_ref_ok_passes_through() {
    let client = AsyncClient {
        host: "api.example.com".to_owned(),
    };
    assert_eq!(client.fetch_ok().await.unwrap(), "api.example.com");
}

//************************************************************************//

struct AsyncQueue {
    items: Vec<String>,
}

impl AsyncQueue {
    #[eros_macros::context("push failed, queue len {}", self.items.len())]
    async fn push(&mut self, item: String) -> eros::Result<()> {
        if self.items.len() >= 2 {
            eros::bail!("queue full");
        }
        self.items.push(item);
        Ok(())
    }
}

#[tokio::test]
async fn test_async_mut_ref_context_is_attached() {
    let mut q = AsyncQueue {
        items: vec!["a".into(), "b".into()],
    };
    let error = q.push("c".into()).await.unwrap_err();
    assert_eq!(error.inner().to_string(), "queue full");
    assert!(format!("{:?}", error).contains("    1. push failed, queue len 2\n"));
}

#[tokio::test]
async fn test_async_mut_ref_ok_mutates_state() {
    let mut q = AsyncQueue { items: vec![] };
    q.push("first".into()).await.unwrap();
    assert_eq!(q.items, vec!["first"]);
}

//************************************************************************//

#[eros_macros::context("no format args at all")]
fn no_args_function() -> eros::Result<()> {
    eros::bail!("bare error")
}

#[test]
fn test_static_context_string_no_args() {
    let error = no_args_function().unwrap_err();
    assert!(format!("{:?}", error).contains("    1. no format args at all\n"));
}

#[eros_macros::context("a={} b={} c={}", a, b, c)]
fn three_arg_function(a: u8, b: u8, c: u8) -> eros::Result<()> {
    eros::bail!("three args error")
}

#[test]
fn test_context_with_three_format_args() {
    let error = three_arg_function(1, 2, 3).unwrap_err();
    assert!(format!("{:?}", error).contains("    1. a=1 b=2 c=3\n"));
}

#[eros_macros::context("debug value is {:?}", value)]
fn debug_format_function(value: &[u8]) -> eros::Result<()> {
    eros::bail!("debug format error")
}

#[test]
fn test_context_with_debug_format_specifier() {
    let error = debug_format_function(&[1, 2, 3]).unwrap_err();
    assert!(format!("{:?}", error).contains("[1, 2, 3]"));
}

//************************************************************************//

// Single #[fmt("{}")] param
#[eros_macros::context]
fn auto_display(#[fmt("{}")] name: &str, ignored: u32) -> eros::Result<()> {
    eros::bail!("inner error")
}

#[test]
fn test_auto_display_single_param() {
    let error = auto_display("alice", 0).unwrap_err();
    assert_eq!(error.inner().to_string(), "inner error");
    assert!(format!("{:?}", error).contains("    1. name: alice\n"));
}

// Single #[fmt("{:?}")] param
#[derive(Debug)]
struct Flags(u8);

#[eros_macros::context]
fn auto_debug(#[fmt("{:?}")] flags: &Flags) -> eros::Result<()> {
    eros::bail!("debug error")
}

#[test]
fn test_auto_debug_single_param() {
    let error = auto_debug(&Flags(0b1010)).unwrap_err();
    let debug_out = format!("{:?}", error);
    // The context line should contain the Debug output of Flags
    assert!(debug_out.contains("flags: Flags(10)"));
}

// Multiple annotated params — both display and debug present
#[derive(Debug)]
struct Mode(String);

#[eros_macros::context]
fn auto_mixed(
    #[fmt("{}")] user: &str,
    count: usize, // unannotated — should NOT appear in context
    #[fmt("{:?}")] mode: &Mode,
) -> eros::Result<()> {
    eros::bail!("mixed error")
}

#[test]
fn test_auto_mixed_params_order_and_content() {
    let error = auto_mixed("bob", 99, &Mode("fast".into())).unwrap_err();
    let debug_out = format!("{:?}", error);
    // Both annotated params appear
    assert!(debug_out.contains("user: bob"));
    assert!(debug_out.contains("mode: Mode(\"fast\")"));
    // Unannotated param does NOT appear
    assert!(!debug_out.contains("count:"));
    // user appears before mode (declaration order preserved)
    let user_pos = debug_out.find("user: bob").unwrap();
    let mode_pos = debug_out.find("mode: Mode").unwrap();
    assert!(user_pos < mode_pos);
}

// Ok path — no context noise on success
#[eros_macros::context]
fn auto_ok(#[fmt("{}")] value: &str) -> eros::Result<String> {
    Ok(value.to_owned())
}

#[test]
fn test_auto_ok_passes_through() {
    assert_eq!(auto_ok("hello").unwrap(), "hello");
}

// Auto format on &self method
struct Processor {
    name: String,
}

impl Processor {
    #[eros_macros::context]
    fn run(&self, #[fmt("{}")] job: &str) -> eros::Result<()> {
        eros::bail!("process failed")
    }
}

#[test]
fn test_auto_display_on_self_method() {
    let p = Processor {
        name: "worker".into(),
    };
    let error = p.run("batch-42").unwrap_err();
    let debug_out = format!("{:?}", error);
    assert!(debug_out.contains("job: batch-42"));
}

// Auto format on async fn
#[eros_macros::context]
async fn auto_async(#[fmt("{}")] endpoint: &str) -> eros::Result<()> {
    eros::bail!("async auto error")
}

#[tokio::test]
async fn test_auto_display_async() {
    let error = auto_async("https://api.example.com/v1").await.unwrap_err();
    assert!(format!("{:?}", error).contains("endpoint: https://api.example.com/v1"));
}

// Auto format on async &mut self method
struct Pipeline {
    stage: String,
}

impl Pipeline {
    #[eros_macros::context]
    async fn execute(&mut self, #[fmt("{:?}")] input: &[u8]) -> eros::Result<()> {
        eros::bail!("pipeline failed")
    }
}

#[tokio::test]
async fn test_auto_debug_async_mut_self() {
    let mut p = Pipeline {
        stage: "encode".into(),
    };
    let error = p.execute(&[1, 2, 3]).await.unwrap_err();
    assert!(format!("{:?}", error).contains("input: [1, 2, 3]"));
}

//************************************************************************//

// Eager formatting happens before owned parameters move into the body.
#[eros_macros::eager_context("processing {}", string)]
fn owned_string_function(string: String) -> eros::Result<()> {
    eros::bail!("owned error")
}

#[test]
fn test_owned_string_eager_context_is_attached() {
    let error = owned_string_function("hello".to_owned()).unwrap_err();
    assert_eq!(error.inner().to_string(), "owned error");
    assert!(format!("{:?}", error).contains("    1. processing hello\n"));
}

#[test]
fn test_owned_string_eager_ok_passes_through() {
    #[eros_macros::eager_context("value was {}", val)]
    fn returns_ok(val: String) -> eros::Result<String> {
        Ok(val)
    }
    assert_eq!(returns_ok("world".to_owned()).unwrap(), "world");
}

// Multiple owned parameters.
#[eros_macros::eager_context("a={} b={}", a, b)]
fn two_owned_params(a: String, b: String) -> eros::Result<()> {
    eros::bail!("two owned error")
}

#[test]
fn test_two_owned_params_eager_context() {
    let error = two_owned_params("foo".to_owned(), "bar".to_owned()).unwrap_err();
    let debug = format!("{:?}", error);
    assert!(debug.contains("    1. a=foo b=bar\n"));
}

// Mix of owned and Copy parameters.
#[eros_macros::eager_context("name={} id={}", name, id)]
fn mixed_owned_and_borrowed(name: String, id: u32) -> eros::Result<()> {
    eros::bail!("mixed error")
}

#[test]
fn test_mixed_owned_and_copy_eager_context() {
    let error = mixed_owned_and_borrowed("alice".to_owned(), 42).unwrap_err();
    assert!(format!("{:?}", error).contains("    1. name=alice id=42\n"));
}

// The same owned parameter can appear twice in the format string.
#[eros_macros::eager_context("first={} again={}", val, val)]
fn duplicate_eager_same_param(val: String) -> eros::Result<()> {
    eros::bail!("duplicate eager error")
}

#[test]
fn test_duplicate_eager_same_param_compiles_and_works() {
    let error = duplicate_eager_same_param("dup".to_owned()).unwrap_err();
    assert!(format!("{:?}", error).contains("    1. first=dup again=dup\n"));
}

// Async free function with an owned parameter.
#[eros_macros::eager_context("async processing {}", payload)]
async fn async_owned_function(payload: String) -> eros::Result<()> {
    eros::bail!("async owned error")
}

#[tokio::test]
async fn test_async_owned_eager_context_is_attached() {
    let error = async_owned_function("data".to_owned()).await.unwrap_err();
    assert_eq!(error.inner().to_string(), "async owned error");
    assert!(format!("{:?}", error).contains("    1. async processing data\n"));
}

#[tokio::test]
async fn test_async_owned_eager_ok_passes_through() {
    #[eros_macros::eager_context("async val={}", v)]
    async fn async_ok(v: String) -> eros::Result<String> {
        Ok(v)
    }
    assert_eq!(async_ok("ok".to_owned()).await.unwrap(), "ok");
}

// &self method — owned param alongside self reference.
struct Dispatcher {
    prefix: String,
}

impl Dispatcher {
    #[eros_macros::eager_context("dispatching {} via {}", task, self.prefix)]
    fn dispatch(&self, task: String) -> eros::Result<()> {
        eros::bail!("dispatch failed")
    }

    #[eros_macros::eager_context("ok dispatch {} via {}", task, self.prefix)]
    fn dispatch_ok(&self, task: String) -> eros::Result<String> {
        Ok(format!("{}/{}", self.prefix, task))
    }
}

#[test]
fn test_self_method_with_eager_owned_param() {
    let d = Dispatcher {
        prefix: "queue-A".to_owned(),
    };
    let error = d.dispatch("job-1".to_owned()).unwrap_err();
    let debug = format!("{:?}", error);
    assert!(debug.contains("dispatching job-1 via queue-A"));
}

#[test]
fn test_self_method_eager_ok_passes_through() {
    let d = Dispatcher {
        prefix: "queue-B".to_owned(),
    };
    assert_eq!(d.dispatch_ok("job-2".to_owned()).unwrap(), "queue-B/job-2");
}

// &mut self method with an owned parameter.
struct Journal {
    entries: Vec<String>,
}

impl Journal {
    #[eros_macros::eager_context("writing entry {}", entry)]
    fn write(&mut self, entry: String) -> eros::Result<()> {
        if self.entries.len() >= 3 {
            eros::bail!("journal full");
        }
        self.entries.push(entry);
        Ok(())
    }
}

#[test]
fn test_mut_self_method_eager_context_is_attached() {
    let mut j = Journal {
        entries: vec!["a".into(), "b".into(), "c".into()],
    };
    let error = j.write("d".to_owned()).unwrap_err();
    assert!(format!("{:?}", error).contains("    1. writing entry d\n"));
}

#[test]
fn test_mut_self_method_eager_ok_mutates_state() {
    let mut j = Journal { entries: vec![] };
    j.write("first".to_owned()).unwrap();
    assert_eq!(j.entries, vec!["first"]);
}

// Async &self method with an owned parameter.
struct AsyncWorker {
    name: String,
}

impl AsyncWorker {
    #[eros_macros::eager_context("worker {} processing {}", self.name, item)]
    async fn process(&self, item: String) -> eros::Result<()> {
        eros::bail!("worker failed")
    }
}

#[tokio::test]
async fn test_async_self_method_eager_context_is_attached() {
    let w = AsyncWorker {
        name: "w1".to_owned(),
    };
    let error = w.process("task-X".to_owned()).await.unwrap_err();
    assert!(format!("{:?}", error).contains("    1. worker w1 processing task-X\n"));
}

// Owned parameters do not need to implement Clone.
#[derive(Debug)]
struct JobId(u64);

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "job#{}", self.0)
    }
}

#[eros_macros::eager_context("running {}", id)]
fn custom_eager_type(id: JobId) -> eros::Result<()> {
    eros::bail!("job failed")
}

#[test]
fn test_custom_eager_type_display_in_context() {
    let error = custom_eager_type(JobId(7)).unwrap_err();
    assert!(format!("{:?}", error).contains("    1. running job#7\n"));
}

// Debug formatting of an owned parameter.
#[eros_macros::eager_context("payload={:?}", data)]
fn debug_eager_function(data: Vec<u8>) -> eros::Result<()> {
    eros::bail!("debug eager error")
}

#[test]
fn test_eager_with_debug_format_specifier() {
    let error = debug_eager_function(vec![10, 20, 30]).unwrap_err();
    assert!(format!("{:?}", error).contains("payload=[10, 20, 30]"));
}

#[eros_macros::eager_context]
fn eager_auto_owned(
    #[fmt("{}")] id: JobId,
    ignored: String,
    #[fmt("{:?}")] data: Vec<u8>,
) -> eros::Result<()> {
    drop((id, ignored, data));
    Err(eros::error!("auto owned error"))?;
    Ok(())
}

#[test]
fn test_eager_auto_owned_params_without_clone() {
    let error = eager_auto_owned(JobId(9), "secret".into(), vec![1, 2]).unwrap_err();
    let message = error.contexts().next().unwrap().to_string();
    assert_eq!(message, "id: job#9\ndata: [1, 2]\n");
}

#[eros_macros::eager_context()]
async fn eager_auto_async(#[fmt("{}")] value: String) -> eros::Result<String> {
    Ok(value)
}

#[tokio::test]
async fn test_eager_auto_async_ok() {
    assert_eq!(eager_auto_async("owned".into()).await.unwrap(), "owned");
}

struct EagerReceiver {
    value: String,
}

impl EagerReceiver {
    #[eros_macros::eager_context("consuming {}", self.value)]
    fn consume(self) -> eros::Result<()> {
        drop(self.value);
        eros::bail!("consumed")
    }

    #[eros_macros::eager_context("before={} input={}", self.value, input)]
    async fn replace(&mut self, input: String) -> eros::Result<()> {
        self.value = input;
        eros::bail!("replaced")
    }

    #[eros_macros::eager_context("async consuming {}", self.value)]
    async fn consume_async(self) -> eros::Result<()> {
        drop(self.value);
        eros::bail!("async consumed")
    }
}

#[test]
fn test_eager_consuming_receiver() {
    let receiver = EagerReceiver {
        value: "original".into(),
    };
    let error = receiver.consume().unwrap_err();
    assert_eq!(
        error.contexts().next().unwrap().to_string(),
        "consuming original"
    );
}

#[tokio::test]
async fn test_eager_async_mut_receiver_captures_input_state() {
    let mut receiver = EagerReceiver {
        value: "original".into(),
    };
    let future = receiver.replace("updated".into());
    let error = future.await.unwrap_err();
    assert_eq!(receiver.value, "updated");
    assert_eq!(
        error.contexts().next().unwrap().to_string(),
        "before=original input=updated"
    );
}

#[tokio::test]
async fn test_eager_async_consuming_receiver() {
    let receiver = EagerReceiver {
        value: "original".into(),
    };
    let error = receiver.consume_async().await.unwrap_err();
    assert_eq!(
        error.contexts().next().unwrap().to_string(),
        "async consuming original"
    );
}

#[eros_macros::eager_context("static message")]
fn eager_no_args() -> eros::Result<()> {
    eros::bail!("failed")
}

#[test]
fn test_eager_static_message() {
    let error = eager_no_args().unwrap_err();
    assert_eq!(
        error.contexts().next().unwrap().to_string(),
        "static message"
    );
}
