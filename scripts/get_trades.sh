#!/bin/bash

BEARER=$(node gen-token/gen.js)

ORDER_DATA='{"jsonrpc": "2.0", "method": "trader_order_info", "id":123, "params": {"id": "1a7fded3-7487-47ca-b88e-6aa438fda631"}  }'

curl -XPOST -H 'content-type: application/json' -H "Authorization: Bearer ${BEARER}" http://localhost:8989 -d "${ORDER_DATA}" -Dresp
