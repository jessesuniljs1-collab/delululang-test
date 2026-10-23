import sys
W=r"C:\Users\jesse\AppData\Local\Temp\claude\D--nelan-DeluluLang\eb34182b-835b-44af-b5ec-07064fcae52c\scratchpad\dogfood"
for l in open(sys.argv[1],encoding='utf-8',errors='replace'):
    l=l.replace(W,"<DF>").replace(W.replace("\\","/"),"<DF>").replace("g_619ec6eea2f10eeebabf262ac85606b0","g_B-read").replace("g_838f9483c4516233679526a24d51ea9c","g_B-rw").replace("g_e3d521bda5e61da2e2beab73c5caf811","g_B-wide")
    print(l.rstrip()[:int(sys.argv[2]) if len(sys.argv)>2 else 240])
