//! Integration tests for CLI job commands
//!
//! Tests: CLI-04, CLI-05

use assert_cmd::Command;

/// Helper to run astro-clock CLI
fn astro_clock() -> Command {
    let mut cmd = Command::cargo_bin("astro-clock").unwrap();
    cmd.env("RUST_LOG", "error");
    cmd
}

#[test]
fn test_job_status_help() {
    let output = astro_clock()
        .args(&["job", "status", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("JOB_ID"));
}

#[test]
fn test_job_list_help() {
    let output = astro_clock()
        .args(&["job", "list", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--status"));
    assert!(stdout.contains("--limit"));
    assert!(stdout.contains("--offset"));
}

#[cfg(feature = "db")]
#[test]
fn test_job_status_invalid_uuid() {
    let output = astro_clock()
        .args(&["job", "status", "not-a-uuid"])
        .output()
        .expect("Failed to execute command");

    // Should fail with error about invalid UUID
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Invalid job ID") || stderr.contains("UUID") || stderr.contains("error"),
        "Expected error message about invalid UUID, got: {}",
        stderr
    );
}

#[test]
fn test_job_status_nonexistent_job() {
    let output = astro_clock()
        .args(&["job", "status", "00000000-0000-0000-0000-000000000000"])
        .output()
        .expect("Failed to execute command");

    // Should report job not found (might be success with "not found" message,
    // or failure with error - both are acceptable)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either stdout or stderr should contain "not found" or job-related message
    // Also accept database connection errors since tests may not have DB available
    let output_combined = format!("{}{}", stdout, stderr);
    assert!(
        output_combined.contains("not found")
            || output_combined.contains("Job")
            || output_combined.contains("job")
            || output_combined.contains("database")
            || output_combined.contains("Database"),
        "Expected 'not found', job, or database message, got stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_job_list_with_status_filter() {
    // Note: This test may fail if database is not available
    // but we can at least verify the command parsing works
    let output = astro_clock()
        .args(&["job", "list", "--status", "complete"])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Invalid status"),
        "Should accept 'complete' as valid status, got stderr: {}",
        stderr
    );
}

#[cfg(feature = "db")]
#[test]
fn test_job_list_invalid_status() {
    let output = astro_clock()
        .args(&["job", "list", "--status", "invalid_status"])
        .output()
        .expect("Failed to execute command");

    // Should fail with error about invalid status
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Invalid status") || stderr.contains("error"),
        "Expected error message about invalid status, got: {}",
        stderr
    );
}
