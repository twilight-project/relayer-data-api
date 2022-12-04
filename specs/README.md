# Relayer API Endpoints

Twilight Pool requires an off-chain [Relayer](https://github.com/twilight-project/twilight-relayer), to open/settle trade orders and prove exchange computation in zero knowledge. It uses Kafka to maintain message queues and event logging and QuestDB to save historic data. It has an API module for message communication between relayer and the client.

![relayer](./img/img1.png)

### Price feeder server:

Price feeder connects to multiple exchanges over websocket to serve as a price oracle to the relayer. Price average calculation is TBD.

![PF](./img/img2.png)

### Kafka Msg Queue:

All trading orders are held in the message queue before they are picked up by the Relayer core for execution. Relayer may host or accept orders from multiple message queue servers to scale exchange TPS. 

![PF](./img/img3.png)

### Quest DB:

[Quest DB](https://questdb.io/) is a time series fast SQL data base. It's used to save historical data and create candle charts for historial trends. 

### Kafka Event Receiver:

For every state change, relayer creates a new event and pushes it into kafka Queue. These events trigger other processes. The types of events are:

1. TraderOrder
2. TraderOrderUpdate
3. TraderOrderFundingUpdate
4. TraderOrderLiquidation
5. LendOrder
6. PoolUpdate
7. FundingRateUpdate
8. CurrentPriceUpdate
9. SortedSetDBUpdate
10. PositionSizeLogDBUpdate
11. Stop

## Relayer API module:

The API module can be divided into two parts:

1. Client RPC server
2. Web socket Server

Authorisation process for the API is TBD.

API module works in a 2 step process:

1. Read events data from kafka and save it in DB (Data Dumping)
2. Api endpoint and web sockets for Kafka queue and DB. (API server)

DB and API Server are two separate executables to support horizontal scaling.

## Data Dumping:

This process subscribes to Kafka event logs and save all the information in postgres DB.

If we recall the purpose of Quest DB (explained above). Relayer saves historic data in quest DB and later uses this data to create candle charts. Right now, relayer-core saves the information in the Quest DB. It will be beneficial to take this responsibility from relayer-core and give it to the data dumping process, this will be the secondary purpose of this process.

## Connection to Kafka:

Kafka connection requires An IP, Port and topic name.
By default, kafka runs on port 9092
The Topic name for event logs in Kafka is ‘CoreEventLogTopic’

### Event types and explanation:

As the primary job of this process is to pick up event logs from Kafka and save them in the DB. It is better if we explain each event type.

1. TraderOrder: This event gets generated when a client sends a request msg to the relayer for an open/settle/cancel perpetual order.
2. TraderOrderUpdate: This event gets generated when relayer updates any pending order e.g. when order is opened or settled when the price is matched.
3. TraderOrderFundingUpdate: This event gets generated at every funding cycle which update traderorder data (i.e. available margin, bankruptcy price etc.)
4. TraderOrderLiquidation: This event gets generated when relayer liquidates any trade order at liquidation price match.
5. LendOrder: This event gets generated when a client(lender) sends a request msg to the relayer for an open/withdrawal request for lending BTC
6. PoolUpdate: This event gets generated when any trade order gets settled/liquidated or at fudning cycle or for when new lend order is opened/withdrawn.
7. FundingRateUpdate: Funding rate update at every funding cycle
8. CurrentPriceUpdate: System update price at every 250ms and generate a event log
9. SortedSetDBUpdate: Not Used anymore
10. PositionSizeLogDBUpdate: Position size log updates at every trade order update i.e. open/settle/liquidate.

## Postgres DB:

For now, it’s been decided that PostgresDB is the best DB for our use case. Right now, Kafka does not have pub/sub functionality. So, the data dumping process will have to poll the kafka service repeatedly to retrieve the logged events.

Some sample code to poll the Kafka service used in our testing scripts is shared below.

- [kafka consumer](../sample%20codes/kafkaconsumer.rs)
- [data dumping](../codes/snapshot_sample.rs)

Once retrieved these events will be saved in the DB. The schema for the DB is to be decided by the developer, but a few sample scripts we ran to test the system used the below shared schema.

![Schema](./img/img4.png)

Please keep in mind that this is a sample and the rest of the schema need to be created. Schema can be created based upon data structure for each event type shared above

### Understanding the Event data structures:

It is important to understand all event type’s Data Structures. This will ensure that the DB mappings are made accordingly, and no information will be lost.

1. TraderOrder(TraderOrder, RpcCommand, usize)
2. TraderOrderUpdate(TraderOrder, RelayerCommand, usize
3. TraderOrderFundingUpdate(TraderOrder, RelayerCommand)
4. TraderOrderLiquidation(TraderOrder, RelayerCommand, usize)
5. LendOrder(LendOrder, RpcCommand, usize)
6. BTC PoolUpdate(LendPoolCommand, usize)
7. FundingRateUpdate(f64, SystemTime)
8. CurrentPriceUpdate(f64, SystemTime
9. SortedSetDBUpdate(SortedSetCommand)
10. PositionSizeLogDBUpdate(PositionSizeLogCommand, PositionSizeLog)

As mentioned above, each event type consists of TradeOrder/LendOrder, RelayerCommand/RpcCommand and size objects.

```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RelayerCommand {
    FundingCycle(PoolBatchOrder, Meta, f64),
    FundingOrderEventUpdate(TraderOrder, Meta),
    PriceTickerLiquidation(Vec<Uuid>, Meta, f64),
    PriceTickerOrderFill(Vec<Uuid>, Meta, f64), //no update for lend pool
    PriceTickerOrderSettle(Vec<Uuid>, Meta, f64),
    FundingCycleLiquidation(Vec<Uuid>, Meta, f64),
    RpcCommandPoolupdate(),
}
```

```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RpcCommand {
    CreateTraderOrder(CreateTraderOrder, Meta),
    CreateLendOrder(CreateLendOrder, Meta),
    ExecuteTraderOrder(ExecuteTraderOrder, Meta),
    ExecuteLendOrder(ExecuteLendOrder, Meta),
    CancelTraderOrder(CancelTraderOrder, Meta),
    RelayerCommandTraderOrderSettleOnLimit(TraderOrder, Meta, f64),
}
```

Above code shows details for RpcCommand and RelayerCommand. Rest of the object structures are available at the below link.

- [Structures link](../sample%20codes/schema_commands_events.rs)

## Communication with Quest DB and:

The secondary task of the data dumping process is to save historic data into quest DB, As mentioned before, right now Relayer-core has this responsibility.

Please use the below connection strings to connect to the quest DB

1. QUESTDB_URL = postgresql://quest:my_password@localhost:8812/qdb
2. QUESTDB_INFLUX_URL = 127.0.0.1:9009

The QUESTDB_URL is to read from quest DB, whereas the QUESTDB_INFLUX_URL is used to write to the DB.

The code which communicates with the QuestDB in relayer-core is shared under. It could be re-used in the data dumping process.

[Sample Code](../sample%20codes/questdb.rs)
