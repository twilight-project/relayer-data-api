# `candle_data` — OHLCV Candlestick Data

Returns OHLCV (Open, High, Low, Close, Volume) candlestick data for the BTC/USD market at a specified interval, starting from a given timestamp with pagination support.

**Endpoint:** Public JSON-RPC — `POST :8987`

---

## Request

```json
{
  "jsonrpc": "2.0",
  "method": "candle_data",
  "params": {
    "interval": "<string>",              // required
    "since":    "<ISO-8601 datetime>",   // required
    "limit":    <integer>,               // required
    "offset":   <integer>                // required
  },
  "id": "1"
}
```

---

## Parameters

### `interval` (required)

The candlestick time resolution. Determines how price data is bucketed.

| Value | Duration | Underlying table |
|---|---|---|
| `"ONE_MINUTE"` | 1 minute | `candles_1min` |
| `"FIVE_MINUTE"` | 5 minutes | `candles_1min` |
| `"FIFTEEN_MINUTE"` | 15 minutes | `candles_1min` |
| `"THIRTY_MINUTE"` | 30 minutes | `candles_1min` |
| `"ONE_HOUR"` | 1 hour | `candles_1hour` |
| `"FOUR_HOUR"` | 4 hours | `candles_1hour` |
| `"EIGHT_HOUR"` | 8 hours | `candles_1hour` |
| `"TWELVE_HOUR"` | 12 hours | `candles_1hour` |
| `"ONE_DAY"` | 1 day | `candles_1day` |
| `"ONE_DAY_CHANGE"` | 1 day (24h change) | `candles_1hour` |

> **Note:** Sub-hourly intervals (`ONE_MINUTE` through `THIRTY_MINUTE`) aggregate from the 1-minute candle table. Hourly intervals aggregate from the 1-hour candle table. `ONE_DAY` uses the daily candle table. `ONE_DAY_CHANGE` is a special variant that reads from the hourly table.

---

### `since` (required)

The start timestamp for the candle query. The `since` value is truncated (floored) to the nearest interval boundary.

| Field | Type | Description |
|---|---|---|
| `since` | ISO-8601 datetime (UTC) | Start of the candle range |

For example, with `interval: "ONE_HOUR"` and `since: "2026-02-20T14:37:00Z"`, the effective start is `2026-02-20T14:00:00Z`.

---

### `limit` (required)

Maximum number of candles to return.

| Field | Type | Description |
|---|---|---|
| `limit` | integer | Number of candle rows to return |

---

### `offset` (required)

Number of candle rows to skip before returning results.

| Field | Type | Description |
|---|---|---|
| `offset` | integer | Number of candles to skip (for pagination) |

> **Performance note:** The query window is bounded to `since + (offset + limit) * interval_duration`, capped at the current time. This prevents generating excessively large time series when `since` is far in the past.

---

## Response

```json
{
  "jsonrpc": "2.0",
  "result": [
    {
      "updated_at": "2026-02-24T12:00:00Z",
      "start": "2026-02-24T10:00:00Z",
      "end": "2026-02-24T11:00:00Z",
      "resolution": "'1 hour'",
      "low": "95200.50",
      "high": "96100.00",
      "open": "95800.00",
      "close": "95950.75",
      "btc_volume": "12.345",
      "trades": 482,
      "usd_volume": "1183920.50"
    },
    {
      "updated_at": "2026-02-24T12:00:00Z",
      "start": "2026-02-24T11:00:00Z",
      "end": "2026-02-24T12:00:00Z",
      "resolution": "'1 hour'",
      "low": "95700.00",
      "high": "96400.25",
      "open": "95950.75",
      "close": "96250.00",
      "btc_volume": "8.721",
      "trades": 315,
      "usd_volume": "839412.00"
    }
  ],
  "id": "1"
}
```

| Field | Type | Description |
|---|---|---|
| `updated_at` | ISO-8601 datetime (UTC) | Server time when the query was executed |
| `start` | ISO-8601 datetime (UTC) | Candle bucket start time |
| `end` | ISO-8601 datetime (UTC) | Candle bucket end time (`start + interval`) |
| `resolution` | string | SQL interval string (e.g. `"'1 hour'"`, `"'5 minutes'"`) |
| `low` | decimal string | Lowest price during the candle period |
| `high` | decimal string | Highest price during the candle period |
| `open` | decimal string | Opening price (first trade in the bucket) |
| `close` | decimal string | Closing price (last trade in the bucket) |
| `btc_volume` | decimal string | Total BTC volume traded in the bucket |
| `trades` | integer | Number of trades in the bucket |
| `usd_volume` | decimal string | Total USD volume traded in the bucket |

> Candle buckets with no matching trade data are excluded from the response.

---

## Examples

### 1. Last 60 one-minute candles

```json
{
  "jsonrpc": "2.0",
  "method": "candle_data",
  "params": {
    "interval": "ONE_MINUTE",
    "since": "2026-02-24T11:00:00Z",
    "limit": 60,
    "offset": 0
  },
  "id": "1"
}
```

Returns up to 60 one-minute candles starting from 11:00 UTC.

---

### 2. Hourly candles with pagination (page 2)

```json
{
  "jsonrpc": "2.0",
  "method": "candle_data",
  "params": {
    "interval": "ONE_HOUR",
    "since": "2026-02-20T00:00:00Z",
    "limit": 24,
    "offset": 24
  },
  "id": "1"
}
```

Skips the first 24 hourly candles and returns the next 24 (i.e. the second day of data).

---

### 3. Four-hour candles for the past week

```json
{
  "jsonrpc": "2.0",
  "method": "candle_data",
  "params": {
    "interval": "FOUR_HOUR",
    "since": "2026-02-17T00:00:00Z",
    "limit": 42,
    "offset": 0
  },
  "id": "1"
}
```

Returns up to 42 four-hour candles (~7 days of data).

---

### 4. Daily candles for the past 30 days

```json
{
  "jsonrpc": "2.0",
  "method": "candle_data",
  "params": {
    "interval": "ONE_DAY",
    "since": "2026-01-25T00:00:00Z",
    "limit": 30,
    "offset": 0
  },
  "id": "1"
}
```

Returns up to 30 daily candles.

---

### 5. Five-minute candles (small page)

```json
{
  "jsonrpc": "2.0",
  "method": "candle_data",
  "params": {
    "interval": "FIVE_MINUTE",
    "since": "2026-02-24T08:00:00Z",
    "limit": 12,
    "offset": 0
  },
  "id": "1"
}
```

Returns up to 12 five-minute candles (1 hour of data starting from 08:00 UTC).

---

### 6. 24-hour change candles

```json
{
  "jsonrpc": "2.0",
  "method": "candle_data",
  "params": {
    "interval": "ONE_DAY_CHANGE",
    "since": "2026-02-23T00:00:00Z",
    "limit": 10,
    "offset": 0
  },
  "id": "1"
}
```

Uses the `ONE_DAY_CHANGE` variant which reads from the hourly candle table.

---

## Error Responses

| Scenario | Error message |
|---|---|
| Invalid or missing `interval` value | Standard JSON-RPC invalid params error |
| Missing required params (`since`, `limit`, `offset`) | Standard JSON-RPC invalid params error |
| Invalid datetime format for `since` | Standard JSON-RPC invalid params error |
| Database unavailable | `"Database error: ..."` |
| Query execution failure | `"Error fetching candles info: ..."` |

```json
{
  "jsonrpc": "2.0",
  "error": { "code": -32602, "message": "Invalid params" },
  "id": "1"
}
```

---

## Interval Quick Reference

```
ONE_MINUTE      → 1min buckets   (from candles_1min)
FIVE_MINUTE     → 5min buckets   (from candles_1min)
FIFTEEN_MINUTE  → 15min buckets  (from candles_1min)
THIRTY_MINUTE   → 30min buckets  (from candles_1min)
ONE_HOUR        → 1hr buckets    (from candles_1hour)
FOUR_HOUR       → 4hr buckets    (from candles_1hour)
EIGHT_HOUR      → 8hr buckets    (from candles_1hour)
TWELVE_HOUR     → 12hr buckets   (from candles_1hour)
ONE_DAY         → 1day buckets   (from candles_1day)
ONE_DAY_CHANGE  → 1day buckets   (from candles_1hour)
```
