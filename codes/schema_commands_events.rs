#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Event {
    TraderOrder(TraderOrder, RpcCommand, usize),
    TraderOrderUpdate(TraderOrder, RelayerCommand, usize),
    TraderOrderFundingUpdate(TraderOrder, RelayerCommand),
    TraderOrderLiquidation(TraderOrder, RelayerCommand, usize),
    LendOrder(LendOrder, RpcCommand, usize),
    PoolUpdate(LendPoolCommand, usize),
    FundingRateUpdate(f64, SystemTime),
    CurrentPriceUpdate(f64, SystemTime),
    SortedSetDBUpdate(SortedSetCommand),
    PositionSizeLogDBUpdate(PositionSizeLogCommand, PositionSizeLog),
    Stop(String),
}


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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RpcCommand {
    CreateTraderOrder(CreateTraderOrder, Meta),
    CreateLendOrder(CreateLendOrder, Meta),
    ExecuteTraderOrder(ExecuteTraderOrder, Meta),
    ExecuteLendOrder(ExecuteLendOrder, Meta),
    CancelTraderOrder(CancelTraderOrder, Meta),
    RelayerCommandTraderOrderSettleOnLimit(TraderOrder, Meta, f64),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PoolBatchOrder {
    pub nonce: usize,
    pub len: usize,
    pub amount: f64, //sats
    pub trader_order_data: Vec<LendPoolCommand>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum LendPoolCommand {
    AddTraderOrderSettlement(RpcCommand, TraderOrder, Payment),
    AddTraderLimitOrderSettlement(RelayerCommand, TraderOrder, Payment),
    AddFundingData(TraderOrder, Payment),
    AddTraderOrderLiquidation(RelayerCommand, TraderOrder, Payment),
    LendOrderCreateOrder(RpcCommand, LendOrder, Deposit),
    LendOrderSettleOrder(RpcCommand, LendOrder, Withdraw),
    BatchExecuteTraderOrder(RelayerCommand),
    InitiateNewPool(LendOrder, Meta),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PositionSizeLogCommand {
    AddPositionSize(PositionType, f64),
    RemovePositionSize(PositionType, f64),
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SortedSetCommand {
    AddLiquidationPrice(Uuid, f64, PositionType),
    AddOpenLimitPrice(Uuid, f64, PositionType),
    AddCloseLimitPrice(Uuid, f64, PositionType),
    RemoveLiquidationPrice(Uuid, PositionType),
    RemoveOpenLimitPrice(Uuid, PositionType),
    RemoveCloseLimitPrice(Uuid, PositionType),
    UpdateLiquidationPrice(Uuid, f64, PositionType),
    UpdateOpenLimitPrice(Uuid, f64, PositionType),
    UpdateCloseLimitPrice(Uuid, f64, PositionType),
    BulkSearchRemoveLiquidationPrice(f64, PositionType),
    BulkSearchRemoveOpenLimitPrice(f64, PositionType),
    BulkSearchRemoveCloseLimitPrice(f64, PositionType),
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TraderOrder {
    pub uuid: Uuid,
    pub account_id: String,
    pub position_type: PositionType,
    pub order_status: OrderStatus,
    pub order_type: OrderType,
    pub entryprice: f64,
    pub execution_price: f64,
    pub positionsize: f64,
    pub leverage: f64,
    pub initial_margin: f64,
    pub available_margin: f64,
    pub timestamp: SystemTime,
    pub bankruptcy_price: f64,
    pub bankruptcy_value: f64,
    pub maintenance_margin: f64,
    pub liquidation_price: f64,
    pub unrealized_pnl: f64,
    pub settlement_price: f64,
    pub entry_nonce: usize,
    pub exit_nonce: usize,
    pub entry_sequence: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LendOrder {
    pub uuid: Uuid,
    pub account_id: String,
    pub balance: f64,
    pub order_status: OrderStatus, //lend or settle
    pub order_type: OrderType,     // LEND
    pub entry_nonce: usize,        // change it to u256
    pub exit_nonce: usize,         // change it to u256
    pub deposit: f64,
    pub new_lend_state_amount: f64,
    pub timestamp: SystemTime,
    pub npoolshare: f64,
    pub nwithdraw: f64,
    pub payment: f64,
    pub tlv0: f64, //total locked value before lend tx
    pub tps0: f64, // total poolshare before lend tx
    pub tlv1: f64, // total locked value after lend tx
    pub tps1: f64, // total poolshre value after lend tx
    pub tlv2: f64, // total locked value before lend payment/settlement
    pub tps2: f64, // total poolshare before lend payment/settlement
    pub tlv3: f64, // total locked value after lend payment/settlement
    pub tps3: f64, // total poolshare after lend payment/settlement
    pub entry_sequence: usize,
}



#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TXType {
    ORDERTX, //TraderOrder
    LENDTX,  //LendOrder
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum OrderType {
    LIMIT,
    MARKET,
    DARK,
    LEND,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PositionType {
    LONG,
    SHORT,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum OrderStatus {
    SETTLED,
    LENDED,
    LIQUIDATE,
    CANCELLED,
    PENDING, // change it to New
    FILLED,  //executed on price ticker
}
