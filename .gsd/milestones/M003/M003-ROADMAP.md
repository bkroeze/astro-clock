# M003: Job Maintenance & Enhanced List API

## Vision
Enhanced job management API with multi-value filters (status/job_type lists, date ranges), cursor-based pagination using stable (created_at, id) anchors with auto-generated next/prev URLs, and a single-job DELETE endpoint that guards against deleting in_process jobs.

## Slice Overview
| ID | Slice | Risk | Depends | Done | After this |
|----|-------|------|---------|------|------------|
| S01 | S01 | medium | — | ✅ | GET /api/v1/jobs?status=complete,failed&job_type=load,query&created_after=2025-01-01&created_before=2025-03-01 returns filtered results; invalid values return 400 |
| S02 | S02 | high | — | ⬜ | GET /api/v1/jobs?count=5 returns page with next/prev URLs; following next returns the next stable page; inserting jobs between page fetches doesn't shift results |
| S03 | DELETE endpoint + integration tests | low | S01, S02 | ⬜ | DELETE /api/v1/jobs/:id returns 204 for completed job, 404 for missing, 409 for in_process; full integration test suite covers filters, pagination, and delete |
