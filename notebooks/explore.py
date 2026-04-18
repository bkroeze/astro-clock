import marimo

__generated_with = "0.23.1"
app = marimo.App(width="medium")


@app.cell(hide_code=True)
def _():
    import marimo as mo
    import requests
    import polars as pl
    import altair as alt
    import time
    from datetime import datetime, timedelta
    from dotenv import load_dotenv
    import os
    import json

    # Load environment variables
    _dotenv_loaded: bool = load_dotenv()

    # API Configuration
    BASE_URL = os.getenv("API_BASE_URL", "http://localhost:8080")

    mo.md(f"""
    ## Environment Setup
    - Env Loaded: *{_dotenv_loaded}*
    - API Base URL: `{BASE_URL}`
    """)
    return BASE_URL, alt, datetime, json, mo, pl, requests, time, timedelta


@app.cell(hide_code=True)
def _(mo):
    mo.md("""
    # 🔭 Astro Clock API Explorer

    Interactive exploration of the Astro Clock REST API endpoints.
    """)
    return


@app.cell(hide_code=True)
def _(mo):
    mo.md("""
    ## Section 1: Health Check
    Quick health check to verify the API is running.
    """)
    return


@app.cell(hide_code=True)
def _(BASE_URL, pl, requests, time):
    _health_results = []

    _db_enabled = None
    for i in range(3):
        _start = time.time()
        try:
            _response = requests.get(f"{BASE_URL}/health", timeout=5)
            _elapsed_ms = (time.time() - _start) * 1000
            _body = _response.json() if _response.status_code == 200 else {}
            if _db_enabled is None:
                _db_enabled = _body.get("db_enabled")
            _db_status = (
                "✅ Enabled"
                if _db_enabled
                else "❌ Disabled"
                if _db_enabled is not None
                else "Unknown"
            )
            _health_results.append(
                {
                    "attempt": i + 1,
                    "status_code": _response.status_code,
                    "response_time_ms": round(_elapsed_ms, 2),
                    "healthy": _response.status_code == 200,
                    "db": _db_status,
                }
            )
        except Exception as e:
            _health_results.append(
                {
                    "attempt": i + 1,
                    "status_code": None,
                    "response_time_ms": None,
                    "healthy": False,
                    "error": str(e),
                }
            )

    health_df = pl.DataFrame(_health_results)
    health_df
    return (health_df,)


@app.cell(hide_code=True)
def _(mo):
    mo.md("""
    ## Section 2: Chart Generation
    Test chart generation endpoints with various parameters and formats.
    """)
    return


@app.cell
def _(BASE_URL, datetime, mo, pl, requests, time, timedelta):
    _chart_timings = []

    _test_cases = [
        {"lat": 40.7128, "lon": -74.0060, "format": "png", "name": "NYC PNG"},
        {"lat": 51.5074, "lon": -0.1278, "format": "svg", "name": "London SVG"},
        {
            "lat": 35.6762,
            "lon": 139.6503,
            "format": "png",
            "time": (datetime.now() - timedelta(days=30)).isoformat(),
            "name": "Tokyo (30 days ago)",
        },
    ]

    for case in _test_cases:
        _start = time.time()
        try:
            _params = {"lat": case["lat"], "lon": case["lon"], "format": case["format"]}
            if "time" in case:
                _params["time"] = case["time"]

            _response = requests.get(f"{BASE_URL}/chart", params=_params, timeout=10)
            _elapsed_ms = (time.time() - _start) * 1000

            _chart_timings.append(
                {
                    "test": case["name"],
                    "format": case["format"],
                    "status": _response.status_code,
                    "response_time_ms": round(_elapsed_ms, 2),
                    "content_length": len(_response.content),
                    "success": _response.status_code == 200,
                }
            )
        except Exception as e:
            _chart_timings.append(
                {
                    "test": case["name"],
                    "format": case.get("format", "png"),
                    "status": None,
                    "response_time_ms": None,
                    "error": str(e),
                    "success": False,
                }
            )

    chart_timing_df = pl.DataFrame(_chart_timings)

    mo.md("### Chart Generation Performance")
    chart_timing_df
    return (chart_timing_df,)


@app.cell
def _(mo):
    mo.md("""
    ### Chart Data (JSON Endpoint)
    Retrieve structured astrological chart data.
    """)
    return


@app.cell
def _(BASE_URL, pl, requests, time):
    _chart_data_results = []

    _locations = [
        {"lat": 40.7128, "lon": -74.0060, "name": "New York"},
        {"lat": 51.5074, "lon": -0.1278, "name": "London"},
        {"lat": -33.8688, "lon": 151.2093, "name": "Sydney"},
    ]

    for loc in _locations:
        _start = time.time()
        try:
            _response = requests.get(
                f"{BASE_URL}/api/v1/chart/data",
                params={"lat": loc["lat"], "lon": loc["lon"]},
                timeout=10,
            )
            _elapsed_ms = (time.time() - _start) * 1000
            _data = _response.json() if _response.status_code == 200 else None

            _chart_data_results.append(
                {
                    "location": loc["name"],
                    "status": _response.status_code,
                    "response_time_ms": round(_elapsed_ms, 2),
                    "num_planets": len(_data.get("planets", [])) if _data else 0,
                    "num_aspects": len(_data.get("aspects", [])) if _data else 0,
                    "house_system": _data.get("metadata", {}).get("house_system", "N/A")
                    if _data
                    else "N/A",
                    "moon_void": _data.get("moon_void_of_course") if _data else None,
                }
            )
        except Exception as e:
            _chart_data_results.append(
                {
                    "location": loc["name"],
                    "status": None,
                    "error": str(e),
                }
            )

    chart_data_df = pl.DataFrame(_chart_data_results)
    chart_data_df
    return (chart_data_df,)


@app.cell(hide_code=True)
def _(mo):
    mo.md("""
    ## Section 3: Job Management
    Explore job creation, status checking, and listing.
    """)
    return


@app.cell
def _(datetime, mo, timedelta):
    start_date_ui = mo.ui.date(
        value=datetime.now().date() - timedelta(days=7), label="Start Date"
    )
    days_slider_ui = mo.ui.slider(1, 365, value=7, label="Days to Load", debounce=True)
    sync_toggle_ui = mo.ui.checkbox(value=True, label="Synchronous Execution")

    mo.hstack([start_date_ui, days_slider_ui, sync_toggle_ui])
    return days_slider_ui, start_date_ui, sync_toggle_ui


@app.cell
def _(
    BASE_URL,
    days_slider_ui,
    mo,
    pl,
    requests,
    start_date_ui,
    sync_toggle_ui,
    time,
):
    _load_results = []
    created_jobs = []

    _payload = {
        "start_date": start_date_ui.value.strftime("%Y-%m-%d"),
        "days": days_slider_ui.value,
        "sync": sync_toggle_ui.value,
    }

    _start = time.time()
    try:
        _response = requests.post(f"{BASE_URL}/api/v1/load", json=_payload, timeout=30)
        _elapsed_ms = (time.time() - _start) * 1000
        _data = _response.json() if _response.status_code in [200, 202] else {}

        _load_results.append(
            {
                "sync_mode": "Synchronous" if sync_toggle_ui.value else "Asynchronous",
                "status_code": _response.status_code,
                "response_time_ms": round(_elapsed_ms, 2),
                "job_id": _data.get("job_id"),
                "job_status": _data.get("status"),
            }
        )

        if _data.get("job_id"):
            created_jobs.append(_data.get("job_id"))
    except Exception as e:
        _load_results.append(
            {
                "sync_mode": "Synchronous" if sync_toggle_ui.value else "Asynchronous",
                "status_code": None,
                "error": str(e),
            }
        )

    load_df = pl.DataFrame(_load_results)
    mo.md("### Load Job Creation Results")
    load_df
    return created_jobs, load_df


@app.cell
def _(created_jobs, mo):
    _job_list = (
        "\n".join([f"- `{job_id}`" for job_id in created_jobs])
        if created_jobs
        else "_No jobs created yet_"
    )
    mo.md(f"""
    ### Created Job IDs

    {_job_list}
    """)
    return


@app.cell
def _(BASE_URL, created_jobs, datetime, json, mo, pl, requests, time):
    # Show job results
    _job_details = []
    _out = None
    if created_jobs:
        for job_id in created_jobs:
            _start = time.time()
            try:
                _response = requests.get(f"{BASE_URL}/api/v1/jobs/{job_id}", timeout=5)
                _elapsed_ms = (time.time() - _start) * 1000
                _data = _response.json() if _response.status_code == 200 else {}

                _payload = _data.get("payload") or {}
                _result = _data.get("result")

                _job_details.append(
                    {
                        "job_id": job_id[:8] + "...",
                        "status": _data.get("status"),
                        "job_type": _data.get("job_type"),
                        "payload": json.dumps(_payload) if _payload else None,
                        "result": json.dumps(_result) if _result else None,
                        "created_at": _data.get("created_at"),
                        "completed_at": _data.get("completed_at"),
                        "response_time_ms": round(_elapsed_ms, 2),
                    }
                )
            except Exception as e:
                _job_details.append({"job_id": job_id[:8] + "...", "error": str(e)})

        _job_details_df = pl.DataFrame(_job_details)
        _now = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        _out = mo.vstack([mo.md(f"### Job Status & Results: {_now}"), _job_details_df])
    else:
        _out = mo.md("_No jobs to check. Create some jobs first._")
    _out or "ERROR"
    return


@app.cell
def _(BASE_URL, mo, pl, requests):
    _list_results = []

    for status_filter in [None, "pending", "in_process", "complete", "failed"]:
        _params = {}
        if status_filter:
            _params["status"] = status_filter

        try:
            _response = requests.get(
                f"{BASE_URL}/api/v1/jobs", params=_params, timeout=5
            )
            _data = _response.json() if _response.status_code == 200 else {}

            _jobs = _data.get("jobs", [])
            _list_results.append(
                {
                    "filter": status_filter or "all",
                    "count": len(_jobs),
                    "total": _data.get("total", 0),
                    "limit": _data.get("limit", 20),
                    "status": _response.status_code,
                }
            )
        except Exception as e:
            _list_results.append({"filter": status_filter or "all", "error": str(e)})

    list_df = pl.DataFrame(_list_results)
    mo.md("### Job Listing Results")
    list_df
    return


@app.cell
def _(mo):
    mo.md("""
    ## Section 4: Query Endpoints
    Test wedding, project, and travel query endpoints.
    """)
    return


@app.cell
def _(BASE_URL, created_jobs, datetime, mo, pl, requests, time, timedelta):
    _query_results = []

    _query_types = ["wedding", "project", "travel"]
    _start_date_str = (datetime.now() - timedelta(days=7)).strftime("%Y-%m-%d")

    for query_type in _query_types:
        _payload = {"start_date": _start_date_str, "days": 14, "sync": False}

        _start = time.time()
        try:
            _response = requests.post(
                f"{BASE_URL}/api/v1/query/{query_type}", json=_payload, timeout=10
            )
            _elapsed_ms = (time.time() - _start) * 1000
            _data = _response.json() if _response.status_code in [200, 202] else {}
            _job_id = _data.get("job_id")
            created_jobs.append(_job_id)
            _query_results.append(
                {
                    "query_type": query_type,
                    "status_code": _response.status_code,
                    "response_time_ms": round(_elapsed_ms, 2),
                    "job_id": _job_id,
                    "job_status": _data.get("status"),
                }
            )
        except Exception as e:
            _query_results.append(
                {"query_type": query_type, "status_code": None, "error": str(e)}
            )

    query_df = pl.DataFrame(_query_results)
    mo.md("### Query Endpoint Results")
    query_df
    return (query_df,)


@app.cell
def _(created_jobs, mo, query_df):
    _all_ids = list(created_jobs) if created_jobs else []
    if query_df is not None and len(query_df) > 0 and "job_id" in query_df.columns:
        _all_ids.extend(query_df["job_id"].to_list())
    query_job_ids = _all_ids
    poll_button = mo.ui.button(label="Refresh Job Status", value=0)
    mo.hstack([poll_button, mo.md(f"**{len(query_job_ids)} jobs to poll**")])
    return poll_button, query_job_ids


@app.cell
def _(BASE_URL, json, mo, pl, poll_button, query_job_ids, requests):
    poll_button
    _out = None
    _polled = []
    if query_job_ids:
        for _job_id in query_job_ids:
            try:
                _resp = requests.get(f"{BASE_URL}/api/v1/jobs/{_job_id}", timeout=5)
                _data = _resp.json() if _resp.status_code == 200 else {}
                _result = _data.get("result")
                _error = _data.get("error")
                _polled.append(
                    {
                        "job_id": str(_job_id)[:8] + "...",
                        "job_type": _data.get("job_type"),
                        "status": _data.get("status"),
                        "payload": json.dumps(_data.get("payload"))
                        if _data.get("payload")
                        else None,
                        "result": json.dumps(_result)[:200] if _result else None,
                        "error": json.dumps(_error) if _error else None,
                        "completed_at": _data.get("completed_at"),
                    }
                )
            except Exception as e:
                _polled.append({"job_id": str(_job_id)[:8] + "...", "error": str(e)})
        _out = mo.vstack([mo.md("### Polled Job Results"), pl.DataFrame(_polled)])
    else:
        _out = mo.md("_No jobs to poll._")
    _out or "ERROR"
    return


@app.cell
def _(mo):
    mo.md("""
    ## Section 5: Performance Visualization
    Visualize API response times across endpoints.
    """)
    return


@app.cell
def _(
    alt,
    chart_data_df,
    chart_timing_df,
    health_df,
    load_df,
    mo,
    pl,
    query_df,
):
    _performance_data = []

    if health_df is not None and len(health_df) > 0:
        for row in health_df.to_dicts():
            if row.get("response_time_ms"):
                _performance_data.append(
                    {
                        "endpoint": "Health",
                        "operation": f"Check #{row['attempt']}",
                        "response_time_ms": row["response_time_ms"],
                        "status": "Success" if row.get("healthy") else "Failed",
                    }
                )

    if chart_timing_df is not None and len(chart_timing_df) > 0:
        for row in chart_timing_df.to_dicts():
            if row.get("response_time_ms"):
                _performance_data.append(
                    {
                        "endpoint": f"Chart ({row.get('format', 'unknown')})",
                        "operation": row.get("test", "chart"),
                        "response_time_ms": row["response_time_ms"],
                        "status": "Success" if row.get("success") else "Failed",
                    }
                )

    if chart_data_df is not None and len(chart_data_df) > 0:
        for row in chart_data_df.to_dicts():
            if row.get("response_time_ms"):
                _performance_data.append(
                    {
                        "endpoint": "Chart Data",
                        "operation": row.get("location", "unknown"),
                        "response_time_ms": row["response_time_ms"],
                        "status": "Success" if row.get("status") == 200 else "Failed",
                    }
                )

    if load_df is not None and len(load_df) > 0:
        for row in load_df.to_dicts():
            if row.get("response_time_ms"):
                _performance_data.append(
                    {
                        "endpoint": "Load Job",
                        "operation": row.get("sync_mode", "unknown"),
                        "response_time_ms": row["response_time_ms"],
                        "status": "Success"
                        if row.get("status_code") in [200, 202]
                        else "Failed",
                    }
                )

    if query_df is not None and len(query_df) > 0:
        for row in query_df.to_dicts():
            if row.get("response_time_ms"):
                _performance_data.append(
                    {
                        "endpoint": "Query",
                        "operation": row.get("query_type", "unknown"),
                        "response_time_ms": row["response_time_ms"],
                        "status": "Success"
                        if row.get("status_code") in [200, 202]
                        else "Failed",
                    }
                )

    if _performance_data:
        perf_df = pl.DataFrame(_performance_data)

        chart = (
            alt.Chart(perf_df.to_pandas())
            .mark_bar()
            .encode(
                x=alt.X("endpoint:N", title="Endpoint"),
                y=alt.Y("response_time_ms:Q", title="Response Time (ms)"),
                color=alt.Color(
                    "status:N",
                    scale=alt.Scale(
                        domain=["Success", "Failed"], range=["#4CAF50", "#F44336"]
                    ),
                ),
                tooltip=["operation", "response_time_ms", "status"],
            )
            .properties(title="API Response Times by Endpoint", width=600, height=300)
        )

        mo.md("### Response Time Summary")
        mo.hstack([chart, perf_df])
    else:
        mo.md("_No performance data available yet. Run the API tests above._")
    return


@app.cell
def _(mo):
    mo.md("""
    ## Section 6: Summary Statistics
    Quick overview of API exploration results.
    """)
    return


@app.cell
def _(chart_data_df, chart_timing_df, health_df, load_df, mo, pl, query_df):
    _summary_stats = []

    _total_requests = 0
    _total_time = 0

    for df, name in [
        (health_df, "Health"),
        (chart_timing_df, "Chart"),
        (chart_data_df, "Chart Data"),
        (load_df, "Load"),
        (query_df, "Query"),
    ]:
        if df is not None and len(df) > 0:
            _times = df.filter(pl.col("response_time_ms").is_not_null())[
                "response_time_ms"
            ]
            if len(_times) > 0:
                _summary_stats.append(
                    {
                        "category": name,
                        "requests": len(_times),
                        "avg_time_ms": round(_times.mean(), 2),
                        "min_time_ms": round(_times.min(), 2),
                        "max_time_ms": round(_times.max(), 2),
                    }
                )
                _total_requests += len(_times)
                _total_time += _times.sum()

    if _summary_stats:
        summary_df = pl.DataFrame(_summary_stats)
        _avg_time = (
            round(_total_time / _total_requests, 2) if _total_requests > 0 else 0
        )
        mo.md(f"""
        ### Overall Statistics

        - **Total Requests:** {_total_requests}
        - **Total Time:** {round(_total_time, 2)} ms
        - **Average Response Time:** {_avg_time} ms

        ### By Category
        """)
        summary_df
    else:
        mo.md("_Run the API tests above to see summary statistics._")
    return


@app.cell
def _(mo):
    mo.md("""
    ---

    ## Raw Response Explorer

    View raw API responses for debugging.
    """)
    return


@app.cell
def _(mo):
    debug_endpoint = mo.ui.dropdown(
        options=[
            ("/health", "Health Check"),
            ("/api/v1/jobs", "List Jobs"),
            ("/api/v1/chart/data?lat=40.7128&lon=-74.0060", "Chart Data (NYC)"),
        ],
        value="/health",
        label="Select Endpoint",
    )
    debug_endpoint
    return (debug_endpoint,)


@app.cell
def _(BASE_URL, debug_endpoint, json, mo, requests):
    if debug_endpoint.value:
        try:
            _url = f"{BASE_URL}{debug_endpoint.value}"
            _response = requests.get(_url, timeout=5)

            _headers_str = json.dumps(dict(_response.headers), indent=2)
            if _response.headers.get("content-type", "").startswith("application/json"):
                _body_str = json.dumps(_response.json(), indent=2)
            else:
                _body_str = _response.text[:500]

            _raw_response = f"""
    **URL:** `{_url}`

    **Status:** {_response.status_code}

    **Headers:**
    ```json
    {_headers_str}
    ```

    **Body:**
    ```json
    {_body_str}
    ```
    """
            mo.md(_raw_response)
        except Exception as e:
            mo.md(f"**Error:** `{e}`")
    return


if __name__ == "__main__":
    app.run()
