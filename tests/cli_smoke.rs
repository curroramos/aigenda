use assert_cmd::Command;

#[test]
fn runs_add_and_list() {
    let mut cmd = Command::cargo_bin("aigenda").unwrap();
    cmd.args(["add", "hello"]).assert().success();

    let mut cmd = Command::cargo_bin("aigenda").unwrap();
    cmd.args(["list"]).assert().success();
}
