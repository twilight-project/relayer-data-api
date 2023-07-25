# Relayer Private API

## Registration API

    - onboarding api (public)
    - retrive api (request should be verified by signature) (public)

## Trader Order

    - get unrealized PnL by order id/public key
    - get unrealized PnL all
    - get all open orders
    - get order history by id(order id)
    - get order history pagination (client id)
    - submit new order (trader order)
    - cancel order by order id (only in case of unfilled/pending limit order)
    - submit order settle request
    - submit bulk new order
    - submit bulk settle request
    - customer trade volume
    - get funding payment

## Lendorder

    - order detail
    - submit new order (lend order)
    - client poolshare value
    - apy

## Public api

    - fee (lets set in env.)
    - Fees History
    - schema fee (taker, maker, volume)

## Admin API

    - fee
    - pshy value
    - limit of volume order
    - exchange order suspemsion/halt (bool)

# Registration API

### onboarding api (public)

Client need to send a signed msg from wallet to register their public key.
Signature should be verified and in reponse client will get the passphrase and api key token (for HMAC refer: [Link1](https://www.okta.com/identity-101/hmac/#:~:text=Hash%2Dbased%20message%20authentication%20code,use%20signatures%20and%20asymmetric%20cryptography.), [Link2](https://www.binance.com/en-BH/support/faq/how-to-create-api-360002502072), [Link3](https://www.binance.com/en/support/faq/how-to-create-api-keys-on-binance-360002502072) [Link4](https://docs.rs/hmac/latest/hmac/) )

<!-- ![onboarding](../img/img5.png) -->
<p align="center">
<img src="../img/img5.png" alt="drawing" style="width:60%; " />
</p>

As image above api should create unique customer id and link the public key to that customer id.
now whenever customer place order using the same api key, database should maintain customerid <-> public key <-> orderid linking.
customer can use different account (different public key) to place order. api key will be a _unified key_ which will track/link the different public key to customer id.

### Retrieve Api Key (regenerate API key)

In case of client lose the api key/ passphrase.
client can regenerate the apikey and passphrase from any of their public key signature (we need to look for the order id from that public key and then search the linked customer id). In this case , old api key will be inactive and new key will be responded to client.

# Trader Order

### get unrealized PnL by order id/public key

Input: PnL can be retrive from both orderID or Public Key

Response: Pnl (float), datetime, current btc price, orderid

### get unrealized PnL all

In PnL all,api need to return PnL of all open orders linked to _unified key_

Response:Array of [ Pnl (float), datetime, current btc price, orderid ]

### get all open trader orders

In all open orders api, api need to return all open orders details linked to _unified key_

Response:Array of [TraderOrder]

### get order history by id(order id)

Input: orderID (Uuid)

Response: [TraderOrder]

### get order history pagination

Client can get order history of all open/close trader orders with time range (from and to datetime) with pagination amx limit of 10(may update in future) order per page. Providing _unified key_

Response:Array of [TraderOrder]

## Note:

For the below list of API, a different application will run which will be direclty connected to relayer for sending msg/new order details to relayer. order will be first verified with _unified key_ insted of client signature to place bulk orders etc. in further steps ZKos also verify the orders and their signature. All api request must be send to kafka topic (need to create new kafka topic)
list of API : _submit new order (trader order)_ to _submit new order (lend order)_

### submit new order (trader order)

Input : {
pub account_id: String,
pub position_type: PositionType,
pub order_type: OrderType,
pub leverage: f64,
pub initial_margin: f64,
pub available_margin: f64,
pub order_status: OrderStatus,
pub entryprice: f64,
pub execution_price: f64,
pub request_time:datetime,
pub order_kill_time:datetime (default :60sec)
}

Respose : Order request submitted successfully.

### submit bulk new order

Input: [ {
pub account_id: String,
pub position_type: PositionType,
pub order_type: OrderType,
pub leverage: f64,
pub initial_margin: f64,
pub available_margin: f64,
pub order_status: OrderStatus,
pub entryprice: f64,
pub execution_price: f64,
pub servertime:datetime,
pub request_time:datetime (default :60sec)
},..]

Order request submitted successfully.

### cancel order by order id (only in case of unfilled/pending limit order)

Input: Public key, OrderId

Respose: Cancellation request submitted successfully.

### submit order settle request

Input: {
pub account_id: String,
pub uuid: Uuid,
pub order_type: OrderType,
pub settle_margin: f64,
pub order_status: OrderStatus,
pub execution_price: f64,
pub servertime:datetime,
pub request_time:datetime (default :60sec)
}

Response: Execution request submitted successfully.

### submit bulk settle request

Input : [{
pub account_id: String,
pub uuid: Uuid,
pub order_type: OrderType,
pub settle_margin: f64,
pub order_status: OrderStatus,
pub execution_price: f64,
pub servertime:datetime,
pub request_time:datetime (default :60sec)
}, ...]

Response: Execution request submitted successfully.

### customer trade volume

Respose : last 30 days trade volume by linked to Customer id / _unified key_

### get funding payment

Input: OrderID, Pagination with datetime range and limit

Response : Array of [ order id, Funding payment, datetime]

# Lendorder

### submit new order (lend order)

Input: {
pub account_id: String,
pub balance: f64,
pub order_type: OrderType,
pub order_status: OrderStatus,
pub deposit: f64,
}

Response: Execution request submitted successfully.

### order detail

Input: orderID (Uuid)

Response: [LendOrder]

### client poolshare value

Input: orderID (Uuid)

Response : orderID, Poolshare f64, Datetime

### apy

need to discuss further.

# Public api

### fee (set default 0.00)

need a public api to retrive latest fee

### Fees History

input : Public Key
Respose : array of [Fee, Datetime]

### schema fee (taker, maker, volume)

fee for taker/maker will be different and will also be different depending upon monthly trade volume by client retrive from _unified key_

### pshy value

api to get latest pshy value
Input : Null
Response: Pshy value, last updated datetime

# Admin API

### fee

this api will update the relayer fees
need a api to set the fees from admin _unified key_

Schema: Fee f64, datetime, adminID(CustomerId linked with _unified key_)

Input : Type:Taker/Maker, volume in btc, Fee f64

### pshy value (default value: 8)

need a table having pshy value (float), updated datetime, adminID(CustomerId linked with _unified key_)

### limit of volume order

TO set max and min order size in btc

input: Max Leverage value, max BTC value etc..

### exchange order suspension/halt (bool)

api to halt receiving new orders for relayer
