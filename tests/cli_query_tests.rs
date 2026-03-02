//! Integration tests for CLI query commands
//!
//! Tests: CLI-01, CLI-02, CLI-03

use assert_cmd::Command;

/// Helper to run astro-clock CLI
fn astro_clock() -> Command {
    let mut cmd = Command::cargo_bin("astro-clock").unwrap();
    cmd.env("RUST_LOG", "error"); // Reduce noise
    cmd
}

#[test]
fn test_query_wedding_help() {
    let output = astro_clock()
        .args(&["query", "wedding", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--start"));
    assert!(stdout.contains("--days"));
    assert!(stdout.contains("--sync"));
}

#[test]
fn test_query_project_help() {
    let output = astro_clock()
        .args(&["query", "project", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--start"));
    assert!(stdout.contains("--days"));
}

#[test]
fn test_query_travel_help() {
    let output = astro_clock()
        .args(&["query", "travel", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--start"));
    assert!(stdout.contains("--days"));
}

#[test]
fn test_query_wedding_invalid_date() {
    let output = astro_clock()
        .args(&[
            "query",
            "wedding",
            "--start",
            "invalid-date",
            "--days",
            "30",
        ])
        .output()
        .expect("Failed to execute command");

    // Should fail with error about invalid date format
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Invalid date format") || stderr.contains("error"),
        "Expected error message about invalid date, got: {}",
        stderr
    );
}

#[test]
fn test_query_project_invalid_days() {
    let output = astro_clock()
        .args(&["query", "project", "--start", "2024-06-01", "--days", "0"])
        .output()
        .expect("Failed to execute command");

    // Should fail with error about invalid days
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Days must be") || stderr.contains("error"),
        "Expected error message about invalid days, got: {}",
        stderr
    );
}

#[test]
fn test_query_travel_days_too_large() {
    let output = astro_clock()
        .args(&["query", "travel", "--start", "2024-06-01", "--days", "500"])
        .output()
        .expect("Failed to execute command");

    // Should fail with error about days exceeding limit
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Days must be") || stderr.contains("error"),
        "Expected error message about days limit, got: {}",
        stderr
    );
}
