**customer_account table**

| Column Name              | Data Type   | Description                                     |
| ------------------------ | ----------- | ----------------------------------------------- |
| id                       | primary key | customer account id                             |
| customer_registration_id | varchar     | alphanumeric customer registration id           |
| username                 | varchar     | username                                        |
| password                 | varchar     | password                                        |
| account_create_datetime  | datetime    | date and time of account creation               |
| password_hint            | json        | password hint, can also be Google Auth 2.0 data |

**customer_apikey_linking table**

| Column Name         | Data Type   | Description                  |
| ------------------- | ----------- | ---------------------------- |
| id                  | primary key | id of api key linking        |
| customer_account_id | integer     | customer account id          |
| api_key             | varchar     | api key                      |
| api_salt_key        | varchar     | api salt key, if any         |
| create_datetime     | datetime    | creation date and time       |
| expiry              | datetime    | expiry date and time, if any |
| is_active           | boolean     | is active or not             |
| remark              | varchar     | optional remark              |
| authorities         | varchar     | authorities, if any          |
| limit_remaining     | integer     | remaining limit, if any      |

**customer_order_linking table**

| Column Name         | Data Type   | Description                     |
| ------------------- | ----------- | ------------------------------- |
| id                  | primary key | id of customer order linking    |
| order_id            | uuid        | order id                        |
| public_key          | varchar     | client public key               |
| customer_account_id | integer     | customer account id             |
| order_status        | varchar     | status of order, open or close  |
| create_datetime     | datetime    | date and time of order creation |

**Authorities Table**

Details of the authorities table will be discussed later.
