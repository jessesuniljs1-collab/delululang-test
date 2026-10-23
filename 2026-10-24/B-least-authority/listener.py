import socket, threading, time, sys, struct, os

LOG = sys.argv[1]
PORTS = [int(p) for p in sys.argv[2].split(",")]
HOSTS = sys.argv[3].split(",") if len(sys.argv) > 3 else ["127.0.0.1"]
lock = threading.Lock()

def parse_sni(b):
    try:
        if len(b) < 6 or b[0] != 0x16:
            return "-"
        # record header 5, handshake header 4
        i = 5
        if b[i] != 0x01:
            return "-"
        i += 4
        i += 2 + 32  # version + random
        sl = b[i]; i += 1 + sl
        cl = struct.unpack(">H", b[i:i+2])[0]; i += 2 + cl
        cm = b[i]; i += 1 + cm
        el = struct.unpack(">H", b[i:i+2])[0]; i += 2
        end = i + el
        while i + 4 <= end:
            t, l = struct.unpack(">HH", b[i:i+4]); i += 4
            if t == 0:
                nl = struct.unpack(">H", b[i+3:i+5])[0]
                return b[i+5:i+5+nl].decode("ascii", "replace")
            i += l
        return "-"
    except Exception as e:
        return "sni-parse-error:%s" % e

def handle(conn, addr, lport):
    conn.settimeout(2.0)
    data = b""
    try:
        data = conn.recv(512)
    except Exception:
        pass
    sni = parse_sni(data)
    with lock:
        with open(LOG, "a") as f:
            f.write("%s lport=%d peer=%s client_port=%d sni=%s len=%d hex=%s\n" % (
                time.strftime("%H:%M:%S"), lport, addr[0], addr[1], sni, len(data), data[:512].hex()))
    try:
        conn.close()
    except Exception:
        pass

def serve(host, port):
    s = socket.socket(socket.AF_INET6 if ":" in host else socket.AF_INET, socket.SOCK_STREAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    try:
        s.bind((host, port))
    except Exception as e:
        with lock:
            with open(LOG, "a") as f:
                f.write("BINDFAIL %s:%d %s\n" % (host, port, e))
        return
    s.listen(50)
    with lock:
        with open(LOG, "a") as f:
            f.write("LISTENING %s:%d\n" % (host, port))
    while True:
        c, a = s.accept()
        threading.Thread(target=handle, args=(c, a, port), daemon=True).start()

for h in HOSTS:
    for p in PORTS:
        threading.Thread(target=serve, args=(h, p), daemon=True).start()
while True:
    time.sleep(1)
