## Public Data

### APIs:

- Live Price Data
- Historical Price Data
- Funding Rate
- Historical Funding Rate
- Open Limit Orders (Order Book)
- Recent Trade Orders (24Hours)
- 24 Hours Pool APY (last day data)
- Market Info
- Candle data (Kline data: 1min, 5min, 15min, 30min, 1hr, 4hr, 8hr, 12hr, 24hr)
- Position Size (For Long, Short and Total)
- Server Time

[Click here for schema](../sample%20codes/apischema.rs)

### Public WebSocket:

- Live Price Data update (live_price_data)
- Recent Close order update (candle*data*{resolution}) (i.e. candle_data_15min)
- Open Limit Orders update (order_book)
- Candle data update (candle_update)
- Recent closed/opened trade detail (recent_trades)
