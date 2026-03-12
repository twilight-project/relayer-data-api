-- Add new order_type enum values for settlement triggers
ALTER TYPE order_type ADD VALUE IF NOT EXISTS 'Stoploss';
ALTER TYPE order_type ADD VALUE IF NOT EXISTS 'Takeprofit';

-- Add new order_status enum values for first-time price additions
ALTER TYPE order_status ADD VALUE IF NOT EXISTS 'LimitPriceAdded';
ALTER TYPE order_status ADD VALUE IF NOT EXISTS 'StopLossAdded';
ALTER TYPE order_status ADD VALUE IF NOT EXISTS 'TakeProfitAdded';
