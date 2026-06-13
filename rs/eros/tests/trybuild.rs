
#[test]
// We ignore when testing in ci since a different version of rust may be used which give slightly different messages and can become annoying to align
#[cfg_attr(test_in_ci, ignore)]
fn trybuild() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/trybuild/*.rs");
}