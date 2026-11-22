n = 0
for i in range(200000):
    parts = "cust42,519".split(",")
    n = n + len(parts)
print(n)
