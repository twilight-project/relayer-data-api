// this function is consuming all the kafka event data and processing
// refer this for reading kafka events and then dump the data in the porstgreSQL DB



pub fn create_snapshot_data(fetchoffset: FetchOffset) -> SnapshotDB {
    let snapshot_db = SNAPSHOT_DATA.lock().unwrap().clone();
    let mut orderdb_traderorder: OrderDBSnapShotTO = snapshot_db.orderdb_traderorder;
    let mut orderdb_lendorder: OrderDBSnapShotLO = snapshot_db.orderdb_lendorder;
    let mut lendpool_database: LendPool = snapshot_db.lendpool_database;
    let mut liquidation_long_sortedset_db = snapshot_db.liquidation_long_sortedset_db;
    let mut liquidation_short_sortedset_db = snapshot_db.liquidation_short_sortedset_db;
    let mut open_long_sortedset_db = snapshot_db.open_long_sortedset_db;
    let mut open_short_sortedset_db = snapshot_db.open_short_sortedset_db;
    let mut close_long_sortedset_db = snapshot_db.close_long_sortedset_db;
    let mut close_short_sortedset_db = snapshot_db.close_short_sortedset_db;
    let mut position_size_log = snapshot_db.position_size_log;
    let mut localdb_hashmap = snapshot_db.localdb_hashmap;
    let mut event_offset: i64 = 0;
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();
    let event_timestamp = time.clone();
    let event_stoper_string = format!("snapsot-start-{}", time);
    let eventstop: Event = Event::Stop(event_stoper_string.clone());
    Event::send_event_to_kafka_queue(
        eventstop.clone(),
        CORE_EVENT_LOG.clone().to_string(),
        String::from("StopLoadMSG"),
    );
    let mut stop_signal: bool = true;

    let recever = Event::receive_event_for_snapshot_from_kafka_queue(
        CORE_EVENT_LOG.clone().to_string(),
        format!("./snapshot/snapshot-version-{}", *SNAPSHOT_VERSION),
        fetchoffset,
    )
    .unwrap();
    let recever1 = recever.lock().unwrap();
    while stop_signal {
        let data = recever1.recv().unwrap();
        // match data.value {
        //     Event::CurrentPriceUpdate(..) => {}
        //     _ => {
        //         println!("Envent log: {:#?}", data);
        //     }
        // }
        match data.value.clone() {
            Event::TraderOrder(order, cmd, seq) => match cmd {
                RpcCommand::CreateTraderOrder(_rpc_request, _metadata) => {
                    // let order_clone = order.clone();
                    orderdb_traderorder
                        .ordertable
                        .insert(order.uuid, order.clone());
                    // orderdb_traderorder.event.push(data.value);
                    if orderdb_traderorder.sequence < order.entry_sequence.clone() {
                        orderdb_traderorder.sequence = order.entry_sequence.clone();
                    }
                    if orderdb_traderorder.aggrigate_log_sequence < seq {
                        orderdb_traderorder.aggrigate_log_sequence = seq;
                    }

                    match order.order_status {
                        OrderStatus::FILLED => match order.position_type {
                            PositionType::LONG => {
                                let _ = liquidation_long_sortedset_db
                                    .add(order.uuid, (order.liquidation_price * 10000.0) as i64);
                            }
                            PositionType::SHORT => {
                                let _ = liquidation_short_sortedset_db
                                    .add(order.uuid, (order.liquidation_price * 10000.0) as i64);
                            }
                        },
                        OrderStatus::PENDING => match order.position_type {
                            PositionType::LONG => {
                                let _ = open_long_sortedset_db
                                    .add(order.uuid, (order.entryprice * 10000.0) as i64);
                            }
                            PositionType::SHORT => {
                                let _ = open_short_sortedset_db
                                    .add(order.uuid, (order.entryprice * 10000.0) as i64);
                            }
                        },
                        _ => {}
                    }
                }
                RpcCommand::CancelTraderOrder(_rpc_request, _metadata) => {
                    let order_clone = order.clone();
                    if orderdb_traderorder.ordertable.contains_key(&order.uuid) {
                        orderdb_traderorder.ordertable.remove(&order.uuid);
                    }
                    // orderdb_traderorder.event.push(data.value);
                    if orderdb_traderorder.sequence < order_clone.entry_sequence {
                        orderdb_traderorder.sequence = order_clone.entry_sequence;
                    }
                    if orderdb_traderorder.aggrigate_log_sequence < seq {
                        orderdb_traderorder.aggrigate_log_sequence = seq;
                    }
                }
                RpcCommand::ExecuteTraderOrder(_rpc_request, _metadata) => {
                    // let order_clone = order.clone();
                    if orderdb_traderorder
                        .ordertable
                        .contains_key(&order.uuid.clone())
                    {
                        orderdb_traderorder.ordertable.remove(&order.uuid.clone());
                    }
                    // orderdb_traderorder.event.push(data.value);
                    if orderdb_traderorder.sequence < order.entry_sequence.clone() {
                        orderdb_traderorder.sequence = order.entry_sequence.clone();
                    }
                    if orderdb_traderorder.aggrigate_log_sequence < seq {
                        orderdb_traderorder.aggrigate_log_sequence = seq;
                    }
                    match order.order_status {
                        OrderStatus::SETTLED => match order.position_type {
                            PositionType::LONG => {
                                let _ = liquidation_long_sortedset_db.remove(order.uuid);
                            }
                            PositionType::SHORT => {
                                let _ = liquidation_short_sortedset_db.remove(order.uuid);
                            }
                        },
                        _ => {}
                    }
                }
                RpcCommand::RelayerCommandTraderOrderSettleOnLimit(
                    _rpc_request,
                    _metadata,
                    _payment,
                ) => {
                    let order_clone = order.clone();
                    if orderdb_traderorder.ordertable.contains_key(&order.uuid) {
                        orderdb_traderorder.ordertable.remove(&order.uuid);
                    }
                    // orderdb_traderorder.event.push(data.value);
                    if orderdb_traderorder.sequence < order_clone.entry_sequence {
                        orderdb_traderorder.sequence = order_clone.entry_sequence;
                    }
                    if orderdb_traderorder.aggrigate_log_sequence < seq {
                        orderdb_traderorder.aggrigate_log_sequence = seq;
                    }
                }
                _ => {}
            },
            Event::TraderOrderUpdate(order, _cmd, seq) => {
                // let order_clone = order.clone();
                orderdb_traderorder
                    .ordertable
                    .insert(order.uuid, order.clone());
                // orderdb_traderorder.event.push(data.value);
                if orderdb_traderorder.sequence < order.entry_sequence.clone() {
                    orderdb_traderorder.sequence = order.entry_sequence.clone();
                }
                if orderdb_traderorder.aggrigate_log_sequence < seq {
                    orderdb_traderorder.aggrigate_log_sequence = seq;
                }

                match order.order_status {
                    OrderStatus::FILLED => match order.position_type {
                        PositionType::LONG => {
                            let _ = liquidation_long_sortedset_db
                                .add(order.uuid, (order.liquidation_price * 10000.0) as i64);
                        }
                        PositionType::SHORT => {
                            let _ = liquidation_short_sortedset_db
                                .add(order.uuid, (order.liquidation_price * 10000.0) as i64);
                        }
                    },
                    _ => {}
                }
            }
            Event::TraderOrderFundingUpdate(order, _cmd) => {
                orderdb_traderorder
                    .ordertable
                    .insert(order.uuid, order.clone());

                match order.position_type {
                    PositionType::LONG => {
                        let _ = liquidation_long_sortedset_db
                            .update(order.uuid, (order.liquidation_price * 10000.0) as i64);
                    }
                    PositionType::SHORT => {
                        let _ = liquidation_short_sortedset_db
                            .update(order.uuid, (order.liquidation_price * 10000.0) as i64);
                    }
                }
            }
            Event::TraderOrderLiquidation(order, _cmd, seq) => {
                // let order_clone = order.clone();
                if orderdb_traderorder
                    .ordertable
                    .contains_key(&order.uuid.clone())
                {
                    orderdb_traderorder.ordertable.remove(&order.uuid.clone());
                }
                // orderdb_traderorder.event.push(data.value);
                if orderdb_traderorder.sequence < order.entry_sequence.clone() {
                    orderdb_traderorder.sequence = order.entry_sequence.clone();
                }
                if orderdb_traderorder.aggrigate_log_sequence < seq {
                    orderdb_traderorder.aggrigate_log_sequence = seq;
                }
            }
            Event::Stop(timex) => {
                if timex == event_stoper_string {
                    stop_signal = false;
                    event_offset = data.offset;
                }
            }
            Event::LendOrder(order, cmd, seq) => match cmd {
                RpcCommand::CreateLendOrder(..) => {
                    let order_clone = order.clone();
                    orderdb_lendorder.ordertable.insert(order.uuid, order);
                    // orderdb_lendorder.event.push(data.value);
                    if orderdb_lendorder.sequence < order_clone.entry_sequence {
                        orderdb_lendorder.sequence = order_clone.entry_sequence;
                    }
                    if orderdb_lendorder.aggrigate_log_sequence < seq {
                        orderdb_lendorder.aggrigate_log_sequence = seq;
                    }
                }
                RpcCommand::ExecuteLendOrder(..) => {
                    // orderdb_lendorder.event.push(data.value);

                    if orderdb_lendorder.ordertable.contains_key(&order.uuid) {
                        orderdb_lendorder.ordertable.remove(&order.uuid);
                    }
                    if orderdb_lendorder.aggrigate_log_sequence < seq {
                        orderdb_lendorder.aggrigate_log_sequence = seq;
                    }
                    if orderdb_lendorder.sequence < order.entry_sequence {
                        orderdb_lendorder.sequence = order.entry_sequence;
                    }
                }
                _ => {}
            },
            Event::PoolUpdate(cmd, seq) => match cmd.clone() {
                LendPoolCommand::InitiateNewPool(lend_order, _metadata) => {
                    let total_pool_share = lend_order.deposit;
                    let total_locked_value = lend_order.deposit * 10000.0;
                    if lendpool_database.sequence < lend_order.entry_sequence {
                        lendpool_database.sequence = lend_order.entry_sequence;
                    }
                    if lendpool_database.nonce < lend_order.entry_nonce {
                        lendpool_database.nonce = lend_order.entry_nonce;
                    }
                    lendpool_database.total_pool_share += total_pool_share;
                    lendpool_database.total_locked_value += total_locked_value;
                    // lendpool_database.event_log.push(data.value);
                    if lendpool_database.aggrigate_log_sequence < seq {
                        lendpool_database.aggrigate_log_sequence = seq;
                    }
                }
                LendPoolCommand::LendOrderCreateOrder(_rpc_request, lend_order, deposit) => {
                    lendpool_database.nonce += 1;
                    lendpool_database.aggrigate_log_sequence += 1;
                    lendpool_database.total_locked_value += deposit * 10000.0;
                    lendpool_database.total_pool_share += lend_order.npoolshare;
                    // lendpool_database.event_log.push(data.value);
                }
                LendPoolCommand::LendOrderSettleOrder(_rpc_request, lend_order, withdraw) => {
                    lendpool_database.nonce += 1;
                    lendpool_database.aggrigate_log_sequence += 1;
                    lendpool_database.total_locked_value -= withdraw;
                    lendpool_database.total_pool_share -= lend_order.npoolshare;
                    // lendpool_database.event_log.push(data.value);
                }
                LendPoolCommand::BatchExecuteTraderOrder(cmd) => {
                    lendpool_database.nonce += 1;
                    lendpool_database.aggrigate_log_sequence += 1;
                    match cmd {
                        RelayerCommand::FundingCycle(batch, _metadata, _fundingrate) => {
                            lendpool_database.total_locked_value -= batch.amount * 10000.0;
                        }
                        RelayerCommand::RpcCommandPoolupdate() => {
                            let batch = lendpool_database.pending_orders.clone();
                            lendpool_database.total_locked_value -= batch.amount * 10000.0;
                            lendpool_database.pending_orders = PoolBatchOrder::new();
                        }
                        _ => {}
                    }
                }
                LendPoolCommand::AddFundingData(..) => {}
                LendPoolCommand::AddTraderOrderSettlement(..) => {}
                LendPoolCommand::AddTraderLimitOrderSettlement(..) => {}
                LendPoolCommand::AddTraderOrderLiquidation(..) => {}
            },
            Event::FundingRateUpdate(funding_rate, _time) => {
                // set_localdb("FundingRate", funding_rate);
                localdb_hashmap.insert("FundingRate".to_string(), funding_rate);
            }
            Event::CurrentPriceUpdate(current_price, _time) => {
                // set_localdb("CurrentPrice", current_price);
                localdb_hashmap.insert("CurrentPrice".to_string(), current_price);
            }
            Event::SortedSetDBUpdate(cmd) => match cmd {
                SortedSetCommand::AddOpenLimitPrice(order_id, entry_price, position_type) => {
                    match position_type {
                        PositionType::LONG => {
                            let _ = open_long_sortedset_db
                                .add(order_id, (entry_price * 10000.0) as i64);
                        }
                        PositionType::SHORT => {
                            let _ = open_short_sortedset_db
                                .add(order_id, (entry_price * 10000.0) as i64);
                        }
                    }
                }
                SortedSetCommand::AddLiquidationPrice(
                    order_id,
                    liquidation_price,
                    position_type,
                ) => match position_type {
                    PositionType::LONG => {
                        let _sortedset_ = liquidation_long_sortedset_db
                            .add(order_id, (liquidation_price * 10000.0) as i64);
                    }
                    PositionType::SHORT => {
                        let _ = liquidation_short_sortedset_db
                            .add(order_id, (liquidation_price * 10000.0) as i64);
                    }
                },
                SortedSetCommand::AddCloseLimitPrice(order_id, execution_price, position_type) => {
                    match position_type {
                        PositionType::LONG => {
                            let _ = close_long_sortedset_db
                                .add(order_id, (execution_price * 10000.0) as i64);
                        }
                        PositionType::SHORT => {
                            let _ = close_short_sortedset_db
                                .add(order_id, (execution_price * 10000.0) as i64);
                        }
                    }
                }
                SortedSetCommand::RemoveOpenLimitPrice(order_id, position_type) => {
                    match position_type {
                        PositionType::LONG => {
                            let _ = open_long_sortedset_db.remove(order_id);
                        }
                        PositionType::SHORT => {
                            let _ = open_short_sortedset_db.remove(order_id);
                        }
                    }
                }
                SortedSetCommand::RemoveLiquidationPrice(order_id, position_type) => {
                    match position_type {
                        PositionType::LONG => {
                            let _ = liquidation_long_sortedset_db.remove(order_id);
                        }
                        PositionType::SHORT => {
                            let _ = liquidation_short_sortedset_db.remove(order_id);
                        }
                    }
                }
                SortedSetCommand::RemoveCloseLimitPrice(order_id, position_type) => {
                    match position_type {
                        PositionType::LONG => {
                            let _ = close_long_sortedset_db.remove(order_id);
                        }
                        PositionType::SHORT => {
                            let _ = close_short_sortedset_db.remove(order_id);
                        }
                    }
                }
                SortedSetCommand::UpdateOpenLimitPrice(order_id, entry_price, position_type) => {
                    match position_type {
                        PositionType::LONG => {
                            let _ = open_long_sortedset_db
                                .update(order_id, (entry_price * 10000.0) as i64);
                        }
                        PositionType::SHORT => {
                            let _ = open_short_sortedset_db
                                .update(order_id, (entry_price * 10000.0) as i64);
                        }
                    }
                }
                SortedSetCommand::UpdateLiquidationPrice(
                    order_id,
                    liquidation_price,
                    position_type,
                ) => match position_type {
                    PositionType::LONG => {
                        let _ = liquidation_long_sortedset_db
                            .update(order_id, (liquidation_price * 10000.0) as i64);
                    }
                    PositionType::SHORT => {
                        let _ = liquidation_short_sortedset_db
                            .update(order_id, (liquidation_price * 10000.0) as i64);
                    }
                },
                SortedSetCommand::UpdateCloseLimitPrice(
                    order_id,
                    execution_price,
                    position_type,
                ) => match position_type {
                    PositionType::LONG => {
                        let _ = close_long_sortedset_db
                            .update(order_id, (execution_price * 10000.0) as i64);
                    }
                    PositionType::SHORT => {
                        let _ = close_short_sortedset_db
                            .update(order_id, (execution_price * 10000.0) as i64);
                    }
                },
                SortedSetCommand::BulkSearchRemoveOpenLimitPrice(price, position_type) => {
                    match position_type {
                        PositionType::LONG => {
                            let _ = open_long_sortedset_db.search_gt((price * 10000.0) as i64);
                        }
                        PositionType::SHORT => {
                            let _ = open_short_sortedset_db.search_lt((price * 10000.0) as i64);
                        }
                    }
                }
                SortedSetCommand::BulkSearchRemoveCloseLimitPrice(price, position_type) => {
                    match position_type {
                        PositionType::LONG => {
                            let _ = close_long_sortedset_db.search_lt((price * 10000.0) as i64);
                        }
                        PositionType::SHORT => {
                            let _ = close_short_sortedset_db.search_gt((price * 10000.0) as i64);
                        }
                    }
                }
                SortedSetCommand::BulkSearchRemoveLiquidationPrice(price, position_type) => {
                    match position_type {
                        PositionType::LONG => {
                            let _ =
                                liquidation_long_sortedset_db.search_gt((price * 10000.0) as i64);
                        }
                        PositionType::SHORT => {
                            let _ =
                                liquidation_short_sortedset_db.search_lt((price * 10000.0) as i64);
                        }
                    }
                }
            },
            Event::PositionSizeLogDBUpdate(_cmd, event) => {
                // position_size_log.total_long_positionsize = event.total_long_positionsize;
                // position_size_log.totalpositionsize = event.totalpositionsize;
                // position_size_log.total_short_positionsize = event.total_short_positionsize;
                position_size_log = event;
            }
        }
    }
    if orderdb_traderorder.sequence > 0 {
        println!("TraderOrder Database Loaded ....");
    } else {
        println!("No old TraderOrder Database found ....\nCreating new TraderOrder_database");
    }
    if orderdb_lendorder.sequence > 0 {
        println!("LendOrder Database Loaded ....");
    } else {
        println!("No old LendOrder Database found ....\nCreating new LendOrder_database");
    }
    if lendpool_database.aggrigate_log_sequence > 0 {
        println!("LendPool Database Loaded ....");
    } else {
        lendpool_database = LendPool::new();
        println!("No old LendPool Database found ....\nCreating new LendPool_database");
    }

    SnapshotDB {
        orderdb_traderorder: orderdb_traderorder.clone(),
        orderdb_lendorder: orderdb_lendorder.clone(),
        lendpool_database: lendpool_database.clone(),
        liquidation_long_sortedset_db: liquidation_long_sortedset_db.clone(),
        liquidation_short_sortedset_db: liquidation_short_sortedset_db.clone(),
        open_long_sortedset_db: open_long_sortedset_db.clone(),
        open_short_sortedset_db: open_short_sortedset_db.clone(),
        close_long_sortedset_db: close_long_sortedset_db.clone(),
        close_short_sortedset_db: close_short_sortedset_db.clone(),
        position_size_log: position_size_log.clone(),
        localdb_hashmap: localdb_hashmap,
        event_offset: event_offset,
        event_timestamp: event_timestamp,
    }
}