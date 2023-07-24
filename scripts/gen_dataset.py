import random

for i in range(0, 2880, 5):
    r = random.randint(0, 30000)
    print(f"({r}, now() + interval '{i} minute'),")

for i in range(0, 2880, 5):
    account = random.choice(['one', 'two', 'three'])
    position_type = random.choice(['LONG', 'SHORT'])
    status = random.choice(['PENDING', 'SETTLED', 'FILLED'])
    order_type = random.choice(['LIMIT', 'MARKET'])
    entryprice = random.random()*5000
    execution_price = entryprice + (random.random() - 0.5)*100
    positionsize = random.randint(200, 5000)
    leverage = random.randint(1, 10)
    initial_margin = random.randint(1, 10)
    available_margin = random.randint(1, 10)
    bankruptcy_price = random.random()*2000
    bankruptcy_value = random.random()*2000
    maintenance_margin = random.random()*2000
    liquidation_price = random.random()*2000
    unrealized_pnl = random.random()*2000
    settlement_price = random.random()*2000
    entry_nonce = i*10
    exit_nonce = entry_nonce + random.randint(3, 20)
    entry_sequence = i

    c1_x = f"(gen_random_uuid(), '{account}', '{position_type}', '{status}', "
    c2_x = c1_x + f"'{order_type}', {entryprice}, {execution_price}, {positionsize}, "
    c3_x = c2_x + f"{leverage}, {initial_margin}, {available_margin}, now() + interval '{i} minute', "
    c4_x = c3_x + f"{bankruptcy_price}, {bankruptcy_value}, {maintenance_margin}, {liquidation_price}, "
    c5_x = c4_x + f"{unrealized_pnl}, {settlement_price}, {entry_nonce}, {exit_nonce}, {entry_sequence}),"
    print(c5_x)
