# T03: 05-job-infrastructure 03

**Slice:** S05 — **Milestone:** M001

## Description

Create JobExecutor that orchestrates job lifecycle with both synchronous and asynchronous execution modes.

Purpose: Provide the core orchestration layer that manages job execution, handles state transitions, and supports both blocking (CLI) and non-blocking (API) execution patterns. The executor bridges the job repository with handler implementations using spawn_blocking for CPU-intensive work to avoid blocking the async runtime.

Output: src/jobs/executor.rs with JobExecutor, JobHandler trait, and complete sync/async execution implementations.

## Must-Haves

- [ ] "JobExecutor orchestrates job lifecycle with proper state transitions"
- [ ] "Synchronous execution blocks until job completes and returns result directly"
- [ ] "Asynchronous execution returns job-id immediately and runs job in background"
- [ ] "spawn_blocking + oneshot channel pattern used for CPU-intensive work"
- [ ] "Job handlers can be registered and executed by the executor"

## Files

- `src/jobs/executor.rs`
- `src/jobs/mod.rs`
- `src/lib.rs`
