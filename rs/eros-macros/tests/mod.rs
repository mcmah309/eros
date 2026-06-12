#[eros_macros::context("arg1 is {}", arg1)]
fn test_function(arg1: &str, arg2: String, arg3: i32) -> eros::Result<String> {
    eros::bail!("This is the error")
}

#[test]
fn test() {
    let error = test_function("test", "arg2".to_owned(), 42).unwrap_err();
    let inner_error = error.error_ref();
    assert_eq!(inner_error.to_string(), "This is the error");
    assert!(format!("{:?}", error).contains("\t- arg1 is test\n"));
}
