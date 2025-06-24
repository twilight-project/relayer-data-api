    -- args: <order_id> <order_status> <side> <price> <price_cents> <position_size> <rfc3339> <timestamp_millis> <exp_time> <execution_type>
    local id = ARGV[1]
    local status = ARGV[2]
    local side = ARGV[3]
    local price = tonumber(ARGV[4])
    local price_cents = tonumber(ARGV[5])
    local size = tonumber(ARGV[6])
    local timestamp = ARGV[7]
    local time = tonumber(ARGV[8])
    local exp_time = tonumber(ARGV[9])
    -- OPEN_LIMIT or CLOSE_LIMIT or OPEN_MARKET or CLOSE_MARKET
    local execution_type = ARGV[10]
    redis.call('ECHO', 'id: ' .. id)

    if (status == "FILLED" or status == "SETTLED" or status == "LIQUIDATE") and execution_type ~= "CLOSE_LIMIT" then
        local old_price = tonumber(redis.call('HGET', 'orders', id))
        redis.call('HDEL', 'orders', id)

        
        local table = { order_id = id, side = side, price = price, positionsize = size, timestamp = timestamp }

        local order_json = cjson.encode(table)
        redis.call('ZADD', 'recent_orders', time, order_json)
        

        -- check if the order is limit order
        local result = tonumber(redis.pcall('ZRANGEBYSCORE', side, old_price, old_price)[1]) or 0
        if result == 0 then
            return
        end

        local new_size = result - (size*old_price/price_cents)

        redis.call('ZREM', side, result)
            
        if new_size > 0
        then
            redis.call('ZADD', side, old_price, new_size)
        end
        -- ------------------------------
        return
    end

    -- settle order on limit
    -- just opened a new order
    if status == "PENDING" or (status == "FILLED" and execution_type == "CLOSE_LIMIT") then
        -- if the limit order is already exist then remove the old limit price and position size
        local is_exist =redis.call('HEXISTS', 'orders', id)
        if is_exist == 1 then
            local old_price = tonumber(redis.call('HGET', 'orders', id))
            redis.call('HDEL', 'orders', id)
            local old_position_size = tonumber(redis.pcall('ZRANGEBYSCORE', side, old_price, old_price)[1]) or 0
            if old_position_size > 0 then

                local new_size = old_position_size - size

                redis.call('ZREM', side, old_position_size)
                
                if new_size > 0
                then
                    redis.call('ZADD', side, old_price, new_size)
                end
            end
        end    
        -- add the new limit price and position size
        redis.call('HSET', 'orders', id, price_cents)

        local result = tonumber(redis.pcall('ZRANGEBYSCORE', side, price_cents, price_cents)[1]) or 0
        local new_size = result + size

        if result ~= 0 then
            redis.call('ZREM', side, result)
        end
        redis.call('ZADD', side, price_cents, new_size)
    end
    -- TODO: clean out <recent_orders> expired > 24h...
    redis.call('ZREMRANGEBYSCORE', 'recent_orders', 0, exp_time)