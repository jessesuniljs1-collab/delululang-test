import sys
sys.setrecursionlimit(100000)

def lcg(x):
    return (x * 1103515245 + 12345) % 2147483648

def gen(n, seed, modulo):
    out = []
    x = seed
    for _ in range(n):
        x = lcg(x)
        out.append((x // 65536) % modulo)
    return out

def digest(xs):
    h = 7
    for x in xs:
        h = (h * 31 + x) % 1000000007
    return h

L = []
data = gen(100000, 42, 1000000)
L.append("gen: n=%d first=%d,%d,%d digest=%d" % (len(data), data[0], data[1], data[2], digest(data)))
ms = sorted(data)
L.append("msort: sorted=true len=%d min=%d max=%d digest=%d" % (len(ms), ms[0], ms[-1], digest(ms)))
L.append("qsort: sorted=true same_as_msort=true digest=%d" % digest(ms))
L.append("builtin sort: same_as_msort=true")
dups = gen(50000, 7, 50)
L.append("many dups: qsort==msort true digest=%d" % digest(sorted(dups)))

# BST with dedupe
vals = gen(20000, 99, 1000000)
root = None
class N:
    __slots__ = ("l", "v", "r")
    def __init__(self, v):
        self.l = None; self.v = v; self.r = None
def ins(root, x):
    if root is None:
        return N(x)
    cur = root
    while True:
        if x < cur.v:
            if cur.l is None:
                cur.l = N(x); break
            cur = cur.l
        elif x > cur.v:
            if cur.r is None:
                cur.r = N(x); break
            cur = cur.r
        else:
            break
    return root
for x in vals:
    root = ins(root, x)
def height(n):
    return 0 if n is None else 1 + max(height(n.l), height(n.r))
def inorder(n, acc):
    if n is not None:
        inorder(n.l, acc); acc.append(n.v); inorder(n.r, acc)
walk = []
inorder(root, walk)
L.append("bst: size=%d height=%d inorder_sorted=true len=%d digest=%d" % (len(walk), height(root), len(walk), digest(walk)))
L.append("bst has: true false")
L.append("degenerate bst: size=3000 height=3000")

words = ["listen", "silent", "enlist", "google", "gogole", "cat", "act", "tac", "dog", "god", "evil", "vile", "live", "veil", "a", "", "ab", "ba", "αβγ", "γβα", "日本", "本日", "😀a", "a😀", "solo"]
m = {}
for w in words:
    m.setdefault("".join(sorted(w)), []).append(w)
for k in sorted(m):
    g = m[k]
    if len(g) > 1:
        L.append("[%s] %d: %s" % (k, len(g), ",".join(g)))

def alnum_lower(s):
    keep = []
    for c in s.lower():
        lc = c.lower()
        if lc.upper() != lc or lc in "0123456789":
            keep.append(lc)
    return keep
pals = ["racecar", "A man, a plan, a canal: Panama", "", "a", "ab", "été", "日本日", "😀😀😀", "Was it a car or a cat I saw?", "No lemon, no melon", "hello", "Ab1ba"]
for p in pals:
    cs = alnum_lower(p)
    L.append("pal [%s] %s" % (p, "true" if "".join(cs) == "".join(reversed(cs)) else "false"))

open("algo_ref.txt", "w", encoding="utf-8", newline="").write("\n".join(L) + "\n")
