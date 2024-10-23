use assert_cmd::Command;

#[test]
fn integration() {
    // Most things are already tested in unit tests, so only minimal tests are performed in integration tests
    // Verify that the command generates a password and that it does not duplicate when re-executed
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.assert().success();

    let password = String::from_utf8(cmd.output().unwrap().stdout).unwrap();

    assert!(cmd.output().unwrap().status.success());
    assert_eq!(password.len(), 16);

    // Checks if it contains at least one uppercase letter, one lowercase letter, one digit, and one symbol
    assert!(password.chars().any(|c| c.is_ascii_uppercase()));
    assert!(password.chars().any(|c| c.is_ascii_lowercase()));
    assert!(password.chars().any(|c| c.is_ascii_digit()));
    assert!(password.chars().any(|c| !c.is_ascii_alphanumeric()));

    // Check if consecutive generated passwords are not duplicated
    let mut cmd2 = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd2.assert().success();

    let password2 = String::from_utf8(cmd2.output().unwrap().stdout).unwrap();

    assert_ne!(password, password2);
}

#[test]
fn integration_error() {
    // Check if an error occurs
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args(["--length", "0"]).assert().failure();
}
