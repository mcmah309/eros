#[test]
#[cfg_attr(miri, ignore)]
fn trybuild() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/trybuild/*.rs");
    t.pass("tests/trybuild/pass/*.rs");
}
