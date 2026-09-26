/* PS-C-06: a HOSTILE guest. It stands for an image whose guest has been subverted — a compromised
 * build the operator's manifest nonetheless vouches for — and does, as PID 1, what such a guest could
 * try against its host. Every attempt prints one `HOSTILE` line on the console; the host's side of
 * each is what the red-team record reads. The launcher always passes its own arguments, so the mode
 * is compiled in (`-DHOSTILE_MODE=\"flood\"`):
 *
 *   flood     write 1 MiB to the console, then never dial in
 *   garbage   dial in, send the ready byte, then random bytes
 *   huge      dial in, send the ready byte, then a frame length of 4 GiB - 1
 *   ports     dial the host on other ports and other context ids, then the real one, twice
 *   silent    dial in, send the ready byte, then say nothing at all
 */
#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/reboot.h>
#include <sys/socket.h>
#include <sys/time.h>
#include <unistd.h>

#ifndef AF_VSOCK
#define AF_VSOCK 40
#endif
struct sockaddr_vm {
    unsigned short svm_family;
    unsigned short svm_reserved1;
    unsigned int svm_port;
    unsigned int svm_cid;
    unsigned char svm_zero[4];
};

static int dial(unsigned int cid, unsigned int port) {
    int fd = socket(AF_VSOCK, SOCK_STREAM, 0);
    if (fd < 0) return -errno;
    struct sockaddr_vm a;
    memset(&a, 0, sizeof a);
    a.svm_family = AF_VSOCK;
    a.svm_cid = cid;
    a.svm_port = port;
    if (connect(fd, (struct sockaddr *)&a, sizeof a) != 0) {
        int e = errno;
        close(fd);
        return -e;
    }
    return fd;
}

static void report(const char *what, int r) {
    if (r >= 0) printf("HOSTILE %s: CONNECTED\n", what);
    else printf("HOSTILE %s: refused (%s)\n", what, strerror(-r));
    fflush(stdout);
}

static void end(void) {
    printf("HOSTILE done\n");
    fflush(stdout);
    sync();
    reboot(RB_AUTOBOOT);
}

#ifndef HOSTILE_MODE
#define HOSTILE_MODE "flood"
#endif

int main(void) {
    const char *mode = HOSTILE_MODE;
    printf("HOSTILE mode: %s\n", mode);
    fflush(stdout);
    if (strcmp(mode, "flood") == 0) {
        char line[1024];
        memset(line, 'A', sizeof line - 1);
        line[sizeof line - 1] = '\n';
        for (int i = 0; i < 1024; i++) (void)!write(1, line, sizeof line);
        printf("HOSTILE flood: wrote 1 MiB\n");
        fflush(stdout);
        pause();
    }
    int fd = dial(2, 1024);
    report("dial host 1024", fd);
    if (fd < 0) end();
    unsigned char ready = 0x06;
    (void)!write(fd, &ready, 1);
    if (strcmp(mode, "garbage") == 0) {
        unsigned char buf[4096];
        uint32_t x = 2463534242u;
        for (size_t i = 0; i < sizeof buf; i++) {
            x ^= x << 13; x ^= x >> 17; x ^= x << 5;
            buf[i] = (unsigned char)x;
        }
        ssize_t n = write(fd, buf, sizeof buf);
        printf("HOSTILE garbage: wrote %zd bytes\n", n);
    } else if (strcmp(mode, "huge") == 0) {
        unsigned char len[4] = {0xff, 0xff, 0xff, 0xff};
        ssize_t n = write(fd, len, 4);
        printf("HOSTILE huge: announced a 4 GiB frame (%zd)\n", n);
        for (int i = 0; i < 64; i++) (void)!write(fd, "xxxxxxxxxxxxxxxx", 16);
    } else if (strcmp(mode, "ports") == 0) {
        /* A second channel: if it connects at all (it can land in the listening socket's backlog before
         * the host has accepted the first), is anything on the host reading it? */
        int second = dial(2, 1024);
        report("dial host 1024 again", second);
        if (second >= 0) {
            struct timeval tv = {5, 0};
            setsockopt(second, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof tv);
            char b;
            ssize_t r = read(second, &b, 1);
            if (r == 0) printf("HOSTILE second channel: closed by the host, unread (EOF)\n");
            else if (r < 0) printf("HOSTILE second channel: %s\n", strerror(errno));
            else printf("HOSTILE second channel: the host SENT a byte\n");
            fflush(stdout);
            close(second);
        }
        report("dial host 1025", dial(2, 1025));
        report("dial host 52", dial(2, 52));
        report("dial host 0xffffffff", dial(2, 0xffffffffu));
        report("dial cid 1 (local)", dial(1, 1024));
        report("dial cid 3 (itself)", dial(3, 1024));
    } else if (strcmp(mode, "silent") == 0) {
        printf("HOSTILE silent: holding the channel, saying nothing\n");
        fflush(stdout);
        pause();
    }
    fflush(stdout);
    /* Keep the channel open a moment, so the host meets the bytes, not a hang-up. */
    sleep(2);
    close(fd);
    end();
    return 0;
}
