totals = {}
for i in range(200000):
    line = f"cust{i % 100},{i % 977}"
    parts = line.split(",")
    name = parts[0]
    cents = int(parts[1])
    totals[name] = totals.get(name, 0) + cents
print(len(totals), sum(totals.values()))
