# External Integrations

**Analysis Date:** 2026-02-24

## APIs & External Services

**Astronomical Calculations:**
- Swiss Ephemeris library (`swiss-eph` crate)
  - SDK/Client: Native Rust bindings to C library
  - Purpose: Planetary position calculations, house cusp calculations, sidereal time
  - Data source: Local ephemeris files in `data/` directory
  - Files: `src/swiss_eph_impl.rs`, `src/ephemeris/mod.rs`

**HTTP Server:**
- Axum-based REST API (`src/server/mod.rs`)
  - Endpoints:
    - `GET /health` - Health check returning JSON status
    - `GET /chart` - Generate chart image (PNG) with query params: `lat`, `lon`, `time`
  - No external API clients (server only, not a client)

## Data Storage

**Databases:**
- PostgreSQL (optional, behind `db` feature flag)
  - Connection: Via `DATABASE_URL` environment variable
  - Client: `sqlx` 0.8 with runtime-tokio and postgres features
  - ORM: Compile-time checked SQL with macros
  - Schema: `src/database/schema.rs`
  - Pool management: `src/database/pool.rs`
  - Tables:
    - `charts` - Stores calculated chart data with JSONB columns for planets/houses

**File Storage:**
- Local filesystem only
  - Output formats: PNG, WebP, Markdown
  - Ephemeris data: `data/*.se1` files
  - Configuration: `config.ron`
  - Font: `Roboto-Regular.ttf`

**Caching:**
- None detected

## Authentication & Identity

**Auth Provider:**
- None - Application is stateless with no authentication
- HTTP server runs on localhost without auth

## Monitoring & Observability

**Error Tracking:**
- `tracing` crate for structured logging
- `tracing-subscriber` with env-filter for log level control
- Log levels: ERROR, WARN, INFO, DEBUG, TRACE (controlled via `RUST_LOG`)

**Logs:**
- stderr output via `tracing_subscriber::fmt`
- No external log aggregation service

## CI/CD & Deployment

**Hosting:**
- Not specified - appears to be designed for local/self-hosted deployment

**CI Pipeline:**
- None detected (no `.github/workflows/`, `.gitlab-ci.yml`, etc.)

## Environment Configuration

**Required env vars:**
- `RUST_LOG` - Controls tracing level (optional, defaults to INFO)
- `DATABASE_URL` - PostgreSQL connection string (required only with `db` feature)

**Secrets location:**
- `.env` file exists (contains environment configuration)

## Webhooks & Callbacks

**Incoming:**
- None - No webhook endpoints defined

**Outgoing:**
- None - No external webhook calls

**External Data Download:**
- Ephemeris file download via curl from GitHub (`aloistr/swisseph`)
  - Command: `just download-ephemeris`
  - Files: `sepl_18.se1`, `semo_18.se1`, `seas_18.se1`
  - URL pattern: `https://raw.githubusercontent.com/aloistr/swisseph/master/ephe/`

---

*Integration audit: 2026-02-24*
