/* PS-C-06: the native probe. Run by the guest kernel as init in place of `delulu`, on the SAME kernel
 * the image boots, it reports what a guest can reach — devices, sockets, files — as `PROBE` lines on
 * the console, and powers the VM off. It is the red team's instrument, not part of the image. */
#define _GNU_SOURCE
#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <string.h>
#include <sys/mount.h>
#include <sys/reboot.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/sysmacros.h>
#include <unistd.h>

#ifndef AF_VSOCK
#define AF_VSOCK 40
#endif

static void list_dir(const char *label, const char *path) {
    DIR *d = opendir(path);
    if (!d) {
        printf("PROBE %s: cannot open %s (%s)\n", label, path, strerror(errno));
        return;
    }
    struct dirent *e;
    int n = 0;
    while ((e = readdir(d))) {
        if (e->d_name[0] == '.') continue;
        printf("PROBE %s: %s\n", label, e->d_name);
        n++;
    }
    closedir(d);
    printf("PROBE %s: %d entries\n", label, n);
}

static void virtio_devices(void) {
    DIR *d = opendir("/sys/bus/virtio/devices");
    if (!d) {
        printf("PROBE virtio: cannot open (%s)\n", strerror(errno));
        return;
    }
    struct dirent *e;
    int n = 0;
    while ((e = readdir(d))) {
        if (e->d_name[0] == '.') continue;
        char p[512], id[32] = "?";
        snprintf(p, sizeof p, "/sys/bus/virtio/devices/%s/device", e->d_name);
        FILE *f = fopen(p, "r");
        if (f) {
            if (fgets(id, sizeof id, f)) id[strcspn(id, "\n")] = 0;
            fclose(f);
        }
        /* virtio ids: 0x0001 net, 0x0002 block, 0x0004 rng, 0x0009 9p, 0x0013 vsock, 0x001a fs */
        printf("PROBE virtio device: %s id=%s\n", e->d_name, id);
        n++;
    }
    closedir(d);
    printf("PROBE virtio devices: %d\n", n);
}

static void try_socket(const char *name, int family, int type) {
    int fd = socket(family, type, 0);
    if (fd >= 0) {
        printf("PROBE socket %s: CREATED\n", name);
        close(fd);
    } else {
        printf("PROBE socket %s: refused (%s)\n", name, strerror(errno));
    }
}

int main(int argc, char **argv, char **envp) {
    for (int i = 0; i < argc; i++) printf("PROBE argv[%d]: %s\n", i, argv[i]);
    for (char **e = envp; *e; e++) printf("PROBE env: %s\n", *e);
    list_dir("root", "/");
    mkdir("/sys", 0555);
    if (mount("sysfs", "/sys", "sysfs", 0, NULL) != 0) printf("PROBE mount sysfs: %s\n", strerror(errno));
    virtio_devices();
    list_dir("net interface", "/sys/class/net");
    list_dir("block device", "/sys/class/block");
    list_dir("virtio drivers", "/sys/bus/virtio/drivers");
    try_socket("AF_INET/stream", AF_INET, SOCK_STREAM);
    try_socket("AF_INET/dgram", AF_INET, SOCK_DGRAM);
    try_socket("AF_INET6/stream", AF_INET6, SOCK_STREAM);
    try_socket("AF_PACKET/raw", AF_PACKET, SOCK_RAW);
    try_socket("AF_NETLINK/raw", AF_NETLINK, SOCK_RAW);
    try_socket("AF_UNIX/stream", AF_UNIX, SOCK_STREAM);
    try_socket("AF_VSOCK/stream", AF_VSOCK, SOCK_STREAM);
    /* A block device the host did not give: the node can be made, but no driver answers it. */
    if (mknod("/vda", S_IFBLK | 0600, makedev(254, 0)) == 0) {
        int fd = open("/vda", O_RDONLY);
        printf("PROBE open block 254:0: %s\n", fd >= 0 ? "OPENED" : strerror(errno));
        if (fd >= 0) close(fd);
    } else {
        printf("PROBE mknod block: %s\n", strerror(errno));
    }
    int fd = open("/proc/self/status", O_RDONLY);
    printf("PROBE /proc: %s\n", fd >= 0 ? "PRESENT" : strerror(errno));
    if (fd >= 0) close(fd);
    printf("PROBE done\n");
    fflush(stdout);
    sync();
    reboot(RB_AUTOBOOT);
    return 0;
}
