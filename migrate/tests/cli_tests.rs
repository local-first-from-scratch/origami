#[test]
fn cli_tests() {
    trycmd::TestCases::new().case("tests/cli/*.md");
}
