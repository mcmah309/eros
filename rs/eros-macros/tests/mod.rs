// ── Existing baseline ────────────────────────────────────────────────────────

#[eros_macros::context("arg1 is {}", arg1)]
fn test_function(arg1: &str, arg2: String, arg3: i32) -> eros::Result<String> {
    eros::bail!("This is the error")
}

#[test]
fn test() {
    let error = test_function("test", "arg2".to_owned(), 42).unwrap_err();
    let inner_error = error.inner_ref();
    assert_eq!(inner_error.to_string(), "This is the error");
    assert!(format!("{:?}", error).contains("\t- arg1 is test\n"));
}

// ── Async free functions ─────────────────────────────────────────────────────

#[eros_macros::context("async arg is {}", value)]
async fn async_function(value: u32) -> eros::Result<u32> {
    eros::bail!("async error")
}

#[tokio::test]
async fn test_async_function_context_is_attached() {
    let error = async_function(99).await.unwrap_err();
    assert_eq!(error.inner_ref().to_string(), "async error");
    assert!(format!("{:?}", error).contains("\t- async arg is 99\n"));
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

// ── &self methods ────────────────────────────────────────────────────────────

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
    assert_eq!(error.inner_ref().to_string(), "connection refused");
    assert!(format!("{:?}", error).contains("\t- fetching from http://localhost\n"));
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

// ── &mut self methods ─────────────────────────────────────────────────────────

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
    assert_eq!(error.inner_ref().to_string(), "overflow");
    assert!(format!("{:?}", error).contains("\t- increment failed at count 95\n"));
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

// ── self (by-value) methods ───────────────────────────────────────────────────

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
    assert_eq!(error.inner_ref().to_string(), "consumed and failed");
    assert!(format!("{:?}", error).contains("\t- consuming wrapper\n"));
}

#[test]
fn test_value_receiver_ok_passes_through() {
    let w = Wrapper("hello".to_owned());
    assert_eq!(w.consume_ok().unwrap(), "hello");
}

// ── Async &self methods ───────────────────────────────────────────────────────

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
    assert_eq!(error.inner_ref().to_string(), "timeout");
    assert!(format!("{:?}", error).contains("\t- async fetch from api.example.com\n"));
}

#[tokio::test]
async fn test_async_shared_ref_ok_passes_through() {
    let client = AsyncClient {
        host: "api.example.com".to_owned(),
    };
    assert_eq!(client.fetch_ok().await.unwrap(), "api.example.com");
}

// ── Async &mut self methods ───────────────────────────────────────────────────

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
    assert_eq!(error.inner_ref().to_string(), "queue full");
    assert!(format!("{:?}", error).contains("\t- push failed, queue len 2\n"));
}

#[tokio::test]
async fn test_async_mut_ref_ok_mutates_state() {
    let mut q = AsyncQueue { items: vec![] };
    q.push("first".into()).await.unwrap();
    assert_eq!(q.items, vec!["first"]);
}

// ── Context string formatting edge cases ────────────────────────────────────

#[eros_macros::context("no format args at all")]
fn no_args_function() -> eros::Result<()> {
    eros::bail!("bare error")
}

#[test]
fn test_static_context_string_no_args() {
    let error = no_args_function().unwrap_err();
    assert!(format!("{:?}", error).contains("\t- no format args at all\n"));
}

#[eros_macros::context("a={} b={} c={}", a, b, c)]
fn three_arg_function(a: u8, b: u8, c: u8) -> eros::Result<()> {
    eros::bail!("three args error")
}

#[test]
fn test_context_with_three_format_args() {
    let error = three_arg_function(1, 2, 3).unwrap_err();
    assert!(format!("{:?}", error).contains("\t- a=1 b=2 c=3\n"));
}

#[eros_macros::context("debug value is {:?}", value)]
fn debug_format_function(value: &Vec<u8>) -> eros::Result<()> {
    eros::bail!("debug format error")
}

#[test]
fn test_context_with_debug_format_specifier() {
    let error = debug_format_function(&vec![1, 2, 3]).unwrap_err();
    assert!(format!("{:?}", error).contains("[1, 2, 3]"));
}
