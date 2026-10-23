import socket, threading, time, sys

# ports and modes: 18443 read-then-close, 18444 read-then-plain-HTTP-reply, 18445 accept and stay silent, 18446 close at once
MODES = {18443: "readclose", 18444: "plainreply", 18445: "silent", 18446: "closenow"}
LOG = open("net/listener.log", "a", encoding="utf-8", buffering=1)
lock = threading.Lock()

def log(msg):
    with lock:
        LOG.write("%.3f %s\n" % (time.time(), msg))

def handle(conn, addr, port, mode):
    t0 = time.time()
    log("port %d conn from %s mode=%s" % (port, addr, mode))
    try:
        if mode == "closenow":
            conn.close()
            log("port %d closed immediately" % port)
            return
        conn.settimeout(3.0)
        data = b""
        try:
            while len(data) < 65536:
                chunk = conn.recv(4096)
                if not chunk:
                    log("port %d peer closed after %d bytes" % (port, len(data)))
                    break
                data += chunk
                if mode in ("readclose", "plainreply") and len(data) >= 5:
                    # a TLS record is 5-byte header + body; once we have the record, stop reading
                    if data[0] == 0x16 and len(data) >= 5 + int.from_bytes(data[3:5], "big"):
                        break
        except socket.timeout:
            log("port %d read timeout after %d bytes" % (port, len(data)))
        kind = "TLS-handshake-record" if data[:1] == b"\x16" and data[1:2] == b"\x03" else ("plain-http?" if data[:4] in (b"GET ", b"POST", b"PUT ", b"HEAD", b"PATC", b"DELE") else "other")
        log("port %d received %d bytes kind=%s head=%s" % (port, len(data), kind, data[:48].hex()))
        # SNI / ALPN visibility (host name the client announces)
        if b"127.0.0.1" in data:
            log("port %d ClientHello mentions 127.0.0.1 literal" % port)
        if mode == "plainreply":
            conn.sendall(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nhello")
            log("port %d sent plain HTTP 200" % port)
        elif mode == "silent":
            end = time.time() + 20
            while time.time() < end:
                time.sleep(0.5)
            log("port %d silent hold released after %.1fs" % (port, time.time() - t0))
    except Exception as e:
        log("port %d error %r" % (port, e))
    finally:
        try:
            conn.close()
        except Exception:
            pass

def serve(port, mode):
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind(("127.0.0.1", port))
    s.listen(16)
    log("listening on %d (%s)" % (port, mode))
    while True:
        c, a = s.accept()
        threading.Thread(target=handle, args=(c, a, port, mode), daemon=True).start()

for p, m in MODES.items():
    threading.Thread(target=serve, args=(p, m), daemon=True).start()
deadline = time.time() + 1200
while time.time() < deadline:
    time.sleep(1)
log("listener deadline reached, exiting")
