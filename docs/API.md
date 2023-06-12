# RPCs
* lend_pool_info
  Retreive current information about the lendpool.

  POST: `{ "method": "lend_pool_info", "params": null, "id": "1", "jsonrpc": "2.0"}`

  Response:
  ```json
  {
    "jsonrpc": "2.0",
    "result": {
      "aggregate_log_sequence": 123423,
      "id": 1,
      "last_snapshot_id": 8765,
      "nonce": 8,
      "pending_orders": 400000,
      "sequence": 43566,
      "total_locked_value": "2345",
      "total_pool_share": "2300"
    },
    "id": "1"
  }
  ```

* trader_order_info

  Get info for a trader order by id.

  POST: `{ "method": "trader_order_info", "params": { "id": "7EC8D23F-9BBE-4063-BD28-9331669A517F" }, "id": "1", "jsonrpc": "2.0"}`

  Response:
  ```json
  {
    "jsonrpc": "2.0",
    "result": {
      "account_id": "uasywxdifg",
      "available_margin": "7234465",
      "bankruptcy_price": "4758584",
      "bankruptcy_value": "48585345",
      "entry_nonce": 123,
      "entry_sequence": 234569,
      "entryprice": "23404",
      "execution_price": "78954",
      "exit_nonce": 456,
      "id": 1,
      "initial_margin": "9393939",
      "leverage": "94949494",
      "liquidation_price": "48485",
      "maintenance_margin": "823434",
      "order_status": "PENDING",
      "order_type": "LIMIT",
      "position_type": "LONG",
      "positionsize": "234466",
      "settlement_price": "90994554",
      "timestamp": "2023-06-12T12:37:54.917350Z",
      "unrealized_pnl": "283442",
      "uuid": "7ec8d23f-9bbe-4063-bd28-9331669a517f"
    },
    "id": "1"
  }
  ```

* lend_order_info

  Get info for a lend order by id.

  POST: `{ "method": "lend_order_info", "params": { "id": "7EC8D23F-9BBE-4063-BD28-9331669A517F" }, "id": "1", "jsonrpc": "2.0"}`

  Response:
  ```json
  {
    "jsonrpc": "2.0",
    "result": {
      "account_id": "uasywxdifg",
      "balance": "4858584",
      "deposit": "2344",
      "entry_nonce": 123,
      "entry_sequence": 344,
      "exit_nonce": 9595,
      "id": 1,
      "new_lend_state_amount": "90954",
      "npoolshare": "234",
      "nwithdraw": "9595",
      "order_status": "PENDING",
      "order_type": "LIMIT",
      "payment": "2345",
      "timestamp": "2023-06-12T12:46:11.878336Z",
      "tlv0": "123",
      "tlv1": "124",
      "tlv2": "234",
      "tlv3": "234",
      "tps0": "345",
      "tps1": "96969",
      "tps2": "9696",
      "tps3": "96969",
      "uuid": "7ec8d23f-9bbe-4063-bd28-9331669a517f"
    },
    "id": "1"
  }
  ```

* get_funding_rate

  Gets current funding rate.

  POST: `{ "method": "get_funding_rate", "params": null, "id": "1", "jsonrpc": "2.0"}`

  Response:
  ```json
  {
    "jsonrpc": "2.0",
    "result": {
      "id": 1,
      "price": "99595",
      "rate": "234",
      "timestamp": "2023-06-12T12:50:44.938092Z"
    },
    "id": "1"
  }
  ```

* btc_usd_price

  Current USD price of BTC.

  POST: `{ "method": "btc_usd_price", "params": null, "id": "1", "jsonrpc": "2.0"}`

  Response:
  ```json
  {
    "jsonrpc": "2.0",
    "result": {
      "id": 576,
      "price": "23484",
      "timestamp": "2023-06-14T12:45:44.930721Z"
    },
    "id": "1"
  }
  ```

* candle_data

  Fetch candlestick data. Possible resolutions are:
    * ONE_MINUTE
    * FIVE_MINUTE
    * FIFTEEN_MINUTE
    * THIRTY_MINUTE
    * ONE_HOUR
    * FOUR_HOUR
    * EIGHT_HOUR
    * TWELVE_HOUR
    * ONE_DAY

  POST: `{ "method": "candle_data", "params": { "interval": "ONE_MINUTE", "since": "2023-05-01T00:00:00.0Z" }, "id": "1", "jsonrpc": "2.0"}`

  Response:
  ```json
  {
    "jsonrpc": "2.0",
    "result": [
      {
        "close": "22862",
        "end": "2023-06-12T12:50:44.930721Z",
        "high": "22862",
        "low": "22862",
        "open": "22862",
        "start": "2023-06-12T12:50:44.930721Z"
      },
      {
        "close": "8056",
        "end": "2023-06-12T12:55:44.930721Z",
        "high": "8056",
        "low": "8056",
        "open": "8056",
        "start": "2023-06-12T12:55:44.930721Z"
      },
      {
        "close": "16775",
        "end": "2023-06-12T13:00:44.930721Z",
        "high": "16775",
        "low": "16775",
        "open": "16775",
        "start": "2023-06-12T13:00:44.930721Z"
      }
    ],
    "id": "1"
  }
  ```

* historical_price

  Get historical BTC price, optionally by range args.

  POST: `{ "method": "historical_price", "params": { "from": "2023-06-12T00:00:00.0Z", "to": "2023-06-12T13:00:00.0Z" }, "id": "1", "jsonrpc": "2.0"}`

  Response:
  ```json
  {
    "jsonrpc": "2.0",
    "result": [
      {
        "id": 1,
        "price": "22862",
        "timestamp": "2023-06-12T12:50:44.930721Z"
      },
      {
        "id": 2,
        "price": "8056",
        "timestamp": "2023-06-12T12:55:44.930721Z"
      }
    ],
    "id": "1"
  }
  ```

* historical_funding_rate

  Get historical funding rates, optionally by range.

  POST: `{ "method": "historical_funding_rate", "params": { "from": "2023-06-12T00:00:00.0Z", "to": "2023-06-12T13:00:00.0Z" }, "id": "1", "jsonrpc": "2.0"}`

  Response:
  ```json
  {
    "jsonrpc": "2.0",
    "result": [
      {
        "id": 1,
        "price": "99595",
        "rate": "234",
        "timestamp": "2023-06-12T12:50:44.938092Z"
      }
    ],
    "id": "1"
  }
  ```

* open_limit_orders

  Fetch open limit orders.

  POST: `{ "method": "open_limit_orders", "params": null, "id": "1", "jsonrpc": "2.0"}`

  Response:
  ```json
  {
    "jsonrpc": "2.0",
    "result": [
      {
        "account_id": "uasywxdifg",
        "available_margin": "7234465",
        "bankruptcy_price": "4758584",
        "bankruptcy_value": "48585345",
        "entry_nonce": 123,
        "entry_sequence": 234569,
        "entryprice": "23404",
        "execution_price": "78954",
        "exit_nonce": 456,
        "id": 1,
        "initial_margin": "9393939",
        "leverage": "94949494",
        "liquidation_price": "48485",
        "maintenance_margin": "823434",
        "order_status": "PENDING",
        "order_type": "LIMIT",
        "position_type": "LONG",
        "positionsize": "234466",
        "settlement_price": "90994554",
        "timestamp": "2023-06-12T12:50:44.936947Z",
        "unrealized_pnl": "283442",
        "uuid": "7ec8d23f-9bbe-4063-bd28-9331669a517f"
      }
    ],
    "id": "1"
  }
  ```

* recent_trade_orders

  Fetch recent trade orders.

  POST: `{ "method": "recent_trade_orders", "params": null, "id": "1", "jsonrpc": "2.0"}`

  Response:
  ```json
  {
    "jsonrpc": "2.0",
    "result": [
      {
        "account_id": "uasywxdifg",
        "available_margin": "7234465",
        "bankruptcy_price": "4758584",
        "bankruptcy_value": "48585345",
        "entry_nonce": 123,
        "entry_sequence": 234569,
        "entryprice": "23404",
        "execution_price": "78954",
        "exit_nonce": 456,
        "id": 1,
        "initial_margin": "9393939",
        "leverage": "94949494",
        "liquidation_price": "48485",
        "maintenance_margin": "823434",
        "order_status": "PENDING",
        "order_type": "LIMIT",
        "position_type": "LONG",
        "positionsize": "234466",
        "settlement_price": "90994554",
        "timestamp": "2023-06-12T12:50:44.936947Z",
        "unrealized_pnl": "283442",
        "uuid": "7ec8d23f-9bbe-4063-bd28-9331669a517f"
      }
    ],
    "id": "1"
  }
  ```

# Subscriptions
* subscribe_live_price_data
* subscribe_order_book

