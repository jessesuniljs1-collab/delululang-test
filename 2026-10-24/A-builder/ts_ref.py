import os, re, sys
from collections import Counter
def files():
    out=[]
    for root,dirs,fs in os.walk('data'):
        for f in fs:
            if f.endswith('.txt'):
                out.append(os.path.relpath(os.path.join(root,f),'data').replace('\\','/'))
    return sorted(out)
def lines(t):
    p=t.split('\n')
    return len(p)-1 if p[-1]=='' else len(p)
def words(t):
    t=re.sub(r'[\r\n\t.,;!?]',' ',t).lower()
    return [w for w in t.split(' ') if w]
def top(c,n=10):
    items=sorted(c.items(), key=lambda kv:(-kv[1], kv[0]))
    return ["  %s %d"%(w,k) for w,k in items[:n]]
rep=[]
tot=Counter(); tl=tw=tc=0
for f in files():
    t=open('data/'+f,encoding='utf-8',newline='').read()
    c=Counter(words(t)); l=lines(t); w=sum(c.values()); ch=len(t)
    tl+=l;tw+=w;tc+=ch; tot.update(c)
    rep.append("%s lines=%d words=%d chars=%d unique=%d"%(f,l,w,ch,len(c)))
    rep+=top(c)
rep.append("TOTAL lines=%d words=%d chars=%d unique=%d"%(tl,tw,tc,len(tot)))
rep+=top(tot)
open('ts_ref.txt','w',encoding='utf-8',newline='').write("\n".join(rep)+"\n")
