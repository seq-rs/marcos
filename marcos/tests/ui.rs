#[test]
fn parse_attributes_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass_*.rs");
}

#[test]
fn parse_attributes_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail_*.rs");
}
