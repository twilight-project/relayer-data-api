# `apy_chart` — APY History Chart

Returns a time series of annualised yield (APY) for the BTC lending pool, suitable for rendering a chart on the frontend.

**Endpoint:** Public JSON-RPC — `POST :8987`

---

## Request

```json
{
  "jsonrpc": "2.0",
  "method": "apy_chart",
  "params": {
    "range":   "<string>",           // required
    "step":    "<string>",           // optional
    "lookback": "<string>"           // optional
  },
  "id": "1"
}
```

---

## Parameters

### `range` (required)

The total time window of the chart — how far back from now the series starts.

| Token | Meaning |
|---|---|
| `"1d"`, `"1day"`, `"24h"`, `"24 hours"` | Last 24 hours |
| `"7d"`, `"1w"`, `"7days"`, `"7 days"` | Last 7 days |
| `"30d"`, `"30days"`, `"30 days"` | Last 30 days |

**Effect:** A longer range gives a wider x-axis on the chart. The default step size (data resolution) also changes automatically with the range — see `step` below.

---

### `step` (optional)

The spacing between data points on the time axis. If omitted, a sensible default is chosen based on `range`:

| Range | Default step |
|---|---|
| 24 hours | 1 minute |
| 7 days | 5 minutes |
| 30 days | 1 hour |

Accepted `step` tokens:

| Token | Meaning |
|---|---|
| `"1m"` | 1 minute |
| `"5m"` | 5 minutes |
| `"15m"` | 15 minutes |
| `"30m"` | 30 minutes |
| `"1h"` | 1 hour |
| `"2h"` | 2 hours |
| `"4h"` | 4 hours |
| `"12h"` | 12 hours |

**Effect:** A smaller step gives more data points (smoother curve, larger response). A larger step gives fewer points (coarser but faster). Mixing a small step with a large range (e.g. `range: "30d"`, `step: "1m"`) will produce tens of thousands of rows and should be avoided.

---

### `lookback` (optional, default `"24 hours"`)

The trailing window used to compute the APY at each grid point. For every bucket on the time axis, the APY is:

```
APY = (price_at_bucket / price_(lookback)_before_bucket) ^ (365_days / lookback) − 1
```

| Token | Meaning |
|---|---|
| `"24hours"`, `"24 hours"` | Compare to price 24 hours earlier (default) |
| `"7d"`, `"7days"`, `"7 days"` | Compare to price 7 days earlier |
| `"30d"`, `"30days"`, `"30 days"` | Compare to price 30 days earlier |

**Effect on the formula:**

| Lookback | Annualisation exponent | Interpretation |
|---|---|---|
| 24 hours | 365 | "If this 1-day return repeated every day for a year" |
| 7 days | ≈ 52.14 | "If this 7-day return repeated every week for a year" |
| 30 days | ≈ 12.17 | "If this 30-day return repeated every month for a year" |

A shorter lookback is more **reactive** (reflects recent price changes quickly but is noisier). A longer lookback is **smoother** but lags behind.

> **Note:** A bucket returns `null` APY (and is omitted from the response) when there is no price data for the lookback reference point — e.g. the first 24 hours of pool history will always have null APY with a 24-hour lookback.

---

## Response

```json
{
  "jsonrpc": "2.0",
  "result": [
    { "bucket_ts": "2026-02-19T10:01:00Z", "apy": "0.0821340000000000" },
    { "bucket_ts": "2026-02-19T10:02:00Z", "apy": "0.0823100000000000" },
    { "bucket_ts": "2026-02-19T10:03:00Z", "apy": "0.0819750000000000" }
  ],
  "id": "1"
}
```

| Field | Type | Description |
|---|---|---|
| `bucket_ts` | ISO-8601 datetime (UTC) | The time of this data point |
| `apy` | decimal string | Annualised yield as a fraction — multiply by 100 for percentage |

`apy: "0.0821"` means **8.21% APY**.

Buckets where APY cannot be computed (insufficient history) are excluded from the array entirely.

---

## Examples

### 1. Default 24-hour chart (1-minute resolution)

```json
{
  "jsonrpc": "2.0",
  "method": "apy_chart",
  "params": { "range": "1d" },
  "id": "1"
}
```

Returns ~1440 points (one per minute), each comparing current price to the price 24 hours before that bucket.

---

### 2. 7-day chart with default step (5-minute resolution)

```json
{
  "jsonrpc": "2.0",
  "method": "apy_chart",
  "params": { "range": "7d" },
  "id": "1"
}
```

Returns ~2016 points (one per 5 minutes over 7 days).

---

### 3. 30-day chart with hourly resolution

```json
{
  "jsonrpc": "2.0",
  "method": "apy_chart",
  "params": { "range": "30d" },
  "id": "1"
}
```

Returns ~720 points (one per hour over 30 days).

---

### 4. 7-day chart with a custom 4-hour step (coarser, faster)

```json
{
  "jsonrpc": "2.0",
  "method": "apy_chart",
  "params": { "range": "7d", "step": "4h" },
  "id": "1"
}
```

Returns ~42 points. Useful when you only need a rough trend line.

---

### 5. 24-hour chart with a 1-minute step, using 7-day lookback (smoother APY)

```json
{
  "jsonrpc": "2.0",
  "method": "apy_chart",
  "params": { "range": "1d", "step": "1m", "lookback": "7days" },
  "id": "1"
}
```

Each data point compares the price to 7 days earlier. The resulting APY curve is much smoother than a 24-hour lookback but reflects longer-term yield rather than recent movement.

---

### 6. 30-day chart with 30-day lookback (monthly yield view)

```json
{
  "jsonrpc": "2.0",
  "method": "apy_chart",
  "params": { "range": "30d", "step": "1h", "lookback": "30days" },
  "id": "1"
}
```

Each data point compares the price to 30 days earlier with an annualisation exponent of ~12.17. The first 30 days of pool history will have no data points since no reference price exists.

---

## Error Responses

| Scenario | Error message |
|---|---|
| Unknown `range` value | `"Unsupported range: <value>"` |
| Unknown `step` value | `"Unsupported step: <value>"` |
| Unknown `lookback` value | `"Unsupported lookback: <value>"` |
| Database unavailable | `"Database error: ..."` |

```json
{
  "jsonrpc": "2.0",
  "error": { "code": -32000, "message": "Unsupported range: 3d" },
  "id": "1"
}
```

---

## Quick Reference

```
range   → how wide the chart x-axis is
step    → how many data points are on that axis
lookback → what period each APY value is calculated over
```

Choosing `range = lookback` (e.g. both `"7d"`) gives the most natural reading: "the annualised return if you had held for 7 days ending at each point."
