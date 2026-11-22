xs = []
s = 12345
for i in range(200000):
    s = (s * 1103515245 + 12345) % 2147483648
    xs.append(s)
ys = sorted(xs)
check = 0
for i in range(1000):
    check = (check + ys[i * 200]) % 1000000007
print(check)
