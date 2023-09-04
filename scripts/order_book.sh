#!/bin/bash

BEARER=$(node gen-token/gen.js)


MINUTE_CANDLE='{"jsonrpc": "2.0", "id": "1", "method": "open_limit_orders", "params": null }'
curl -XPOST -H 'content-type: application/json' -H "Authorization: Bearer ${BEARER}" http://localhost:8989 -d "${MINUTE_CANDLE}"
