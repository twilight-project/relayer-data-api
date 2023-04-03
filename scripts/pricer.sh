#!/bin/bash

RAND=$(python3 -c 'import random; print(20000*random.random())')

MESSAGE='{"jsonrpc": "2.0", "method": "SetPrice", "params": { "orderid": "34f7deea-d207-11ed-a040-f73f34c29cc2", "price": '$RAND', "key": 45}, "id": "3"}'

curl -XPOST -H 'content-type: application/json' http://localhost:3030 -d "$MESSAGE"
