# `account_summary_by_twilight_address` — Single-Address Position Summary

Returns aggregate position statistics (settled, filled, liquidated) for a single Twilight address over a specified date range.

**Endpoint:** Public JSON-RPC — `POST :8987`

---

## Request

```json
{
  "jsonrpc": "2.0",
  "method": "account_summary_by_twilight_address",
  "params": {
    "t_address": "<string>",              // required
    "from":      "<ISO-8601 datetime>",   // optional (required if `since` is omitted)
    "to":        "<ISO-8601 datetime>",   // optional
    "since":     "<ISO-8601 datetime>"    // optional shorthand
  },
  "id": "1"
}
```

---

## Parameters

### `t_address` (required)

The Twilight address to query.

| Field | Type | Description |
|---|---|---|
| `t_address` | string | A valid Twilight address (e.g. `"twilight1abc..."`) |

---

### `from` / `to` (optional pair)

Explicit date range for the query window.

| Field | Type | Default | Description |
|---|---|---|---|
| `from` | ISO-8601 datetime (UTC) | — | Start of the range. Required if `since` is not provided. Must not be in the future. |
| `to` | ISO-8601 datetime (UTC) | current time | End of the range. Clamped to the current time if set in the future. |

---

### `since` (optional shorthand)

A convenience alternative to `from`/`to`. When provided, the query range becomes `[since, now]`.

| Field | Type | Default | Description |
|---|---|---|---|
| `since` | ISO-8601 datetime (UTC) | — | Start of the range. Must not be in the future. |

> **Note:** If `since` is provided, `from` and `to` are ignored.

---

## Date Validation Rules

| Rule | Error message |
|---|---|
| Neither `since` nor `from` is provided | `"Either 'since' or 'from' must be provided"` |
| `to` is provided without `from` | `"'to' cannot be provided without 'from'"` |
| `since` is in the future | `"'since' must be at least 7 days older than the current time."` |
| `from` is in the future | `"'from' must be at least 7 days older than the current time."` |
| `to` < `from` (after clamping) | `"'to' must be greater than or equal to 'from'."` |

---

## Response

```json
{
  "jsonrpc": "2.0",
  "result": {
    "from": "2026-02-01T00:00:00Z",
    "to": "2026-02-23T12:00:00Z",
    "settled_positionsize": "1500.00",
    "filled_positionsize": "3200.50",
    "liquidated_positionsize": "75.25",
    "settled_count": 12,
    "filled_count": 28,
    "liquidated_count": 2
  },
  "id": "1"
}
```

| Field | Type | Description |
|---|---|---|
| `from` | ISO-8601 datetime (UTC) | Effective start of the query range |
| `to` | ISO-8601 datetime (UTC) | Effective end of the query range |
| `settled_positionsize` | decimal string | Total position size of settled orders |
| `filled_positionsize` | decimal string | Total position size of filled orders |
| `liquidated_positionsize` | decimal string | Total position size of liquidated orders |
| `settled_count` | integer | Number of settled orders |
| `filled_count` | integer | Number of filled orders |
| `liquidated_count` | integer | Number of liquidated orders |

---

## Examples

### 1. Using `since` (everything from a date to now)

```json
{
  "jsonrpc": "2.0",
  "method": "account_summary_by_twilight_address",
  "params": {
    "t_address": "twilight1abc123def456",
    "since": "2026-02-01T00:00:00Z"
  },
  "id": "1"
}
```

Returns the summary for `twilight1abc123def456` from Feb 1 to now.

---

### 2. Using `from` + `to` (explicit window)

```json
{
  "jsonrpc": "2.0",
  "method": "account_summary_by_twilight_address",
  "params": {
    "t_address": "twilight1abc123def456",
    "from": "2026-02-01T00:00:00Z",
    "to": "2026-02-15T00:00:00Z"
  },
  "id": "1"
}
```

Returns the summary for the first half of February only.

---

### 3. Using `from` only (from a date to now)

```json
{
  "jsonrpc": "2.0",
  "method": "account_summary_by_twilight_address",
  "params": {
    "t_address": "twilight1abc123def456",
    "from": "2026-02-10T00:00:00Z"
  },
  "id": "1"
}
```

When `to` is omitted, it defaults to the current time.

---

## Error Responses

| Scenario | Error message |
|---|---|
| Missing both `since` and `from` | `"Either 'since' or 'from' must be provided"` |
| `to` provided without `from` | `"'to' cannot be provided without 'from'"` |
| `since` is in the future | `"'since' must be at least 7 days older than the current time."` |
| `from` is in the future | `"'from' must be at least 7 days older than the current time."` |
| `to` < `from` | `"'to' must be greater than or equal to 'from'."` |
| Invalid params (wrong types, etc.) | Standard JSON-RPC invalid params error |
| Database unavailable | `"Database error: ..."` |

```json
{
  "jsonrpc": "2.0",
  "error": { "code": -32000, "message": "Either 'since' or 'from' must be provided" },
  "id": "1"
}
```
