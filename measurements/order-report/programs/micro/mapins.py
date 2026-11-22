totals = {}
names = [f"cust{k}" for k in range(100)]
for i in range(200000):
    name = names[i % 100]
    totals[name] = totals.get(name, 0) + 1
print(len(totals))
