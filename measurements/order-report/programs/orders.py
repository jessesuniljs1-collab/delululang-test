from dataclasses import dataclass


@dataclass
class Order:
    customer: str
    item: str
    qty: int
    cents: int


def parse_line(line):
    f = line.split(",")
    if len(f) != 4:
        raise ValueError("expected 4 fields: " + line)
    try:
        qty = int(f[2].strip())
    except ValueError:
        raise ValueError("bad quantity: " + line)
    try:
        cents = int(f[3].strip())
    except ValueError:
        raise ValueError("bad price: " + line)
    return Order(f[0].strip(), f[1].strip(), qty, cents)


def money(c):
    return f"{c // 100}.{c % 100:02d}"


orders, errors, totals = [], [], {}
for line in open("data/orders.csv").read().split("\n"):
    if not line.strip():
        continue
    try:
        orders.append(parse_line(line))
    except ValueError as e:
        errors.append(str(e))
for o in orders:
    cents, count = totals.get(o.customer, (0, 0))
    totals[o.customer] = (cents + o.qty * o.cents, count + 1)
report = ""
for name, (cents, count) in sorted(totals.items(), key=lambda kv: -kv[1][0]):
    report += f"{name:<8}{money(cents):>10}  ({count} orders)\n"
for e in errors:
    report += "skipped: " + e + "\n"
open("out/report.txt", "w", newline="\n").write(report)
print(f"wrote report.txt: {len(orders)} orders, {len(errors)} skipped")
