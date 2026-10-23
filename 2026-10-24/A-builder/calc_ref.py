import ast, re, os

EXPRS = [
    "1+2*3", "(1+2)*3", "10/3", "-10/3", "10%3", "-10%3", "10/-3", "-10%-3", "2^10", "2^3^2",
    "-2^2", "(-2)^2", "2^0", "0^0", "7-3-2", "100/10/5", "1 + 2 * (3 - (4 / 2)) % 5", "((((1))))",
    "  42  ", "3 * -4", "3 - -4", "--5", "---5", "1/0", "5%0", "0/5", "(1+2", "1+", "+1", "1 2",
    "abc", "1 $ 2", "", "()", "2 ** 3", "9223372036854775807", "9223372036854775808",
    "-9223372036854775807", "9223372036854775807 + 0", "2^62", "2^-1", "1_000", "12abc",
    "１２", "2 ^ 3 ^ 2 ^ 1", "((((((((((((((((((((((((((((((((((((((((((((((((((7))))))))))))))))))))))))))))))))))))))))))))))))))",
    "1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1", "8/2/2/2", "100 - 99 - 1", "2*3%4*5", "(2+3)*(4-1)/5",
    "1/(2-2)", "10/(5-5)+3", "-(3)", "(3)(4)", "3(4)", "1+*2", "*2", ")", "1)", "99999999999 * 99999999999 / 99999999999",
    "0^5", "(-1)^3", "(-1)^4", "1^1000000000000", "(-1)^1000000000001", "5 % 3 % 2", "123456789 * 987654321",
    "2^3^0", "-5 % 3", "5 % -3", "-5 / 3", "1000000 * 1000000 * 1000000", "42\t+\t1",
]

def trunc_div(a, b):
    q = abs(a) // abs(b)
    return q if (a >= 0) == (b >= 0) else -q

def trunc_mod(a, b):
    return a - b * trunc_div(a, b)

I64MAX = 2 ** 63 - 1
I64MIN = -2 ** 63

class Err(Exception):
    def __init__(self, k):
        self.k = k

class Ovf(Exception):
    pass

def chk(v):
    if v > I64MAX or v < I64MIN:
        raise Ovf()
    return v

def ev(n):
    if isinstance(n, ast.Constant):
        return n.value
    if isinstance(n, ast.UnaryOp):
        if isinstance(n.op, ast.USub):
            return chk(-ev(n.operand))
        raise Err("parse")
    if isinstance(n, ast.BinOp):
        a = ev(n.left)
        b = ev(n.right)
        o = n.op
        if isinstance(o, ast.Add): return chk(a + b)
        if isinstance(o, ast.Sub): return chk(a - b)
        if isinstance(o, ast.Mult): return chk(a * b)
        if isinstance(o, ast.Div):
            if b == 0: raise Err("divzero")
            return chk(trunc_div(a, b))
        if isinstance(o, ast.Mod):
            if b == 0: raise Err("divzero")
            return chk(trunc_mod(a, b))
        if isinstance(o, ast.Pow):
            if b < 0: raise Err("negexp")
            if a in (0, 1, -1):
                return 1 if b == 0 else (0 if a == 0 else (1 if a == 1 else (1 if b % 2 == 0 else -1)))
            if b > 70: raise Ovf()
            return chk(a ** b)
    raise Err("parse")

def ref(src):
    if re.search(r"[^0-9+\-*/%^() \t]", src):
        return "ERR lex"
    if src.count("**") or re.search(r"\d\s*\(|\)\s*\d|\)\s*\(", src):
        # '**' and juxtaposition are parse errors in the calculator
        return "ERR parse"
    if re.search(r"(?<!\d)0\d", src):
        pass
    for m in re.finditer(r"\d+", src):
        if int(m.group()) > I64MAX:
            return "ERR lex"
    py = src.replace("^", "**").replace("/", "/")
    try:
        tree = ast.parse(py.strip(), mode="eval")
    except SyntaxError:
        return "ERR parse"
    # Python's `+x` and its rules for -2**2 match the calculator; unary plus does not exist there
    try:
        return str(ev(tree.body))
    except Err as e:
        return "ERR " + e.k
    except Ovf:
        return "OVERFLOW"

os.makedirs("exprs", exist_ok=True)
OK = [e for e in EXPRS if ref(e) != "OVERFLOW"]
OV = [e for e in EXPRS if ref(e) == "OVERFLOW"]
with open("exprs/list.txt", "w", encoding="utf-8", newline="") as f:
    f.write("\n".join(OK) + "\n")
with open("calc_ref.txt", "w", encoding="utf-8", newline="") as f:
    for e in OK:
        f.write(e + " => " + ref(e) + "\n")
with open("exprs/overflow_list.txt", "w", encoding="utf-8", newline="") as f:
    f.write("\n".join(OV) + "\n")
print(len(OK), "expressions;", len(OV), "overflow:", OV)
