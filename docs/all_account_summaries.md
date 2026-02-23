# `all_account_summaries` — Paginated Position Summaries for All Addresses

Returns aggregate position statistics (settled, filled, liquidated) for every Twilight address that has activity within the specified date range, with pagination support.

**Endpoint:** Public JSON-RPC — `POST :8987`

---

## Request

```json
{
  "jsonrpc": "2.0",
  "method": "all_account_summaries",
  "params": {
    "from":   "<ISO-8601 datetime>",   // optional (required if `since` is omitted)
    "to":     "<ISO-8601 datetime>",   // optional
    "since":  "<ISO-8601 datetime>",   // optional shorthand
    "limit":  <integer>,               // optional, default 50
    "offset": <integer>                // optional, default 0
  },
  "id": "1"
}
```

---

## Parameters

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

### `limit` (optional)

Maximum number of distinct addresses to return per page.

| Field | Type | Default | Constraints | Description |
|---|---|---|---|---|
| `limit` | integer | 50 | Clamped to range 1–500 | Number of address summaries per response |

---

### `offset` (optional)

Number of distinct addresses to skip before returning results.

| Field | Type | Default | Constraints | Description |
|---|---|---|---|---|
| `offset` | integer | 0 | Minimum 0 | Number of addresses to skip (for pagination) |

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

## Pagination

Pagination is over **distinct addresses**, not individual result rows. Each address appears at most once in the `summaries` array with its aggregated totals for the date range.

To page through all addresses:
- **Page 1:** `limit: 50, offset: 0` (or omit both for defaults)
- **Page 2:** `limit: 50, offset: 50`
- **Page 3:** `limit: 50, offset: 100`
- Continue until the response returns fewer items than `limit`, indicating the last page.

---

## Response

```json
{
  "jsonrpc": "2.0",
  "result": {
    "from": "2026-02-01T00:00:00Z",
    "to": "2026-02-23T12:00:00Z",
    "limit": 50,
    "offset": 0,
    "summaries": [
      {
        "twilight_address": "twilight1abc123def456",
        "settled_positionsize": "1500.00",
        "filled_positionsize": "3200.50",
        "liquidated_positionsize": "75.25",
        "settled_count": 12,
        "filled_count": 28,
        "liquidated_count": 2
      },
      {
        "twilight_address": "twilight1xyz789ghi012",
        "settled_positionsize": "800.00",
        "filled_positionsize": "1200.00",
        "liquidated_positionsize": "0.00",
        "settled_count": 5,
        "filled_count": 10,
        "liquidated_count": 0
      }
    ]
  },
  "id": "1"
}
```

### Top-level fields

| Field | Type | Description |
|---|---|---|
| `from` | ISO-8601 datetime (UTC) | Effective start of the query range |
| `to` | ISO-8601 datetime (UTC) | Effective end of the query range |
| `limit` | integer | The limit that was applied |
| `offset` | integer | The offset that was applied |
| `summaries` | array | List of per-address summary objects |

### `summaries[]` item fields

| Field | Type | Description |
|---|---|---|
| `twilight_address` | string | The Twilight address |
| `settled_positionsize` | decimal string | Total position size of settled orders |
| `filled_positionsize` | decimal string | Total position size of filled orders |
| `liquidated_positionsize` | decimal string | Total position size of liquidated orders |
| `settled_count` | integer | Number of settled orders |
| `filled_count` | integer | Number of filled orders |
| `liquidated_count` | integer | Number of liquidated orders |

---

## Examples

### 1. Basic query with `from` and `limit`

```json
{
  "jsonrpc": "2.0",
  "method": "all_account_summaries",
  "params": {
    "from": "2026-02-01T00:00:00Z",
    "limit": 20
  },
  "id": "1"
}
```

Returns the first 20 addresses with activity since Feb 1.

---

### 2. Page 2 using `offset`

```json
{
  "jsonrpc": "2.0",
  "method": "all_account_summaries",
  "params": {
    "from": "2026-02-01T00:00:00Z",
    "limit": 20,
    "offset": 20
  },
  "id": "1"
}
```

Skips the first 20 addresses and returns the next 20.

---

### 3. Using `since`

```json
{
  "jsonrpc": "2.0",
  "method": "all_account_summaries",
  "params": {
    "since": "2026-02-15T00:00:00Z"
  },
  "id": "1"
}
```

Returns up to 50 addresses (default limit) with activity from Feb 15 to now.

---

### 4. Explicit window with `from` + `to` + `limit` + `offset`

```json
{
  "jsonrpc": "2.0",
  "method": "all_account_summaries",
  "params": {
    "from": "2026-02-01T00:00:00Z",
    "to": "2026-02-15T00:00:00Z",
    "limit": 100,
    "offset": 200
  },
  "id": "1"
}
```

Returns addresses 201–300 with activity in the first half of February.

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
