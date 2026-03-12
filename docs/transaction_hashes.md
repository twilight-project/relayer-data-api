# `transaction_hashes` RPC Method

## Overview

The `transaction_hashes` method returns an **event log** (audit trail) for orders associated with a given account. Each row represents a **lifecycle event** that occurred on an order — from submission, through chain execution, to settlement or error. Think of it as the "activity history" for all orders belonging to an account.

---

## Request

**Method:** `POST`
**URL:** `API_ENDPOINT/api`
**Content-Type:** `application/json`

### Query by Account ID

```json
{
  "jsonrpc": "2.0",
  "method": "transaction_hashes",
  "id": 1,
  "params": {
    "AccountId": {
      "id": "0c143e1f5e7bf18546d39fc330ee28e2b9969d92fcef7e0aa723b14a7284ba5b1994e21711c15b2ce8a171595893ef0af487da5a39cc917bb3ff1764fbbe57f179fed1ab40"
    }
  }
}
```

### Other Query Variants

You can also query by **order UUID** or **request ID**:

```json
// By Order UUID
{
  "jsonrpc": "2.0",
  "method": "transaction_hashes",
  "id": 1,
  "params": {
    "TxId": {
      "id": "668a85e3-1cef-4396-af6b-3294b24a8030"
    }
  }
}

// By Request ID (returned when you submit an order)
{
  "jsonrpc": "2.0",
  "method": "transaction_hashes",
  "id": 1,
  "params": {
    "RequestId": {
      "id": "REQIDA372F1F5AA40137BB40008C07A0D56017BEFD6720EA4F329A5EA68E477F8030C"
    }
  }
}
```

### Optional Parameters (all variants)

| Parameter | Type             | Default | Description                                      |
| --------- | ---------------- | ------- | ------------------------------------------------ |
| `id`      | `String`         | —       | **Required.** The account ID, order UUID, or request ID |
| `status`  | `OrderStatus`    | `null`  | Optional filter — return only rows matching this status |
| `limit`   | `i64`            | `500`   | Max rows to return (clamped to 1–500)            |
| `offset`  | `i64`            | `0`     | Pagination offset                                |

---

## Response

Returns a JSON array of transaction hash event objects, ordered by insertion time.

### Response Fields

| Field          | Type              | Description |
| -------------- | ----------------- | ----------- |
| `id`           | `i64`             | Auto-incrementing database row ID |
| `order_id`     | `String` (UUID)   | The order UUID this event belongs to |
| `account_id`   | `String`          | The account's public key |
| `tx_hash`      | `String`          | The on-chain transaction hash (empty string `""` if no chain tx was involved) |
| `order_type`   | `OrderType`       | The type of action — see [Order Type Values](#order-type-values) |
| `order_status` | `OrderStatus`     | The status at the time of this event — see [Order Status Values](#order-status-values) |
| `datetime`     | `String` (ISO8601)| Timestamp when the event was created (e.g. `"2026-03-11T10:46:02.380642912+00:00"`) |
| `output`       | `String` or `null`| Hex-encoded on-chain output memo (present on successful chain transactions) |
| `request_id`   | `String` or `null`| The request ID assigned when the order was submitted via RPC (prefixed with `REQID`) |
| `reason`       | `String` or `null`| Human-readable reason for updates, cancellations, or errors |
| `old_price`    | `f64` or `null`   | Previous price (for price-update/cancel events) |
| `new_price`    | `f64` or `null`   | New price (for price-update events or the execution price on limit orders) |

### Example: Real Response

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "transaction_hashes",
  "params": {
    "AccountId": {
      "id": "0c143e1f5e7bf18546d39fc330ee28e2b9969d92fcef7e0aa723b14a7284ba5b1994e21711c15b2ce8a171595893ef0af487da5a39cc917bb3ff1764fbbe57f179fed1ab40"
    }
  },
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": [
    {
      "id": 288,
      "order_id": "668a85e3-1cef-4396-af6b-3294b24a8030",
      "account_id": "0c143e1f5e7bf185...fed1ab40",
      "tx_hash": "607AAB7B5774804E88ED0CACE728CA0EAE2B191817EFDB019E44FD967DB3E553",
      "order_type": "MARKET",
      "order_status": "FILLED",
      "datetime": "2026-03-11T10:46:02.380642912+00:00",
      "output": "01000000010000002a00000000000000313832...01000000",
      "request_id": "REQIDA372F1F5AA40137BB40008C07A0D56017BEFD6720EA4F329A5EA68E477F8030C",
      "reason": null,
      "old_price": null,
      "new_price": null
    },
    {
      "id": 289,
      "order_id": "668a85e3-1cef-4396-af6b-3294b24a8030",
      "account_id": "0c143e1f5e7bf185...fed1ab40",
      "tx_hash": "",
      "order_type": "SLTP",
      "order_status": "StopLossAdded",
      "datetime": "2026-03-11T10:46:08.929504024+00:00",
      "output": null,
      "request_id": "REQID9201984714864BBE212BA9F44D9D1B9DC85962EB53EAF2D0D2BE8E9E9A891276",
      "reason": "Add Stop loss price",
      "old_price": null,
      "new_price": 62697.6
    },
    {
      "id": 290,
      "order_id": "668a85e3-1cef-4396-af6b-3294b24a8030",
      "account_id": "0c143e1f5e7bf185...fed1ab40",
      "tx_hash": "",
      "order_type": "SLTP",
      "order_status": "TakeProfitAdded",
      "datetime": "2026-03-11T10:46:08.929511194+00:00",
      "output": null,
      "request_id": "REQID9201984714864BBE212BA9F44D9D1B9DC85962EB53EAF2D0D2BE8E9E9A891276",
      "reason": "Add Take profit price",
      "old_price": null,
      "new_price": 87080.0
    },
    {
      "id": 291,
      "order_id": "668a85e3-1cef-4396-af6b-3294b24a8030",
      "account_id": "0c143e1f5e7bf185...fed1ab40",
      "tx_hash": "",
      "order_type": "SLTP",
      "order_status": "CancelledTakeProfit",
      "datetime": "2026-03-11T10:46:31.739348053+00:00",
      "output": null,
      "request_id": "REQID2B272B761E2A1656558436FB4DADAC860C70B91D92730EF1242DE54CAAEE8ACD",
      "reason": "Take profit cancelled by user",
      "old_price": 87080.0,
      "new_price": null
    },
    {
      "id": 292,
      "order_id": "668a85e3-1cef-4396-af6b-3294b24a8030",
      "account_id": "0c143e1f5e7bf185...fed1ab40",
      "tx_hash": "",
      "order_type": "SLTP",
      "order_status": "StopLossUpdated",
      "datetime": "2026-03-11T10:47:21.690832450+00:00",
      "output": null,
      "request_id": "REQID269EE42C4714E630D0A5C10E35EC40B2655FA6D178E9DAACD8596A22257E6A1C",
      "reason": "Stop loss price replaced",
      "old_price": 62697.6,
      "new_price": 62697.6
    },
    {
      "id": 293,
      "order_id": "668a85e3-1cef-4396-af6b-3294b24a8030",
      "account_id": "0c143e1f5e7bf185...fed1ab40",
      "tx_hash": "",
      "order_type": "SLTP",
      "order_status": "TakeProfitAdded",
      "datetime": "2026-03-11T10:47:21.690840201+00:00",
      "output": null,
      "request_id": "REQID269EE42C4714E630D0A5C10E35EC40B2655FA6D178E9DAACD8596A22257E6A1C",
      "reason": "Add Take profit price",
      "old_price": null,
      "new_price": 87080.0
    }
  ],
  "id": 1
}
```

### Reading the Example Above

This response shows a single market order (`668a85e3-...`) going through these events:

| # | Status | What happened |
|---|--------|---------------|
| 1 | `FILLED` (MARKET) | Order was executed on-chain. `tx_hash` has the chain hash, `output` has the output memo. |
| 2 | `StopLossAdded` (SLTP) | User set a stop-loss at **62697.6** for the first time. `old_price` is null. |
| 3 | `TakeProfitAdded` (SLTP) | User set a take-profit at **87080.0** for the first time, in the same request (same `request_id`). |
| 4 | `CancelledTakeProfit` (SLTP) | User cancelled the TP. `old_price` = 87080.0 shows what was removed. |
| 5 | `StopLossUpdated` (SLTP) | SL was replaced. `old_price` = 62697.6 (old), `new_price` = 62697.6 (new — same value, but a new SL/TP combo was submitted). |
| 6 | `TakeProfitAdded` (SLTP) | TP was re-added at **87080.0** (first TP after cancellation) in the same request as event #5. |

### Example: Limit Order — Pending then Filled

This shows a limit order that is submitted at a specific entry price, waits for the market to reach it, and then gets filled on-chain.

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": [
    {
      "id": 310,
      "order_id": "a23f9b71-4de8-4a02-b1c3-89d5e6f7a012",
      "account_id": "0c143e1f5e7bf185...fed1ab40",
      "tx_hash": "",
      "order_type": "LIMIT",
      "order_status": "PENDING",
      "datetime": "2026-03-11T12:00:05.142831000+00:00",
      "output": null,
      "request_id": "REQID5F3A8B2C1D4E6F7A8B9C0D1E2F3A4B5C6D7E8F9A0B1C2D3E4F5A6B7C8D9E0F",
      "reason": null,
      "old_price": null,
      "new_price": 78500.0
    },
    {
      "id": 345,
      "order_id": "a23f9b71-4de8-4a02-b1c3-89d5e6f7a012",
      "account_id": "0c143e1f5e7bf185...fed1ab40",
      "tx_hash": "B8C4D5E6F7A8B9C0D1E2F3A4B5C6D7E8F9A0B1C2D3E4F5A6B7C8D9E0F1A2B3",
      "order_type": "LIMIT",
      "order_status": "FILLED",
      "datetime": "2026-03-11T12:15:32.987654000+00:00",
      "output": "01000000010000002b00000000000000323033...02000000",
      "request_id": "REQID5F3A8B2C1D4E6F7A8B9C0D1E2F3A4B5C6D7E8F9A0B1C2D3E4F5A6B7C8D9E0F",
      "reason": null,
      "old_price": null,
      "new_price": null
    }
  ],
  "id": 1
}
```

**Reading the example:**

| # | Status | What happened |
|---|--------|---------------|
| 1 | `PENDING` (LIMIT) | Limit order submitted with entry price **78500.0** (`new_price`). No chain tx yet, so `tx_hash` is empty and `output` is null. |
| 2 | `FILLED` (LIMIT) | Market price reached 78500.0 — order was executed on-chain. `tx_hash` has the chain hash, `output` has the output memo. Same `request_id` links both events. |

After this, the order is an active position. Subsequent events (SL/TP adds, settlement) would follow with the same `order_id`.

---

## Order Type Values

The `order_type` field tells you the **category of the action** that generated this event.

| Value        | Description |
| ------------ | ----------- |
| `MARKET`     | The event was generated by a **market order** action (immediate execution at current price). On `SETTLED` status, means the user manually closed the position. |
| `LIMIT`      | The event was generated by a **limit order** action (pending until price reaches entry). On `SETTLED` status, means the position was closed via a limit-close price trigger. |
| `SLTP`       | The event was generated by a **stop-loss or take-profit** action (SL/TP add, update, or cancel). |
| `Stoploss`   | On `SETTLED` status — the position was **closed because the stop-loss price was hit**. |
| `Takeprofit` | On `SETTLED` status — the position was **closed because the take-profit price was hit**. |
| `LEND`       | The event was generated by a **lend order** action. |
| `DARK`       | The event was generated by a **dark pool** order action. |

---

## Order Status Values

Each row's `order_status` tells you **what happened** at that point in the order's lifecycle.

### Lifecycle Statuses (happy path)

| Status          | Meaning |
| --------------- | ------- |
| `PENDING`       | A **limit order** was submitted and is waiting for the price to reach its entry price. `new_price` contains the limit entry price. |
| `FILLED`        | The order was **successfully executed** on-chain. `tx_hash` contains the chain transaction hash, `output` contains the hex-encoded output memo. |
| `SETTLED`       | The order was **closed and settled** on-chain. `tx_hash` contains the settlement transaction hash. The `order_type` field indicates **how** the order was settled — see [Settlement order_type Breakdown](#settlement-order_type-breakdown). |
| `LIQUIDATE`     | The order was **liquidated** due to insufficient margin. |
| `LENDED`        | A lend order was successfully created. |

### Update/Modification Statuses

| Status              | order_type | Meaning |
| ------------------- | ---------- | ------- |
| `LimitPriceAdded`   | `LIMIT`    | A limit entry price was **set for the first time**. `old_price` = null, `new_price` = the entry price. |
| `LimitPriceUpdated` | `LIMIT`    | The limit entry price was **changed**. `old_price` = previous, `new_price` = updated. |
| `StopLossAdded`     | `SLTP`     | A stop-loss was **set for the first time**. `old_price` = null, `new_price` = the SL price. `reason` = "Add Stop loss price". |
| `StopLossUpdated`   | `SLTP`     | A stop-loss was **replaced** with a new price. `old_price` = previous SL, `new_price` = new SL price. `reason` = "Stop loss price replaced". |
| `TakeProfitAdded`   | `SLTP`     | A take-profit was **set for the first time**. `old_price` = null, `new_price` = the TP price. `reason` = "Add Take profit price". |
| `TakeProfitUpdated` | `SLTP`     | A take-profit was **replaced** with a new price. `old_price` = previous TP, `new_price` = new TP price. `reason` = "Take profit price replaced". |
| `FilledUpdated`     | varies     | An update was applied to a filled order (e.g., margin/leverage change). |
| `OrderUpdated`      | varies     | A general order update event. |

### Cancellation Statuses

| Status                | order_type | Meaning |
| --------------------- | ---------- | ------- |
| `CANCELLED`           | varies     | The order was **cancelled** (by user or system). `reason` explains why. |
| `CancelledStopLoss`   | `SLTP`     | A stop-loss was **removed**. `old_price` = the cancelled SL price. `reason` = "Stop loss cancelled by user". |
| `CancelledTakeProfit` | `SLTP`     | A take-profit was **removed**. `old_price` = the cancelled TP price. `reason` = "Take profit cancelled by user". |
| `CancelledLimitClose` | `LIMIT`    | A limit-close (close order at specific price) was cancelled. |

### Error/Rejection Statuses

| Status                | Meaning |
| --------------------- | ------- |
| `RejectedByRiskEngine`| The order **failed risk checks** (e.g., exceeds position limits). `reason` explains. |
| `RejectedFromChain`   | The chain **rejected** the transaction. `reason` contains the chain error. |
| `DuplicateOrder`      | A duplicate order was detected and rejected. |
| `UtxoError`           | A UTXO handling error occurred during chain submission. `reason` has details. |
| `SerializationError`  | Data serialization failed during chain submission. `reason` has details. |
| `BincodeError`        | Bincode deserialization error. |
| `HexCodeError`        | Hex decoding error. |
| `NoResponseFromChain` | The chain did not respond in time. |
| `Error`               | A general/unclassified error. |
| `OrderNotFound`       | The referenced order was not found. |
| `RequestSubmitted`    | The request was received and queued (informational). |

---

## Settlement order_type Breakdown

When an order is settled (`order_status: "SETTLED"`), the `order_type` field tells you **what triggered the settlement**:

| order_type    | order_status | Meaning |
| ------------- | ------------ | ------- |
| `MARKET`      | `SETTLED`    | The user **manually closed** a market order (standard close/settle). |
| `LIMIT`       | `SETTLED`    | The order was closed via a **limit-close** (user set a close-at-price, and the price was reached). |
| `Stoploss`    | `SETTLED`    | The order was closed because the **stop-loss price** was hit. |
| `Takeprofit`  | `SETTLED`    | The order was closed because the **take-profit price** was hit. |

This lets you determine exactly **why** a position was closed by looking at a single field rather than tracing back through the event history.

### Example: Order settled by Stop-Loss trigger

```json
{
  "id": 350,
  "order_id": "668a85e3-1cef-4396-af6b-3294b24a8030",
  "account_id": "0c143e1f5e7bf185...fed1ab40",
  "tx_hash": "A1B2C3D4E5F6...",
  "order_type": "Stoploss",
  "order_status": "SETTLED",
  "datetime": "2026-03-11T11:30:00.000000000+00:00",
  "output": "01000000...",
  "request_id": "REQID...",
  "reason": null,
  "old_price": null,
  "new_price": null
}
```

### Example: Order settled by Take-Profit trigger

```json
{
  "id": 351,
  "order_id": "772b96f4-2def-4507-bf7c-4305c35b9141",
  "account_id": "0c143e1f5e7bf185...fed1ab40",
  "tx_hash": "F6E5D4C3B2A1...",
  "order_type": "Takeprofit",
  "order_status": "SETTLED",
  "datetime": "2026-03-11T12:15:00.000000000+00:00",
  "output": "01000000...",
  "request_id": "REQID...",
  "reason": null,
  "old_price": null,
  "new_price": null
}
```

---

## Event Lifecycle Examples

Transaction hash events are emitted by **relayer-core** at key points during an order's lifecycle. Multiple rows will exist for a single order — each representing a stage.

### Market Order (happy path)

```
FILLED  (MARKET)  → Order executed on-chain (tx_hash set, output set)
SETTLED (MARKET)  → Order closed and settled on-chain (tx_hash set)
```

### Market Order with SL/TP — Manual Close

```
FILLED             (MARKET) → Order executed on-chain
StopLossAdded      (SLTP)   → User adds SL for first time (new_price = SL price)
TakeProfitAdded    (SLTP)   → User adds TP for first time (new_price = TP price)
SETTLED            (MARKET) → User manually closed the position
```

### Market Order — Closed by Stop-Loss

```
FILLED             (MARKET)    → Order executed on-chain
StopLossAdded      (SLTP)      → User adds SL at 62000
TakeProfitAdded    (SLTP)      → User adds TP at 87000
SETTLED            (Stoploss)  → Price hit 62000, stop-loss triggered settlement
```

### Market Order — Closed by Take-Profit

```
FILLED             (MARKET)      → Order executed on-chain
StopLossAdded      (SLTP)        → User adds SL at 62000
TakeProfitAdded    (SLTP)        → User adds TP at 87000
SETTLED            (Takeprofit)  → Price hit 87000, take-profit triggered settlement
```

### Limit Order — Manual Close

```
PENDING (LIMIT)   → Limit order submitted (new_price = entry price)
FILLED  (LIMIT)   → Price reached entry, executed on-chain (tx_hash set)
SETTLED (MARKET)  → User manually closed the position
```

### Limit Order — Closed by Limit-Close Price

```
PENDING (LIMIT)  → Limit order submitted (new_price = entry price)
FILLED  (LIMIT)  → Price reached entry, executed on-chain
SETTLED (LIMIT)  → Position closed via limit-close price trigger
```

### Limit Order with Price Update

```
PENDING            (LIMIT) → Submitted (new_price = 65000)
LimitPriceUpdated  (LIMIT) → Entry price changed (old_price = 65000, new_price = 64500)
FILLED             (LIMIT) → Executed on-chain
SETTLED            (MARKET) → User manually closed
```

### Rejected Order

```
RejectedByRiskEngine → reason: "Risk engine rejected order"
```

### Chain Failure

```
PENDING            → Limit order submitted
RejectedFromChain  → Chain rejected (reason has error details)
```

### Cancelled Limit Order

```
PENDING   → Limit order submitted
CANCELLED → User cancelled before fill
```

---

## Filtering by Status

To get only specific events, pass the `status` parameter:

```json
{
  "jsonrpc": "2.0",
  "method": "transaction_hashes",
  "id": 1,
  "params": {
    "AccountId": {
      "id": "0c143e1f5e7bf185...fed1ab40",
      "status": "FILLED"
    }
  }
}
```

```json
{
  "jsonrpc": "2.0",
  "method": "transaction_hashes",
  "id": 1,
  "params": {
    "AccountId": {
      "id": "0c143e1f5e7bf185...fed1ab40",
      "status": "StopLossUpdated"
    }
  }
}
```

---

## Pagination

Results default to 500 rows max. Use `limit` and `offset` for pagination:

```json
{
  "jsonrpc": "2.0",
  "method": "transaction_hashes",
  "id": 1,
  "params": {
    "AccountId": {
      "id": "0c143e1f5e7bf185...fed1ab40",
      "limit": 50,
      "offset": 100
    }
  }
}
```

---

## Key Notes for Developers

1. **Multiple rows per order** — Each order generates multiple entries representing its lifecycle stages. Use `order_id` to group events for a single order.

2. **`tx_hash` is empty for non-chain events** — Events like SL/TP updates, cancellations, rejections, and `PENDING` submissions don't involve chain transactions, so `tx_hash` will be `""`.

3. **`order_type` changes within the same order** — The initial `FILLED` event may be `MARKET`, but subsequent SL/TP events for the same `order_id` will have `order_type: "SLTP"`. Group by `order_id`, not by `order_type`.

4. **`request_id` groups related events** — SL and TP set in a single request share the same `request_id` (see events #2/#3 and #5/#6 in the example). The `request_id` is also what you get back from `submit_trade_order`. Use the `RequestId` query variant to track a specific submission.

5. **`Added` vs `Updated` statuses** — `StopLossAdded`/`TakeProfitAdded`/`LimitPriceAdded` are used when a price is set for the **first time** (old_price = null). `StopLossUpdated`/`TakeProfitUpdated`/`LimitPriceUpdated` are used when **replacing** an existing price (old_price = previous value). If a SL/TP is cancelled and then re-added, the re-add is `Added` (not `Updated`), since there's no active price to replace.

6. **`old_price` / `new_price` semantics:**
   - **Adding** SL/TP/Limit (`*Added`): `old_price` = null, `new_price` = the price set.
   - **Replacing** SL/TP/Limit (`*Updated`): `old_price` = previous price, `new_price` = new price.
   - **Cancelling** SL/TP: `old_price` = the removed price, `new_price` = null.
   - **Limit pending**: `new_price` = the entry price.
   - All other statuses: both are null.

7. **`reason` provides human-readable context** — Present on SL/TP events ("Add Stop loss price", "Stop loss price replaced", "Take profit cancelled by user"), cancellations, and all error statuses. Null on `FILLED`/`SETTLED`/`PENDING`.

8. **`output` is hex-encoded chain data** — Only present on `FILLED` and `SETTLED` events that involve successful on-chain transactions. This is the raw output memo from the chain.
