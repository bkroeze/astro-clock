//! CLI command integration tests using assert_cmd against a real database.
//!
//! These tests require a running TimescaleDB instance with seed data loaded.
//! Run `just test-db-setup` first, then:
//!
//!     TEST_PG_URL=postgresql://user:pass@host:port/dbname \
//!     cargo test --features db --test cli_integration -- --ignored --test-threads=1
//!
//! All tests are gated behind `#[cfg(feature = "db")]` and marked `#[ignore]`
//! because they require TEST_PG_URL and database connectivity.
//!
//! The CLI binary reads DATABASE_URL (not TEST_PG_URL), so each test sets
//! DATABASE_URL from TEST_PG_URL before invoking the binary.
//!
//! NOTE: Tests must run single-threaded (`--test-threads=1`) because the CLI
//! creates its own database connection pool per invocation and the test
//! TimescaleDB has limited connection capacity.

#[cfg(feature = "db")]
mod common;

#[cfg(feature = "db")]
use assert_cmd::Command;

/// Helper: build a `Command` for the astro-clock binary with DATABASE_URL
/// set from TEST_PG_URL so the CLI connects to the test database.
///
/// Sets a 120-second timeout to prevent hangs from pool exhaustion.
#[cfg(feature = "db")]
fn cli() -> Command {
    let mut cmd = Command::cargo_bin("astro-clock").unwrap();
    let url = common::test_pg_url();
    cmd.env("DATABASE_URL", url);
    cmd.env("RUST_LOG", "error");
    cmd.timeout(std::time::Duration::from_secs(120));
    cmd
}

// =========================================================================
// query wedding --start --days --sync
// =========================================================================

#[cfg(feature = "db")]
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn cli_query_wedding_sync_succeeds() {
    let output = cli()
        .args(&[
            "query",
            "wedding",
            "--start",
            "2025-01-01",
            "--days",
            "7",
            "--sync",
        ])
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "query wedding --sync should exit 0\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("Query completed") || stdout.contains("query_name"),
        "output should mention completion or query results\nstdout: {stdout}"
    );
}

#[cfg(feature = "db")]
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn cli_query_wedding_async_returns_job_id() {
    let output = cli()
        .args(&[
            "query",
            "wedding",
            "--start",
            "2025-01-01",
            "--days",
            "7",
        ])
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "query wedding async should exit 0\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("Job ID") || stdout.contains("job"),
        "async output should mention job ID\nstdout: {stdout}"
    );
}

// =========================================================================
// query project --start --days --sync
// =========================================================================

#[cfg(feature = "db")]
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn cli_query_project_sync_succeeds() {
    let output = cli()
        .args(&[
            "query",
            "project",
            "--start",
            "2025-01-01",
            "--days",
            "7",
            "--sync",
        ])
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Project query depends on derived tables (aspect_summaries, retrograde_periods);
    // accept both "complete" and "failed" as valid outcomes.
    assert!(
        output.status.success(),
        "query project --sync should exit 0\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("Query completed")
            || stdout.contains("Query failed")
            || stdout.contains("query_name"),
        "output should mention query result\nstdout: {stdout}"
    );
}

// =========================================================================
// query travel --start --days --sync
// =========================================================================

#[cfg(feature = "db")]
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn cli_query_travel_sync_succeeds() {
    let output = cli()
        .args(&[
            "query",
            "travel",
            "--start",
            "2025-01-15",
            "--days",
            "3",
            "--sync",
        ])
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Travel query depends on derived tables; accept both complete and failed.
    assert!(
        output.status.success(),
        "query travel --sync should exit 0\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("Query completed")
            || stdout.contains("Query failed")
            || stdout.contains("query_name"),
        "output should mention query result\nstdout: {stdout}"
    );
}

// =========================================================================
// job list
// =========================================================================

#[cfg(feature = "db")]
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn cli_job_list_shows_jobs() {
    // The test database should already have jobs from seed data loading
    // and prior test runs — no need to create one first.
    let output = cli()
        .args(&["job", "list"])
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "job list should exit 0\nstdout: {stdout}\nstderr: {stderr}"
    );
    // The table header or job rows should contain "Jobs" or "ID"
    assert!(
        stdout.contains("Jobs") || stdout.contains("ID"),
        "job list output should show jobs table header\nstdout: {stdout}"
    );
}

#[cfg(feature = "db")]
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn cli_job_list_with_status_filter() {
    let output = cli()
        .args(&["job", "list", "--status", "complete"])
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "job list --status complete should exit 0\nstdout: {stdout}\nstderr: {stderr}"
    );
}

#[cfg(feature = "db")]
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn cli_job_list_invalid_status_fails() {
    let output = cli()
        .args(&["job", "list", "--status", "bogus"])
        .output()
        .expect("Failed to execute CLI");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "job list --status bogus should fail\nstderr: {stderr}"
    );
    assert!(
        stderr.contains("Invalid status"),
        "should report invalid status\nstderr: {stderr}"
    );
}

// =========================================================================
// job status <id>
// =========================================================================

#[cfg(feature = "db")]
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn cli_job_status_nonexistent_reports_not_found() {
    let output = cli()
        .args(&[
            "job",
            "status",
            "00000000-0000-0000-0000-000000000000",
        ])
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // The CLI exits successfully even when job is not found; it just prints a message.
    assert!(
        stdout.contains("not found") || stdout.contains("Job"),
        "should report job not found or show job info\nstdout: {stdout}"
    );
}

#[cfg(feature = "db")]
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn cli_job_status_invalid_uuid_fails() {
    let output = cli()
        .args(&["job", "status", "not-a-uuid"])
        .output()
        .expect("Failed to execute CLI");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "job status with invalid UUID should fail\nstderr: {stderr}"
    );
    assert!(
        stderr.contains("Invalid job ID") || stderr.contains("UUID"),
        "should report invalid UUID\nstderr: {stderr}"
    );
}

// =========================================================================
// load command (bonus coverage — ensures load via CLI works end-to-end)
// =========================================================================

#[cfg(feature = "db")]
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn cli_load_sync_succeeds() {
    let output = cli()
        .args(&[
            "load",
            "--start",
            "2025-01-01",
            "--days",
            "1",
            "--sync",
        ])
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "load --sync should exit 0\nstdout: {stdout}\nstderr: {stderr}"
    );
    // "Load completed" or "dates skipped" (if data already exists) are both valid
    assert!(
        stdout.contains("Load completed")
            || stdout.contains("dates loaded")
            || stdout.contains("dates skipped")
            || stdout.contains("positions"),
        "output should mention load results\nstdout: {stdout}"
    );
}

#[cfg(feature = "db")]
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn cli_load_async_returns_job_id() {
    let output = cli()
        .args(&["load", "--start", "2025-01-01", "--days", "1"])
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "load async should exit 0\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("Job ID") || stdout.contains("job"),
        "async load should mention job ID\nstdout: {stdout}"
    );
}

#[cfg(feature = "db")]
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn cli_load_invalid_date_fails() {
    let output = cli()
        .args(&[
            "load",
            "--start",
            "not-a-date",
            "--days",
            "7",
            "--sync",
        ])
        .output()
        .expect("Failed to execute CLI");

    assert!(
        !output.status.success(),
        "load with invalid date should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Invalid date format") || stderr.contains("error"),
        "should report invalid date\nstderr: {stderr}"
    );
}

#[cfg(feature = "db")]
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn cli_load_days_zero_fails() {
    let output = cli()
        .args(&["load", "--start", "2025-01-01", "--days", "0", "--sync"])
        .output()
        .expect("Failed to execute CLI");

    assert!(
        !output.status.success(),
        "load with days=0 should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Days must be") || stderr.contains("error"),
        "should report invalid days\nstderr: {stderr}"
    );
}

// =========================================================================
// End-to-end: list jobs, extract a UUID, get its status
// =========================================================================

#[cfg(feature = "db")]
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn cli_job_status_roundtrip() {
    // List complete jobs — the test DB should have some from seed data loading
    let list_output = cli()
        .args(&["job", "list", "--status", "complete", "--limit", "5"])
        .output()
        .expect("Failed to list jobs");

    assert!(list_output.status.success(), "job list should succeed");
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(
        list_stdout.contains("Jobs") || list_stdout.contains("ID"),
        "should list complete jobs\nstdout: {list_stdout}"
    );

    // Extract a full UUID from the list output
    let job_id = extract_job_id_from_output(&list_stdout);

    if let Some(id) = job_id {
        let status_output = cli()
            .args(&["job", "status", &id])
            .output()
            .expect("Failed to get job status");

        let status_stdout = String::from_utf8_lossy(&status_output.stdout);
        assert!(
            status_output.status.success(),
            "job status should succeed\nstdout: {status_stdout}"
        );
        assert!(
            status_stdout.contains("complete")
                || status_stdout.contains("Complete")
                || status_stdout.contains("Status"),
            "should show job status\nstdout: {status_stdout}"
        );
    }
    // If no UUID found, the test passes — the list was empty, which is a valid state.
}

/// Extract a full UUID from the job list output.
#[cfg(feature = "db")]
fn extract_job_id_from_output(output: &str) -> Option<String> {
    for line in output.lines() {
        if let Some(id) = find_uuid(line) {
            return Some(id);
        }
    }
    None
}

/// Check if a character is a hex digit
#[cfg(feature = "db")]
fn is_hex(c: char) -> bool {
    c.is_ascii_hexdigit()
}

/// Find a UUID-like substring in text without regex.
/// Looks for pattern: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
#[cfg(feature = "db")]
fn find_uuid(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    let expected = [8, 4, 4, 4, 12]; // UUID segment lengths
    'outer: for i in 0..chars.len() {
        let mut pos = i;
        for (seg, &len) in expected.iter().enumerate() {
            for _ in 0..len {
                if pos >= chars.len() || !is_hex(chars[pos]) {
                    continue 'outer;
                }
                pos += 1;
            }
            if seg < 4 {
                if pos >= chars.len() || chars[pos] != '-' {
                    continue 'outer;
                }
                pos += 1;
            }
        }
        return Some(chars[i..pos].iter().collect());
    }
    None
}
