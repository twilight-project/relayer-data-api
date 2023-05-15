import random

for i in range(0, 2880, 5):
    r = random.randint(0, 30000)
    print(f"({r}, now() + interval '{i} minute'),")
