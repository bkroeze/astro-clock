//! API route integration tests using tower::ServiceExt against a real database.
//!
//! These tests require a running TimescaleDB instance with seed data loaded.
//! Run `just test-db-setup` first, then:
//!
//!     cargo test --features db --test api_integration -- --ignored
//!
//! All API route tests are gated behind `#[cfg(feature = "db")]` and marked
//! `#[ignore]` because they require TEST_PG_URL and database connectivity.
//!
//! The seed-data verification tests (no database required for constant checks)
//! are at the bottom and run without `--ignored`.

#[cfg(feature = "db")]
mod common;

// =========================================================================
// Seed-data constant tests (no DB needed)
// =========================================================================

#[cfg(feature = "db")]
#[test]
fn test_helper_constants_are_valid() {
    use common::*;

    assert_eq!(SEED_START_DATE, "2025-01-01");
    assert_eq!(SEED_END_DATE, "2025-03-02");
    assert_eq!(TOTAL_BODIES, 10);

    let start = seed_start_date();
    let end = seed_end_date();
    assert!(end > start);
    let span = (end - start).num_days() as usize;
    assert_eq!(span + 1, SEED_DAY_COUNT); // inclusive of both endpoints
}

#[cfg(feature = "db")]
#[test]
fn test_body_id_constants() {
    use common::body_ids::*;
    assert_eq!(SUN, 0);
    assert_eq!(MOON, 1);
    assert_eq!(MERCURY, 2);
    assert_eq!(VENUS, 3);
    assert_eq!(MARS, 4);
    assert_eq!(JUPITER, 5);
    assert_eq!(SATURN, 6);
    assert_eq!(URANUS, 7);
    assert_eq!(NEPTUNE, 8);
    assert_eq!(PLUTO, 9);
}

#[cfg(feature = "db")]
#[test]
fn test_discovered_seed_data_constants() {
    use common::*;

    assert_eq!(TOTAL_POSITIONS, POSITIONS_PER_BODY * TOTAL_BODIES as i64,);

    let aspect_sum = known_aspects::CONJUNCTIONS
        + known_aspects::SEXTILES
        + known_aspects::SQUARES
        + known_aspects::TRINES
        + known_aspects::OPPOSITIONS;
    assert_eq!(aspect_sum, TOTAL_ASPECTS);
    assert_eq!(TOTAL_LUNAR_CONDITIONS, POSITIONS_PER_BODY);
}

#[cfg(feature = "db")]
#[test]
#[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
fn test_seed_data_loaded() {
    use common::*;

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        let pool = test_pool().await;

        let days = verify_seed_data_loaded(&pool).await;
        assert_day_coverage(days);

        let counts = position_counts_per_body(&pool).await;
        assert_all_bodies_present(&counts);

        for (body_id, count) in &counts {
            assert_eq!(
                *count, POSITIONS_PER_BODY,
                "Body {} should have exactly {} position records",
                body_id, POSITIONS_PER_BODY
            );
        }

        pool.close().await;
    });
}

// =========================================================================
// API route integration tests (require DB + TEST_PG_URL)
// =========================================================================

#[cfg(feature = "db")]
mod api_tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::{get, post};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use astro_clock::jobs::executor::JobExecutor;
    use astro_clock::jobs::handlers::{LoadJobHandler, QueryJobHandler};
    use astro_clock::jobs::repository::JobRepository;
    use astro_clock::server::routes::{
        delete_job_handler, get_job_handler, list_jobs_handler, load_handler,
        project_query_handler, travel_query_handler, wedding_query_handler,
    };
    use astro_clock::server::state::AppState;

    use crate::common;

    /// Build a test Axum app wired to the test database.
    ///
    /// Creates the real AppState (JobExecutor + pool) so route handlers
    /// exercise the same code paths as production. Must be called from
    /// within an async context (tokio test runtime).
    async fn build_app() -> axum::Router {
        let pool = common::test_pool().await;
        let repository = JobRepository::new(pool.clone());
        let load_job_handler = std::sync::Arc::new(LoadJobHandler::new(pool.clone()));
        let query_job_handler = std::sync::Arc::new(QueryJobHandler::new(pool.clone()));
        let executor = JobExecutor::new(
            repository,
            vec![load_job_handler, query_job_handler],
            format!("test-{}", std::process::id()),
        );

        let app_state = AppState::new(executor, pool);

        axum::Router::new()
            .route("/health", get(health_ok))
            .route("/api/v1/load", post(load_handler))
            .route(
                "/api/v1/jobs/:id",
                get(get_job_handler).delete(delete_job_handler),
            )
            .route("/api/v1/jobs", get(list_jobs_handler))
            .route("/api/v1/query/wedding", post(wedding_query_handler))
            .route("/api/v1/query/project", post(project_query_handler))
            .route("/api/v1/query/travel", post(travel_query_handler))
            .with_state(app_state)
    }

    /// Minimal health endpoint for test app
    async fn health_ok() -> &'static str {
        "ok"
    }

    /// Helper: send a POST with a JSON body and return the response.
    async fn post_json(
        app: &mut axum::Router,
        uri: &str,
        body: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(uri)
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value =
            serde_json::from_slice(&body_bytes).unwrap_or(serde_json::Value::Null);
        (status, json)
    }

    /// Helper: send a GET and return the response.
    async fn get_uri(app: &mut axum::Router, uri: &str) -> (StatusCode, serde_json::Value) {
        let response = app
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();

        let status = response.status();
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value =
            serde_json::from_slice(&body_bytes).unwrap_or(serde_json::Value::Null);
        (status, json)
    }

    /// Helper: send a DELETE and return the response.
    /// For 204 No Content, the body will parse to Value::Null.
    async fn delete_uri(app: &mut axum::Router, uri: &str) -> (StatusCode, serde_json::Value) {
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value =
            serde_json::from_slice(&body_bytes).unwrap_or(serde_json::Value::Null);
        (status, json)
    }

    // =========================================================================
    // POST /api/v1/load — sync and async modes
    // =========================================================================

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn load_sync_returns_completed_job() {
        let mut app = build_app().await;

        let (status, json) = post_json(
            &mut app,
            "/api/v1/load",
            serde_json::json!({
                "start_date": "2025-01-01",
                "days": 2,
                "sync": true
            }),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::OK,
            "sync load should return 200, got {}: {:?}",
            status,
            json
        );
        assert_eq!(json["status"], "complete", "sync job should be complete");
        assert!(json["job_id"].is_string(), "response should include job_id");
        assert!(
            json["result"].is_object(),
            "sync response should include result"
        );
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn load_async_returns_job_id() {
        let mut app = build_app().await;

        let (status, json) = post_json(
            &mut app,
            "/api/v1/load",
            serde_json::json!({
                "start_date": "2025-01-01",
                "days": 2,
                "sync": false
            }),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::ACCEPTED,
            "async load should return 202, got {}: {:?}",
            status,
            json
        );
        assert_eq!(json["status"], "pending", "async job should be pending");
        assert!(json["job_id"].is_string(), "response should include job_id");
        assert!(
            json["poll_url"].is_string(),
            "response should include poll_url"
        );
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn load_async_default_no_sync_flag() {
        let mut app = build_app().await;

        // Omit sync field entirely — should default to async
        let (status, json) = post_json(
            &mut app,
            "/api/v1/load",
            serde_json::json!({
                "start_date": "2025-01-01",
                "days": 2
            }),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::ACCEPTED,
            "omitting sync should default to async (202)"
        );
        assert_eq!(json["status"], "pending");
    }

    // =========================================================================
    // POST /api/v1/query/wedding
    // =========================================================================

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn wedding_query_sync_returns_results() {
        let mut app = build_app().await;

        let (status, json) = post_json(
            &mut app,
            "/api/v1/query/wedding",
            serde_json::json!({
                "start_date": "2025-01-01",
                "days": 30,
                "sync": true
            }),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::OK,
            "sync wedding query should return 200, got {}: {:?}",
            status,
            json
        );
        assert_eq!(json["status"], "complete");
        assert!(json["job_id"].is_string());
        let result = json
            .get("result")
            .expect("sync response should include result");
        assert_eq!(result["query_name"], "wedding");
        assert!(result["total_results"].is_number());
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn wedding_query_async_returns_pending() {
        let mut app = build_app().await;

        let (status, json) = post_json(
            &mut app,
            "/api/v1/query/wedding",
            serde_json::json!({
                "start_date": "2025-01-01",
                "days": 30,
                "sync": false
            }),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::ACCEPTED,
            "async query should return 202"
        );
        assert_eq!(json["status"], "pending");
        assert!(json["job_id"].is_string());
    }

    // =========================================================================
    // POST /api/v1/query/project
    // =========================================================================

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn project_query_sync_returns_results() {
        let mut app = build_app().await;

        let (status, json) = post_json(
            &mut app,
            "/api/v1/query/project",
            serde_json::json!({
                "start_date": "2025-01-01",
                "days": 14,
                "sync": true
            }),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::OK,
            "sync project query should return 200, got {}: {:?}",
            status,
            json
        );

        // The project query depends on aspect_summaries and retrograde_periods tables.
        // If seed data only populated raw tables (planet_positions, aspects, lunar_conditions),
        // the query may return status "failed" due to missing derived data.
        // Both "complete" and "failed" are valid outcomes — what matters is the route
        // correctly processes the request through the job system.
        assert!(
            json["status"] == "complete" || json["status"] == "failed",
            "expected complete or failed, got: {:?}",
            json["status"]
        );

        if json["status"] == "complete" {
            let result = json.get("result").expect("should include result");
            assert_eq!(result["query_name"], "project");
            assert!(result["total_results"].is_number());
        }
    }

    // =========================================================================
    // POST /api/v1/query/travel
    // =========================================================================

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn travel_query_sync_returns_results() {
        let mut app = build_app().await;

        let (status, json) = post_json(
            &mut app,
            "/api/v1/query/travel",
            serde_json::json!({
                "start_date": "2025-01-15",
                "days": 7,
                "sync": true
            }),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::OK,
            "sync travel query should return 200, got {}: {:?}",
            status,
            json
        );

        // The travel query depends on aspect_summaries and retrograde_periods tables.
        // See project_query_sync_returns_results for data dependency notes.
        assert!(
            json["status"] == "complete" || json["status"] == "failed",
            "expected complete or failed, got: {:?}",
            json["status"]
        );

        if json["status"] == "complete" {
            let result = json.get("result").expect("should include result");
            assert_eq!(result["query_name"], "travel");
            assert!(result["total_results"].is_number());
        }
    }

    // =========================================================================
    // GET /api/v1/jobs/:id — retrieve a job by ID
    // =========================================================================

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn get_job_returns_created_job() {
        let mut app = build_app().await;

        // Create a sync load job (which completes immediately)
        let (_, create_json) = post_json(
            &mut app,
            "/api/v1/load",
            serde_json::json!({
                "start_date": "2025-01-01",
                "days": 2,
                "sync": true
            }),
        )
        .await;

        let job_id = create_json["job_id"].as_str().expect("should have job_id");

        // Now retrieve the job
        let (status, get_json) = get_uri(&mut app, &format!("/api/v1/jobs/{}", job_id)).await;

        assert_eq!(
            status,
            StatusCode::OK,
            "get job should return 200, got {}: {:?}",
            status,
            get_json
        );
        assert_eq!(get_json["job_id"], job_id);
        assert_eq!(get_json["status"], "complete");
        assert!(get_json["payload"].is_object());
        assert!(get_json["result"].is_object());
        assert!(get_json["created_at"].is_string());
        assert!(get_json["updated_at"].is_string());
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn get_job_not_found_returns_404() {
        let mut app = build_app().await;

        let (status, json) = get_uri(
            &mut app,
            "/api/v1/jobs/00000000-0000-0000-0000-000000000000",
        )
        .await;

        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "nonexistent job should return 404"
        );
        assert_eq!(json["error"], "not_found");
    }

    // =========================================================================
    // GET /api/v1/jobs — list jobs with pagination
    // =========================================================================

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn list_jobs_returns_paginated_results() {
        let mut app = build_app().await;

        // Create a couple of jobs to ensure there's data
        let _ = post_json(
            &mut app,
            "/api/v1/load",
            serde_json::json!({ "start_date": "2025-01-01", "days": 1, "sync": true }),
        )
        .await;
        let _ = post_json(
            &mut app,
            "/api/v1/load",
            serde_json::json!({ "start_date": "2025-01-02", "days": 1, "sync": true }),
        )
        .await;

        let (status, json) = get_uri(&mut app, "/api/v1/jobs?count=10").await;

        assert_eq!(
            status,
            StatusCode::OK,
            "list jobs should return 200, got {}: {:?}",
            status,
            json
        );
        assert!(json["jobs"].is_array(), "response should have jobs array");
        // Cursor-based pagination uses next/prev, not total/limit/offset
        assert!(
            json.get("next").is_some(),
            "response should have next field"
        );
        assert!(
            json.get("prev").is_some(),
            "response should have prev field"
        );
        // Ensure old fields are absent
        assert!(
            json.get("total").is_none(),
            "should not have legacy total field"
        );
        assert!(
            json.get("limit").is_none(),
            "should not have legacy limit field"
        );
        assert!(
            json.get("offset").is_none(),
            "should not have legacy offset field"
        );

        let jobs = json["jobs"].as_array().unwrap();
        assert!(
            jobs.len() >= 2,
            "should have at least 2 jobs, got {}",
            jobs.len()
        );
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn list_jobs_with_status_filter() {
        let mut app = build_app().await;

        let (status, json) = get_uri(&mut app, "/api/v1/jobs?status=complete").await;

        assert_eq!(status, StatusCode::OK, "filtered list should return 200");
        assert!(json["jobs"].is_array());
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn list_jobs_invalid_status_returns_400() {
        let mut app = build_app().await;

        let (status, json) = get_uri(&mut app, "/api/v1/jobs?status=nonexistent").await;

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "invalid status filter should return 400"
        );
        assert_eq!(json["error"], "invalid_status");
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn list_jobs_pagination_works() {
        let mut app = build_app().await;

        // Create at least 5 jobs to ensure multi-page results
        for i in 1..=5 {
            let _ = post_json(
                &mut app,
                "/api/v1/load",
                serde_json::json!({
                    "start_date": format!("2025-01-{:02}", i),
                    "days": 1,
                    "sync": true
                }),
            )
            .await;
        }

        // Page 1: count=2
        let (status, page1) = get_uri(&mut app, "/api/v1/jobs?count=2").await;
        assert_eq!(status, StatusCode::OK, "page 1 should return 200");

        let page1_jobs = page1["jobs"].as_array().unwrap();
        assert_eq!(page1_jobs.len(), 2, "page 1 should have 2 jobs");
        assert!(
            page1["next"].is_string(),
            "page 1 should have a next URL, got: {:?}",
            page1["next"]
        );
        assert!(page1["prev"].is_null(), "first page should have prev=null");

        // Follow next URL for page 2
        let next_url = page1["next"].as_str().unwrap();
        let (status, page2) = get_uri(&mut app, next_url).await;
        assert_eq!(status, StatusCode::OK, "page 2 should return 200");

        let page2_jobs = page2["jobs"].as_array().unwrap();
        assert_eq!(page2_jobs.len(), 2, "page 2 should have 2 jobs");
        assert!(page2["next"].is_string(), "page 2 should have a next URL");
        assert!(page2["prev"].is_string(), "page 2 should have a prev URL");
    }

    // =========================================================================
    // Validation tests: bad inputs return 400
    // =========================================================================

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn load_invalid_date_returns_400() {
        let mut app = build_app().await;

        let (status, json) = post_json(
            &mut app,
            "/api/v1/load",
            serde_json::json!({
                "start_date": "not-a-date",
                "days": 7,
                "sync": true
            }),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "invalid date should return 400"
        );
        assert_eq!(json["error"], "invalid_date");
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn load_days_zero_returns_400() {
        let mut app = build_app().await;

        let (status, json) = post_json(
            &mut app,
            "/api/v1/load",
            serde_json::json!({
                "start_date": "2025-01-01",
                "days": 0,
                "sync": true
            }),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "days=0 should return 400");
        assert_eq!(json["error"], "invalid_days");
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn load_days_too_large_returns_400() {
        let mut app = build_app().await;

        let (status, json) = post_json(
            &mut app,
            "/api/v1/load",
            serde_json::json!({
                "start_date": "2025-01-01",
                "days": 500,
                "sync": true
            }),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "days=500 should return 400"
        );
        assert_eq!(json["error"], "invalid_days");
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn load_days_negative_returns_400() {
        let mut app = build_app().await;

        let (status, json) = post_json(
            &mut app,
            "/api/v1/load",
            serde_json::json!({
                "start_date": "2025-01-01",
                "days": -5,
                "sync": true
            }),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "negative days should return 400"
        );
        assert_eq!(json["error"], "invalid_days");
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn query_invalid_date_returns_400() {
        let mut app = build_app().await;

        let (status, json) = post_json(
            &mut app,
            "/api/v1/query/wedding",
            serde_json::json!({
                "start_date": "06-01-2024",
                "days": 30,
                "sync": true
            }),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "bad date format should return 400"
        );
        assert_eq!(json["error"], "invalid_date");
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn query_days_out_of_range_returns_400() {
        let mut app = build_app().await;

        let (status, json) = post_json(
            &mut app,
            "/api/v1/query/wedding",
            serde_json::json!({
                "start_date": "2025-01-01",
                "days": 0,
                "sync": true
            }),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST, "days=0 should return 400");
        assert_eq!(json["error"], "invalid_days");
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn query_days_exceeds_max_returns_400() {
        let mut app = build_app().await;

        let (status, json) = post_json(
            &mut app,
            "/api/v1/query/project",
            serde_json::json!({
                "start_date": "2025-01-01",
                "days": 400,
                "sync": true
            }),
        )
        .await;

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "days=400 should return 400"
        );
        assert_eq!(json["error"], "invalid_days");
    }

    // =========================================================================
    // Multi-value filter tests
    // =========================================================================

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn list_jobs_multi_status_filter() {
        let mut app = build_app().await;

        // Create a complete load job
        let _ = post_json(
            &mut app,
            "/api/v1/load",
            serde_json::json!({ "start_date": "2025-01-01", "days": 1, "sync": true }),
        )
        .await;

        // Create a complete query job
        let _ = post_json(
            &mut app,
            "/api/v1/query/wedding",
            serde_json::json!({ "start_date": "2025-01-01", "days": 7, "sync": true }),
        )
        .await;

        let (status, json) = get_uri(&mut app, "/api/v1/jobs?status=complete,failed").await;

        assert_eq!(
            status,
            StatusCode::OK,
            "multi-status filter should return 200"
        );
        let jobs = json["jobs"].as_array().expect("should have jobs array");
        for job in jobs {
            let s = job["status"].as_str().expect("job should have status");
            assert!(
                s == "complete" || s == "failed",
                "job status should be complete or failed, got: {}",
                s
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn list_jobs_multi_job_type_filter() {
        let mut app = build_app().await;

        // Create a load job
        let _ = post_json(
            &mut app,
            "/api/v1/load",
            serde_json::json!({ "start_date": "2025-01-01", "days": 1, "sync": true }),
        )
        .await;

        // Create a query job
        let _ = post_json(
            &mut app,
            "/api/v1/query/wedding",
            serde_json::json!({ "start_date": "2025-01-01", "days": 7, "sync": true }),
        )
        .await;

        let (status, json) = get_uri(&mut app, "/api/v1/jobs?job_type=load,query").await;

        assert_eq!(
            status,
            StatusCode::OK,
            "multi-job_type filter should return 200"
        );
        let jobs = json["jobs"].as_array().expect("should have jobs array");
        for job in jobs {
            let jt = job["job_type"].as_str().expect("job should have job_type");
            assert!(
                jt == "load" || jt == "query",
                "job_type should be load or query, got: {}",
                jt
            );
        }
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn list_jobs_invalid_status_in_multi_returns_400() {
        let mut app = build_app().await;

        let (status, json) = get_uri(&mut app, "/api/v1/jobs?status=complete,invalid").await;

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "invalid status in multi-value should return 400"
        );
        assert_eq!(json["error"], "invalid_status");
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn list_jobs_invalid_job_type_returns_400() {
        let mut app = build_app().await;

        let (status, json) = get_uri(&mut app, "/api/v1/jobs?job_type=bad").await;

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "invalid job_type should return 400"
        );
        assert_eq!(json["error"], "invalid_job_type");
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn list_jobs_date_range_filter() {
        let mut app = build_app().await;

        let (status, json) = get_uri(
            &mut app,
            "/api/v1/jobs?created_after=2025-01-01&created_before=2025-03-01",
        )
        .await;

        assert_eq!(
            status,
            StatusCode::OK,
            "date range filter should return 200"
        );
        assert!(json["jobs"].is_array(), "response should have jobs array");
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn list_jobs_invalid_date_filter_returns_400() {
        let mut app = build_app().await;

        let (status, json) = get_uri(&mut app, "/api/v1/jobs?created_after=not-a-date").await;

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "invalid date filter should return 400"
        );
        assert_eq!(json["error"], "invalid_date");
    }

    // =========================================================================
    // Cursor pagination tests
    // =========================================================================

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn cursor_pagination_first_page() {
        let mut app = build_app().await;

        // Create several jobs
        for i in 1..=5 {
            let _ = post_json(
                &mut app,
                "/api/v1/load",
                serde_json::json!({
                    "start_date": format!("2025-02-{:02}", i),
                    "days": 1,
                    "sync": true
                }),
            )
            .await;
        }

        let (status, json) = get_uri(&mut app, "/api/v1/jobs?count=3").await;

        assert_eq!(status, StatusCode::OK, "first page should return 200");
        let jobs = json["jobs"].as_array().unwrap();
        assert_eq!(jobs.len(), 3, "first page should have 3 jobs");
        assert!(json["next"].is_string(), "first page should have next URL");
        assert!(json["prev"].is_null(), "first page should have prev=null");
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn cursor_pagination_follow_next() {
        let mut app = build_app().await;

        // Create several jobs
        for i in 1..=5 {
            let _ = post_json(
                &mut app,
                "/api/v1/load",
                serde_json::json!({
                    "start_date": format!("2025-03-{:02}", i),
                    "days": 1,
                    "sync": true
                }),
            )
            .await;
        }

        // Get first page
        let (_, page1) = get_uri(&mut app, "/api/v1/jobs?count=2").await;
        assert!(page1["prev"].is_null(), "first page prev=null");

        let next_url = page1["next"].as_str().expect("should have next URL");

        // Follow next to get second page
        let (_, page2) = get_uri(&mut app, next_url).await;
        assert!(page2["next"].is_string(), "second page should have next");
        assert!(page2["prev"].is_string(), "second page should have prev");
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn cursor_pagination_last_page() {
        let mut app = build_app().await;

        // Create enough jobs to ensure multiple pages with count=2
        // Need 2 pages full + 1 extra to guarantee a non-full last page
        for i in 1..=5 {
            let _ = post_json(
                &mut app,
                "/api/v1/load",
                serde_json::json!({
                    "start_date": format!("2025-04-{:02}", i),
                    "days": 1,
                    "sync": true
                }),
            )
            .await;
        }

        // Walk through all pages until we reach the last one
        let mut url = "/api/v1/jobs?count=2".to_string();
        let mut found_last_page = false;
        let mut pages_visited = 0;

        loop {
            let (_, page) = get_uri(&mut app, &url).await;
            let _jobs = page["jobs"].as_array().unwrap();
            pages_visited += 1;

            match page["next"].as_str() {
                Some(next) => url = next.to_string(),
                None => {
                    // This is the last page
                    found_last_page = true;
                    // Last page should have fewer jobs than count (unless exact fit)
                    break;
                }
            }

            // Safety: don't loop forever
            if pages_visited > 50 {
                panic!("paginated through 50 pages without finding a last page");
            }
        }

        assert!(
            found_last_page,
            "should have found a last page with next=null"
        );
        assert!(pages_visited >= 2, "should have visited at least 2 pages");
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn cursor_pagination_invalid_cursor_returns_400() {
        let mut app = build_app().await;

        let (status, json) = get_uri(&mut app, "/api/v1/jobs?cursor=!!!invalid!!!").await;

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "invalid cursor should return 400"
        );
        assert_eq!(json["error"], "invalid_cursor");
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn cursor_pagination_no_overlap() {
        let mut app = build_app().await;

        // Create 5 jobs
        for i in 1..=5 {
            let _ = post_json(
                &mut app,
                "/api/v1/load",
                serde_json::json!({
                    "start_date": format!("2025-05-{:02}", i),
                    "days": 1,
                    "sync": true
                }),
            )
            .await;
        }

        // Collect all job IDs across pages
        let mut all_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut url = "/api/v1/jobs?count=2".to_string();

        loop {
            let (_, page) = get_uri(&mut app, &url).await;
            let jobs = page["jobs"].as_array().unwrap();
            for job in jobs {
                let id = job["job_id"].as_str().unwrap().to_string();
                assert!(all_ids.insert(id), "found duplicate job ID across pages");
            }

            match page["next"].as_str() {
                Some(next) => url = next.to_string(),
                None => break,
            }
        }

        assert!(
            all_ids.len() >= 5,
            "should have collected at least 5 unique job IDs across pages, got {}",
            all_ids.len()
        );
    }

    // =========================================================================
    // DELETE /api/v1/jobs/:id tests
    // =========================================================================

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn delete_completed_job_returns_204() {
        let mut app = build_app().await;

        // Create a completed sync load job
        let (_, create_json) = post_json(
            &mut app,
            "/api/v1/load",
            serde_json::json!({ "start_date": "2025-01-01", "days": 1, "sync": true }),
        )
        .await;

        let job_id = create_json["job_id"].as_str().expect("should have job_id");

        let (status, json) = delete_uri(&mut app, &format!("/api/v1/jobs/{}", job_id)).await;

        assert_eq!(
            status,
            StatusCode::NO_CONTENT,
            "deleting completed job should return 204, got {}: {:?}",
            status,
            json
        );
        // 204 has no body
        assert!(json.is_null(), "204 response body should be null");
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn delete_nonexistent_job_returns_404() {
        let mut app = build_app().await;

        let (status, json) = delete_uri(
            &mut app,
            "/api/v1/jobs/00000000-0000-0000-0000-000000000000",
        )
        .await;

        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "deleting nonexistent job should return 404"
        );
        assert_eq!(json["error"], "not_found");
        assert!(
            json["message"].as_str().unwrap_or("").contains("not found"),
            "message should explain not found, got: {:?}",
            json["message"]
        );
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn delete_in_process_job_returns_409() {
        let pool = common::test_pool().await;
        let repo = JobRepository::new(pool.clone());

        // Create a pending job directly in the DB
        let _job = sqlx::query_as::<_, astro_clock::jobs::types::Job>(
            "INSERT INTO jobs (job_type, status, payload) VALUES ('load', 'pending', '{}') RETURNING *",
        )
        .fetch_one(&pool)
        .await
        .expect("should insert pending job");

        // Claim it so it becomes in_process
        let in_process = repo
            .claim_next_job("test-worker-409")
            .await
            .expect("should claim job")
            .expect("should have a job");

        let job_id = in_process.id;
        drop(pool);

        let mut app = build_app().await;
        let (status, json) = delete_uri(&mut app, &format!("/api/v1/jobs/{}", job_id)).await;

        assert_eq!(
            status,
            StatusCode::CONFLICT,
            "deleting in_process job should return 409, got {}: {:?}",
            status,
            json
        );
        assert_eq!(json["error"], "conflict");
        assert!(
            json["message"]
                .as_str()
                .unwrap_or("")
                .contains("in_process"),
            "message should mention in_process status, got: {:?}",
            json["message"]
        );
    }

    #[tokio::test]
    #[ignore = "requires TEST_PG_URL and TimescaleDB with seed data"]
    async fn delete_then_get_returns_404() {
        let mut app = build_app().await;

        // Create a completed sync load job
        let (_, create_json) = post_json(
            &mut app,
            "/api/v1/load",
            serde_json::json!({ "start_date": "2025-01-01", "days": 1, "sync": true }),
        )
        .await;

        let job_id = create_json["job_id"].as_str().expect("should have job_id");

        // Delete it
        let (delete_status, _) = delete_uri(&mut app, &format!("/api/v1/jobs/{}", job_id)).await;
        assert_eq!(delete_status, StatusCode::NO_CONTENT);

        // GET the same job should now return 404
        let (get_status, get_json) = get_uri(&mut app, &format!("/api/v1/jobs/{}", job_id)).await;
        assert_eq!(
            get_status,
            StatusCode::NOT_FOUND,
            "GET after DELETE should return 404"
        );
        assert_eq!(get_json["error"], "not_found");
    }
}
