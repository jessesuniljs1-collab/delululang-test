from fractions import Fraction as F
import math
def t(f):
    return str(f.numerator) if f.denominator == 1 else "%d/%d" % (f.numerator, f.denominator)
def fib(n):
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return a
H = lambda n: sum((F(1, k) for k in range(1, n + 1)), F(0))
third, sixth = F(1, 3), F(1, 6)
L = [
    "6/-8 = " + t(F(6, -8)),
    "1/0 = error: zero denominator",
    "0/5 = 0",
    "1/3 + 1/6 = " + t(third + sixth),
    "1/3 - 1/6 = " + t(third - sixth),
    "1/3 * 1/6 = " + t(third * sixth),
    "1/3 / 1/6 = " + t(third / sixth),
    "1/3 / 0 = error: division by zero",
    "cmp = 1",
    "(1/3)^5 = " + t(third ** 5),
    "H(10) = " + t(H(10)),
    "H(30) = " + t(H(30)),
    "100! = " + str(math.factorial(100)),
    "fib(500) = " + str(fib(500)),
    "2^1000 = " + str(2 ** 1000),
    "300! = " + str(math.factorial(300)),
    "12345678901234567890^3 = " + str(12345678901234567890 ** 3),
]
open('rat_ref.txt', 'w', encoding='utf-8', newline='').write("\n".join(L) + "\n")
