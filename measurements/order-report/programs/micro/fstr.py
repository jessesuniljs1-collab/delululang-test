n = 0
for i in range(200000):
    line = f"cust{i % 100},{i % 977}"
    n = n + len(line)
print(n)
