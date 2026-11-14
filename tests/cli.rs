//! Informational commands must work on release runners without a desktop.
use std::process::Command;

#[test]
fn help_identifies_wgpu_without_opening_a_window() {
    let output = Command::new(env!("CARGO_BIN_EXE_battlezone"))
        .arg("--help")
        .output()
        .expect("run built executable");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("launches the game in a wgpu window"));
}

#[test]
fn version_matches_the_package_for_release_tag_checks() {
    let output = Command::new(env!("CARGO_BIN_EXE_battlezone"))
        .arg("--version")
        .output()
        .expect("run built executable");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        format!("battlezone {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn unknown_option_exits_with_an_error_without_opening_a_window() {
    let output = Command::new(env!("CARGO_BIN_EXE_battlezone"))
        .arg("--unknown")
        .output()
        .expect("run built executable");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown option"));
}
