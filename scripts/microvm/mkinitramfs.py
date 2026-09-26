#!/usr/bin/env python3
"""Write the guest's initramfs (PS-C-02): a newc cpio archive, byte-for-byte deterministic.

usage: mkinitramfs.py <static-delulu> <output.cpio>

The archive holds four entries and nothing else: `/dev`, the two device nodes a process needs before
it can print (`/dev/console`, `/dev/null` — Rust's runtime opens `/dev/null` for any standard stream
it finds closed, and aborts if it cannot), and `/delulu`, which the kernel runs as PID 1.

Every field a build could vary is fixed — modification times are 0, owners are 0:0, inode numbers
count up in the order below — and nothing is read from the build machine but the one binary. The
device nodes are archive entries, not files on disk, so writing them needs no privilege; that is why
this is a script and not `cpio -o`, which would need them to exist first (and root to make them).
The archive is uncompressed: the guest kernel carries no decompressor.
"""
import sys


def entry(out, ino, name, mode, data=b"", rdev=(0, 0), nlink=1):
    name_b = name.encode() + b"\0"
    fields = (ino, mode, 0, 0, nlink, 0, len(data), 0, 0, rdev[0], rdev[1], len(name_b), 0)
    out += ("070701" + "".join("%08X" % v for v in fields)).encode() + name_b
    out += b"\0" * ((4 - len(out) % 4) % 4)
    out += data
    out += b"\0" * ((4 - len(out) % 4) % 4)


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__.strip().splitlines()[2])
    binary, dest = sys.argv[1], sys.argv[2]
    with open(binary, "rb") as f:
        data = f.read()
    if not data.startswith(b"\x7fELF"):
        sys.exit(f"{binary} is not an ELF executable")
    out = bytearray()
    entry(out, 1, "dev", 0o040755, nlink=2)
    entry(out, 2, "dev/console", 0o020600, rdev=(5, 1))
    entry(out, 3, "dev/null", 0o020666, rdev=(1, 3))
    entry(out, 4, "delulu", 0o100755, data)
    entry(out, 0, "TRAILER!!!", 0)
    with open(dest, "wb") as f:
        f.write(bytes(out))


main()
