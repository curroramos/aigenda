use assert_cmd::Command;

#[test]
fn runs_add_and_list() {
    // just verifies the binary runs; full isolation comes later
    let mut cmd = Command::cargo_bin("aigenda").unwrap();
    cmd.args(["add", "hello"]).assert().success();

    let mut cmd = Command::cargo_bin("aigenda").unwrap();
    cmd.args(["list"]).assert().success();
}
