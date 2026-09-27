//! Opening a checked path so that what is opened is what was checked (campaign finding FS-RACE-1).
//!
//! # The finding (2026-09-27, the multi-OS agent pass, Sonnet 5 on Linux)
//!
//! Containment was CHECK-then-USE. [`crate::prim`] resolved a program's path — canonicalizing every
//! link on it (C84) and refusing a link it could not resolve (SYMLINK-DANGLE-1) — and then handed the
//! LEXICAL path to `std::fs::write`, which asked the operating system to resolve it all over again.
//! Between the two, the path could change. With `scope/race_link` flipped in a loop between an inside
//! target and `../outside/race_target.txt`, 81 of 150 single-shot `write_text("race_link", …)` runs
//! reported `Ok`, and the file OUTSIDE the grant held the payload; the same link held still was
//! refused `DL0904` every time. It reproduced under `--sandbox` (40 of 80), because the host performs a
//! guest's effects with this same code.
//!
//! Who can race: not a DeluluLang program — the language has no way to make a link. The racer is
//! another process with write access to the granted directory: a less trusted user sharing it, or
//! anything else on the machine. That is the confused-deputy case, and it is exactly the case a
//! grant is supposed to bound.
//!
//! # The rule now
//!
//! The path the check approved is opened ONE COMPONENT AT A TIME from the top of the filesystem, and
//! no component is followed if it is a link: on Unix each step is `openat(parent, name, O_NOFOLLOW)`;
//! on Windows it is `NtCreateFile` relative to the parent's handle with `FILE_OPEN_REPARSE_POINT`,
//! and a directory that turns out to be a junction or symlink is refused. The path walked is the
//! CANONICAL one — every link the program was allowed to use has already been resolved into it — so a
//! link met during the walk is one that appeared after the check, and it is refused, not followed.
//! A link that stays inside the grant keeps working exactly as before, because it was resolved at
//! check time.
//!
//! Only regular files are read or written (the FIFO finding of the same pass): a FIFO, a device or
//! a socket placed in a granted directory is refused rather than opened, because reading one can
//! block for ever and writing one reaches whatever is on the other end. On Unix the open itself is
//! non-blocking, so even the refusal cannot hang.
//!
//! # What this does not close — stated so nobody oversells it
//!
//! * **Hard links.** A hard link inside the grant IS a file inside the grant; there is no link to
//!   refuse. A process that can make one to a file it can already reach has the file. Documented as
//!   a residual, as it was before (HARDLINK-1).
//! * **The grant's own path.** The walk starts at the filesystem root and refuses a link anywhere on
//!   the canonical path, but the canonical form of the GRANTED directory is computed at use time: a
//!   granted directory that is itself a symlink is followed when it is resolved, as it always was.
//!   The directory an operator grants, and its parents, are the operator's to keep.

use std::fs::File;
use std::io::{self, Read as _, Write as _};
use std::path::{Path, PathBuf};

/// What happened when a checked path was opened.
#[derive(Debug)]
pub enum Outcome<T> {
    /// The operation ran on the file the check approved.
    Done(T),
    /// An ordinary I/O failure — not found, permission, not a regular file. The program sees an
    /// `Err`, as it always has for these.
    Io(io::Error),
    /// The path is not inside the grant — or stopped being inside it between the check and the open.
    /// The caller refuses with `DL0904`; the text says why, in the operator's terms.
    Escaped(&'static str),
}

/// The reason given when the path resolved outside the grant at open time.
pub const OUTSIDE: &str = "it resolves outside the grant";
/// The reason given when a link appeared on the path after the check.
pub const SWAPPED: &str = "a component became a symbolic link or junction after the scope check \
                           (the path changed while it was being opened); refused rather than followed";

/// What to do with the file at the end of the path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Access {
    Read,
    /// Create if absent; truncated only AFTER it is known to be a regular file.
    Write,
    /// Create if absent; every write lands at the end.
    Append,
    /// The path names a directory whose entries are listed.
    List,
}

/// Resolve `joined` (a path the caller built inside `root`) and check it against `root` on disk —
/// the same resolution [`crate::prim::contains_on_disk`] makes — returning the canonical path to
/// open, or why it is refused.
fn checked(root: &Path, joined: &Path) -> Result<PathBuf, &'static str> {
    let (Some(r), Some(c)) = (crate::prim::resolve_for_decision(root), crate::prim::resolve_for_decision(joined))
    else {
        return Err(OUTSIDE);
    };
    if c.starts_with(&r) {
        Ok(c)
    } else {
        Err(OUTSIDE)
    }
}

fn not_regular() -> io::Error {
    io::Error::other(
        "not a regular file: a FIFO, device or socket in a granted directory is refused, because \
         reading one can block for ever and writing one reaches whatever is on the other end",
    )
}

/// Read a regular file inside `root` as UTF-8 text.
pub fn read_text(root: &Path, joined: &Path) -> Outcome<String> {
    match open(root, joined, Access::Read) {
        Outcome::Done(Opened::File(mut f)) => {
            let mut s = String::new();
            match f.read_to_string(&mut s) {
                Ok(_) => Outcome::Done(s),
                Err(e) => Outcome::Io(e),
            }
        }
        Outcome::Done(Opened::Dir(_)) => Outcome::Io(not_regular()),
        Outcome::Io(e) => Outcome::Io(e),
        Outcome::Escaped(w) => Outcome::Escaped(w),
    }
}

/// Read a regular file inside `root` as bytes (a plugin artifact).
pub fn read_bytes(root: &Path, joined: &Path) -> Outcome<Vec<u8>> {
    match open(root, joined, Access::Read) {
        Outcome::Done(Opened::File(mut f)) => {
            let mut b = Vec::new();
            match f.read_to_end(&mut b) {
                Ok(_) => Outcome::Done(b),
                Err(e) => Outcome::Io(e),
            }
        }
        Outcome::Done(Opened::Dir(_)) => Outcome::Io(not_regular()),
        Outcome::Io(e) => Outcome::Io(e),
        Outcome::Escaped(w) => Outcome::Escaped(w),
    }
}

/// Write (or append) `body` to a regular file inside `root`, creating it if absent.
pub fn write_text(root: &Path, joined: &Path, body: &str, append: bool) -> Outcome<()> {
    match open(root, joined, if append { Access::Append } else { Access::Write }) {
        Outcome::Done(Opened::File(mut f)) => {
            let res = if append { Ok(()) } else { f.set_len(0) };
            match res.and_then(|()| f.write_all(body.as_bytes())) {
                Ok(()) => Outcome::Done(()),
                Err(e) => Outcome::Io(e),
            }
        }
        Outcome::Done(Opened::Dir(_)) => Outcome::Io(not_regular()),
        Outcome::Io(e) => Outcome::Io(e),
        Outcome::Escaped(w) => Outcome::Escaped(w),
    }
}

/// The entry names of a directory inside `root` (`.` and `..` excluded), in the order the OS gives.
pub fn list_dir(root: &Path, joined: &Path) -> Outcome<Vec<String>> {
    match open(root, joined, Access::List) {
        Outcome::Done(Opened::Dir(d)) => match imp::entries(d) {
            Ok(v) => Outcome::Done(v),
            Err(e) => Outcome::Io(e),
        },
        Outcome::Done(Opened::File(_)) => Outcome::Io(io::Error::new(io::ErrorKind::NotADirectory, "not a directory")),
        Outcome::Io(e) => Outcome::Io(e),
        Outcome::Escaped(w) => Outcome::Escaped(w),
    }
}

enum Opened {
    File(File),
    Dir(imp::DirHandle),
}

fn open(root: &Path, joined: &Path, access: Access) -> Outcome<Opened> {
    let real = match checked(root, joined) {
        Ok(p) => p,
        Err(why) => return Outcome::Escaped(why),
    };
    imp::open_nofollow(&real, access)
}

// ----- Unix: openat, one component at a time, O_NOFOLLOW ---------------------------------------

#[cfg(unix)]
mod imp {
    use super::{not_regular, Access, Opened, Outcome, SWAPPED};
    use std::ffi::{CStr, CString, OsStr};
    use std::fs::File;
    use std::io;
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    use std::os::unix::ffi::OsStrExt;
    use std::path::{Component, Path};

    pub struct DirHandle(OwnedFd);

    // `O_PATH` on Linux: a directory on the way need only be searchable, as with an ordinary path
    // lookup. Elsewhere `O_RDONLY`, which also needs read permission on each directory walked.
    #[cfg(any(target_os = "linux", target_os = "android"))]
    const WALK: libc::c_int = libc::O_PATH | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC;
    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    const WALK: libc::c_int = libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC;

    fn cstr(s: &OsStr) -> io::Result<CString> {
        CString::new(s.as_bytes()).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "a NUL in a path"))
    }

    /// `openat(dir, name, flags)`, and if it failed, whether `name` is now a link — the difference
    /// between an ordinary error and the race this module exists for.
    fn step(dir: libc::c_int, name: &CStr, flags: libc::c_int) -> Result<OwnedFd, Outcome<Opened>> {
        // SAFETY: `name` is a valid NUL-terminated string and `dir` a descriptor we own; the mode
        // argument is read only when O_CREAT is set.
        let fd = unsafe { libc::openat(dir, name.as_ptr(), flags, 0o666 as libc::c_uint) };
        if fd >= 0 {
            // SAFETY: a fresh descriptor, owned by nothing else.
            return Ok(unsafe { OwnedFd::from_raw_fd(fd) });
        }
        let e = io::Error::last_os_error();
        if is_link_at(dir, name) {
            return Err(Outcome::Escaped(SWAPPED));
        }
        Err(Outcome::Io(e))
    }

    fn is_link_at(dir: libc::c_int, name: &CStr) -> bool {
        // SAFETY: zeroed is a valid `stat`; the call only writes into it.
        let mut st: libc::stat = unsafe { std::mem::zeroed() };
        // SAFETY: as above; AT_SYMLINK_NOFOLLOW stats the link itself.
        let rc = unsafe { libc::fstatat(dir, name.as_ptr(), &mut st, libc::AT_SYMLINK_NOFOLLOW) };
        rc == 0 && (st.st_mode & libc::S_IFMT) == libc::S_IFLNK
    }

    fn file_type(fd: libc::c_int) -> io::Result<libc::mode_t> {
        // SAFETY: as in `is_link_at`.
        let mut st: libc::stat = unsafe { std::mem::zeroed() };
        if unsafe { libc::fstat(fd, &mut st) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(st.st_mode & libc::S_IFMT)
    }

    pub(super) fn open_nofollow(real: &Path, access: Access) -> Outcome<Opened> {
        let mut names = Vec::new();
        for c in real.components() {
            match c {
                Component::RootDir => {}
                Component::Normal(n) => match cstr(n) {
                    Ok(s) => names.push(s),
                    Err(e) => return Outcome::Io(e),
                },
                // A canonical path has neither; meeting one means it is not the path that was checked.
                _ => return Outcome::Escaped(SWAPPED),
            }
        }
        // SAFETY: a constant path; `/` is never a link.
        let top = unsafe { libc::open(c"/".as_ptr(), WALK & !libc::O_NOFOLLOW) };
        if top < 0 {
            return Outcome::Io(io::Error::last_os_error());
        }
        // SAFETY: fresh descriptor.
        let mut cur = unsafe { OwnedFd::from_raw_fd(top) };
        let Some((last, dirs)) = names.split_last() else {
            // The path is `/` itself.
            return if access == Access::List { reopen_dir(cur) } else { Outcome::Io(not_regular()) };
        };
        for d in dirs {
            cur = match step(cur.as_raw_fd(), d, WALK) {
                Ok(fd) => fd,
                Err(o) => return o,
            };
        }
        let base = libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK;
        let flags = match access {
            Access::Read => libc::O_RDONLY | base,
            Access::Write => libc::O_WRONLY | libc::O_CREAT | base,
            Access::Append => libc::O_WRONLY | libc::O_CREAT | libc::O_APPEND | base,
            Access::List => libc::O_RDONLY | libc::O_DIRECTORY | base,
        };
        let fd = match step(cur.as_raw_fd(), last, flags) {
            Ok(fd) => fd,
            Err(o) => return o,
        };
        let kind = match file_type(fd.as_raw_fd()) {
            Ok(k) => k,
            Err(e) => return Outcome::Io(e),
        };
        match (access, kind) {
            (Access::List, libc::S_IFDIR) => Outcome::Done(Opened::Dir(DirHandle(fd))),
            (Access::List, _) => Outcome::Io(io::Error::new(io::ErrorKind::NotADirectory, "not a directory")),
            (_, libc::S_IFREG) => {
                // Back to blocking I/O for the read or write itself (a regular file ignores the
                // flag, but nothing should depend on that).
                // SAFETY: plain fcntl on a descriptor we own.
                unsafe {
                    let fl = libc::fcntl(fd.as_raw_fd(), libc::F_GETFL);
                    if fl >= 0 {
                        libc::fcntl(fd.as_raw_fd(), libc::F_SETFL, fl & !libc::O_NONBLOCK);
                    }
                }
                Outcome::Done(Opened::File(File::from(fd)))
            }
            (_, libc::S_IFDIR) => Outcome::Io(io::Error::new(io::ErrorKind::IsADirectory, "is a directory")),
            _ => Outcome::Io(not_regular()),
        }
    }

    fn reopen_dir(fd: OwnedFd) -> Outcome<Opened> {
        // SAFETY: `.` relative to a directory descriptor we own.
        let d = unsafe { libc::openat(fd.as_raw_fd(), c".".as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC) };
        if d < 0 {
            return Outcome::Io(io::Error::last_os_error());
        }
        // SAFETY: fresh descriptor.
        Outcome::Done(Opened::Dir(DirHandle(unsafe { OwnedFd::from_raw_fd(d) })))
    }

    pub(super) fn entries(d: DirHandle) -> io::Result<Vec<String>> {
        use std::os::fd::IntoRawFd;
        let raw = d.0.into_raw_fd();
        // SAFETY: fdopendir takes ownership of `raw`; closedir below releases both.
        let dir = unsafe { libc::fdopendir(raw) };
        if dir.is_null() {
            let e = io::Error::last_os_error();
            // SAFETY: fdopendir failed, so `raw` is still ours to close.
            unsafe { libc::close(raw) };
            return Err(e);
        }
        let mut out = Vec::new();
        loop {
            // SAFETY: `dir` is a valid DIR*; readdir returns null at the end (or on error).
            let ent = unsafe { libc::readdir(dir) };
            if ent.is_null() {
                break;
            }
            // SAFETY: d_name is NUL-terminated for the lifetime of this entry.
            let name = unsafe { CStr::from_ptr((*ent).d_name.as_ptr()) };
            let b = name.to_bytes();
            if b != b"." && b != b".." {
                out.push(String::from_utf8_lossy(b).into_owned());
            }
        }
        // SAFETY: closes the stream and its descriptor.
        unsafe { libc::closedir(dir) };
        Ok(out)
    }
}

// ----- Windows: NtCreateFile relative to the parent's handle, FILE_OPEN_REPARSE_POINT ----------

#[cfg(windows)]
mod imp {
    use super::{not_regular, Access, Opened, Outcome, SWAPPED};
    use std::fs::File;
    use std::io;
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use std::path::{Component, Path, PathBuf};

    use windows_sys::Wdk::Foundation::OBJECT_ATTRIBUTES;
    use windows_sys::Wdk::Storage::FileSystem::{
        NtCreateFile, FILE_DIRECTORY_FILE, FILE_NON_DIRECTORY_FILE, FILE_OPEN, FILE_OPEN_IF,
        FILE_OPEN_REPARSE_POINT, FILE_SYNCHRONOUS_IO_NONALERT,
    };
    use windows_sys::Win32::Foundation::{RtlNtStatusToDosError, HANDLE, OBJ_CASE_INSENSITIVE, UNICODE_STRING};
    use windows_sys::Win32::Foundation::ERROR_NO_MORE_FILES;
    use windows_sys::Win32::Storage::FileSystem::{
        FileFullDirectoryInfo, GetFileInformationByHandleEx, FILE_APPEND_DATA, FILE_ATTRIBUTE_NORMAL,
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_FULL_DIR_INFO, FILE_GENERIC_READ, FILE_GENERIC_WRITE,
        FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
        FILE_TRAVERSE, FILE_WRITE_ATTRIBUTES, SYNCHRONIZE,
    };
    use windows_sys::Win32::System::IO::IO_STATUS_BLOCK;

    /// A directory opened for listing, enumerated through its HANDLE — never looked up by path again.
    pub struct DirHandle(OwnedHandle);

    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;

    /// `NtCreateFile(parent \ name)`: `name` is ONE component, opened relative to the handle — the
    /// parent is never looked up by path again, so nothing that happens to its path matters.
    fn step(parent: HANDLE, name: &std::ffi::OsStr, access: u32, disposition: u32, options: u32) -> io::Result<OwnedHandle> {
        let wide: Vec<u16> = name.encode_wide().collect();
        let bytes = u16::try_from(wide.len() * 2).map_err(|_| io::Error::other("a path component too long"))?;
        let us = UNICODE_STRING { Length: bytes, MaximumLength: bytes, Buffer: wide.as_ptr() as *mut u16 };
        let oa = OBJECT_ATTRIBUTES {
            Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: parent,
            ObjectName: &us,
            Attributes: OBJ_CASE_INSENSITIVE,
            ..Default::default()
        };
        // SAFETY: zeroed is a valid IO_STATUS_BLOCK.
        let mut iosb: IO_STATUS_BLOCK = unsafe { std::mem::zeroed() };
        let mut h: HANDLE = std::ptr::null_mut();
        // SAFETY: every pointer is to a live local; `wide` outlives the call.
        let status = unsafe {
            NtCreateFile(
                &mut h,
                access | SYNCHRONIZE,
                &oa,
                &mut iosb,
                std::ptr::null(),
                FILE_ATTRIBUTE_NORMAL,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                disposition,
                options | FILE_OPEN_REPARSE_POINT | FILE_SYNCHRONOUS_IO_NONALERT,
                std::ptr::null(),
                0,
            )
        };
        if status < 0 {
            // SAFETY: a pure conversion.
            let code = unsafe { RtlNtStatusToDosError(status) };
            return Err(io::Error::from_raw_os_error(code as i32));
        }
        // SAFETY: a fresh handle, owned by nothing else.
        Ok(unsafe { OwnedHandle::from_raw_handle(h as _) })
    }

    fn is_reparse(h: &OwnedHandle) -> io::Result<bool> {
        // A borrowed File view for the metadata query; ManuallyDrop so the handle is not closed.
        // SAFETY: the handle is valid for the duration of the call.
        let f = std::mem::ManuallyDrop::new(unsafe { File::from_raw_handle(h.as_raw_handle()) });
        Ok(f.metadata()?.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0)
    }

    pub(super) fn open_nofollow(real: &Path, access: Access) -> Outcome<Opened> {
        // The volume (or share) root, opened by path: `C:\` or `\\?\UNC\server\share\` cannot be a
        // link. Everything below it is opened relative to a handle.
        let mut top = PathBuf::new();
        let mut names = Vec::new();
        for c in real.components() {
            match c {
                Component::Prefix(p) => top.push(p.as_os_str()),
                Component::RootDir => top.push(std::path::MAIN_SEPARATOR_STR),
                Component::Normal(n) => names.push(n.to_owned()),
                _ => return Outcome::Escaped(SWAPPED),
            }
        }
        let mut cur: OwnedHandle = match std::fs::OpenOptions::new()
            .access_mode(FILE_LIST_DIRECTORY | FILE_TRAVERSE | FILE_READ_ATTRIBUTES | SYNCHRONIZE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
            .open(&top)
        {
            Ok(f) => f.into(),
            Err(e) => return Outcome::Io(e),
        };
        let Some((last, dirs)) = names.split_last() else {
            return if access == Access::List {
                Outcome::Done(Opened::Dir(DirHandle(cur)))
            } else {
                Outcome::Io(not_regular())
            };
        };
        for d in dirs {
            cur = match step(cur.as_raw_handle() as HANDLE, d, FILE_TRAVERSE | FILE_READ_ATTRIBUTES, FILE_OPEN, FILE_DIRECTORY_FILE) {
                Ok(h) => h,
                Err(e) => return Outcome::Io(e),
            };
            // FILE_OPEN_REPARSE_POINT opened the link ITSELF rather than following it; a directory
            // on the canonical path that is a link now was not one when the path was checked.
            match is_reparse(&cur) {
                Ok(false) => {}
                Ok(true) => return Outcome::Escaped(SWAPPED),
                Err(e) => return Outcome::Io(e),
            }
        }
        let parent = cur.as_raw_handle() as HANDLE;
        let opened = match access {
            Access::Read => step(parent, last, FILE_GENERIC_READ, FILE_OPEN, FILE_NON_DIRECTORY_FILE),
            // Opened, not truncated: truncation waits until the file is known to be a regular one.
            Access::Write => step(parent, last, FILE_GENERIC_WRITE | FILE_READ_ATTRIBUTES, FILE_OPEN_IF, FILE_NON_DIRECTORY_FILE),
            Access::Append => step(
                parent,
                last,
                FILE_APPEND_DATA | FILE_WRITE_ATTRIBUTES | FILE_READ_ATTRIBUTES,
                FILE_OPEN_IF,
                FILE_NON_DIRECTORY_FILE,
            ),
            Access::List => step(parent, last, FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES, FILE_OPEN, FILE_DIRECTORY_FILE),
        };
        let h = match opened {
            Ok(h) => h,
            Err(e) => return Outcome::Io(e),
        };
        match is_reparse(&h) {
            Ok(false) => {}
            Ok(true) => return Outcome::Escaped(SWAPPED),
            Err(e) => return Outcome::Io(e),
        }
        if access == Access::List {
            return Outcome::Done(Opened::Dir(DirHandle(h)));
        }
        let f = File::from(h);
        match f.metadata() {
            Ok(m) if m.is_file() => Outcome::Done(Opened::File(f)),
            Ok(_) => Outcome::Io(not_regular()),
            Err(e) => Outcome::Io(e),
        }
    }

    pub(super) fn entries(d: DirHandle) -> io::Result<Vec<String>> {
        // FILE_FULL_DIR_INFO records, packed; 8-byte aligned, as the records are.
        let mut buf = vec![0u64; 8192];
        let mut out = Vec::new();
        loop {
            // SAFETY: the buffer is live and its size is passed in bytes.
            let ok = unsafe {
                GetFileInformationByHandleEx(
                    d.0.as_raw_handle() as HANDLE,
                    FileFullDirectoryInfo,
                    buf.as_mut_ptr().cast(),
                    (buf.len() * 8) as u32,
                )
            };
            if ok == 0 {
                let e = io::Error::last_os_error();
                if e.raw_os_error() == Some(ERROR_NO_MORE_FILES as i32) {
                    break;
                }
                return Err(e);
            }
            let base = buf.as_ptr() as *const u8;
            let mut off = 0usize;
            loop {
                // SAFETY: `off` is an offset the previous record gave, inside the filled buffer.
                let rec = unsafe { &*(base.add(off) as *const FILE_FULL_DIR_INFO) };
                let len = rec.FileNameLength as usize / 2;
                // SAFETY: the name is `FileNameLength` bytes long, starting at `FileName`.
                let name = unsafe { std::slice::from_raw_parts(rec.FileName.as_ptr(), len) };
                let name = String::from_utf16_lossy(name);
                if name != "." && name != ".." {
                    out.push(name);
                }
                if rec.NextEntryOffset == 0 {
                    break;
                }
                off += rec.NextEntryOffset as usize;
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn base(tag: &str) -> PathBuf {
        let b = std::env::temp_dir().join(format!(
            "delulu-beneath-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(b.join("scope")).unwrap();
        fs::create_dir_all(b.join("outside")).unwrap();
        b
    }

    /// A directory link any user can make: a symlink on Unix, a junction on Windows (which needs no
    /// privilege, unlike a symlink there).
    fn dir_link(target: &Path, link: &Path) -> bool {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, link).is_ok()
        }
        #[cfg(windows)]
        {
            std::process::Command::new("cmd")
                .args(["/C", "mklink", "/J"])
                .arg(link)
                .arg(target)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
    }

    /// The ordinary operations, unchanged: read, write (truncating), append, list without `.`/`..`.
    #[test]
    fn a_plain_file_inside_the_grant_reads_writes_appends_and_lists() {
        let b = base("plain");
        let scope = b.join("scope");
        assert!(matches!(write_text(&scope, &scope.join("a.txt"), "one two", false), Outcome::Done(())));
        assert!(matches!(write_text(&scope, &scope.join("a.txt"), "three", false), Outcome::Done(())));
        assert!(matches!(write_text(&scope, &scope.join("a.txt"), "+four", true), Outcome::Done(())));
        match read_text(&scope, &scope.join("a.txt")) {
            Outcome::Done(s) => assert_eq!(s, "three+four", "write truncates, append appends"),
            o => panic!("{o:?}"),
        }
        fs::create_dir_all(scope.join("sub")).unwrap();
        match list_dir(&scope, &scope) {
            Outcome::Done(mut v) => {
                v.sort();
                assert_eq!(v, ["a.txt", "sub"]);
            }
            o => panic!("{o:?}"),
        }
        assert!(matches!(read_text(&scope, &scope.join("missing.txt")), Outcome::Io(e) if e.kind() == io::ErrorKind::NotFound));
        assert!(matches!(write_text(&scope, &scope.join("nodir").join("x.txt"), "x", false), Outcome::Io(_)));
        assert!(matches!(read_text(&scope, &scope.join("sub")), Outcome::Io(_)), "a directory is not read as text");
        assert!(matches!(read_text(&scope, &b.join("outside").join("x")), Outcome::Escaped(OUTSIDE)));
        let _ = fs::remove_dir_all(&b);
    }

    /// FS-RACE-1, deterministically: the walk is handed a canonical path with a link on it — the
    /// state a racer leaves between the check and the open — and must refuse it rather than follow
    /// it, creating nothing on the far side.
    #[test]
    fn a_link_met_on_the_walk_is_refused_and_nothing_is_created_beyond_it() {
        let b = base("walk");
        let scope = b.join("scope");
        let outside = b.join("outside");
        if !dir_link(&outside, &scope.join("d")) {
            eprintln!("skipped: no directory link could be made here");
            return;
        }
        let canon_scope = fs::canonicalize(&scope).unwrap();
        let real = canon_scope.join("d").join("f.txt");
        for access in [Access::Write, Access::Append, Access::Read] {
            match imp::open_nofollow(&real, access) {
                Outcome::Escaped(w) => assert_eq!(w, SWAPPED),
                Outcome::Done(_) => panic!("{access:?}: followed a link that appeared after the check"),
                Outcome::Io(e) => panic!("{access:?}: refused for the wrong reason: {e}"),
            }
        }
        assert!(matches!(imp::open_nofollow(&canon_scope.join("d"), Access::List), Outcome::Escaped(SWAPPED)));
        assert!(!outside.join("f.txt").exists(), "nothing was created through the link");
        // The same link, met by the public door: refused at the CHECK, because it resolves outside.
        assert!(matches!(write_text(&scope, &scope.join("d").join("f.txt"), "x", false), Outcome::Escaped(OUTSIDE)));
        let _ = fs::remove_dir_all(&b);
    }

    /// A link that stays INSIDE the grant keeps working: the check resolves it, and the walk opens
    /// what it resolves to.
    #[test]
    fn a_link_that_stays_inside_the_grant_still_works() {
        let b = base("inner");
        let scope = b.join("scope");
        fs::create_dir_all(scope.join("real")).unwrap();
        fs::write(scope.join("real").join("t.txt"), "inside").unwrap();
        if !dir_link(&scope.join("real"), &scope.join("alias")) {
            eprintln!("skipped: no directory link could be made here");
            return;
        }
        match read_text(&scope, &scope.join("alias").join("t.txt")) {
            Outcome::Done(s) => assert_eq!(s, "inside"),
            o => panic!("{o:?}"),
        }
        assert!(matches!(write_text(&scope, &scope.join("alias").join("u.txt"), "w", false), Outcome::Done(())));
        assert_eq!(fs::read_to_string(scope.join("real").join("u.txt")).unwrap(), "w");
        let _ = fs::remove_dir_all(&b);
    }

    /// The FIFO finding (same agent pass): reading a FIFO placed in the grant hung for ever. It is
    /// now refused as not a regular file, at once — the open is non-blocking, so even the refusal
    /// cannot hang.
    #[cfg(unix)]
    #[test]
    fn a_fifo_in_the_grant_is_refused_at_once_for_read_and_write() {
        let b = base("fifo");
        let scope = b.join("scope");
        let fifo = scope.join("pipe");
        let c = std::ffi::CString::new(fifo.to_str().unwrap()).unwrap();
        // SAFETY: a valid path string.
        assert_eq!(unsafe { libc::mkfifo(c.as_ptr(), 0o600) }, 0);
        let t = std::time::Instant::now();
        assert!(matches!(read_text(&scope, &fifo), Outcome::Io(_)));
        assert!(matches!(write_text(&scope, &fifo, "x", false), Outcome::Io(_)));
        assert!(t.elapsed() < std::time::Duration::from_secs(5), "refused without blocking");
        let _ = fs::remove_dir_all(&b);
    }

    /// FS-RACE-1 as it was found: one thread flips `scope/d` between a real directory and a link to
    /// a directory OUTSIDE the grant, as fast as renames go, while this one writes `d/f.txt` and reads
    /// `d/secret.txt` through the public door, over and over. Nothing may land outside, and the
    /// outside secret may never be read. Against the old check-then-`std::fs::write` the outside file
    /// was written (the mutant run recorded in `V2_LOG.md`).
    #[test]
    fn a_directory_flipped_to_a_link_mid_operation_never_lets_a_write_or_read_through() {
        let b = base("race");
        let scope = b.join("scope");
        let outside = b.join("outside");
        fs::create_dir_all(scope.join("d")).unwrap();
        fs::write(scope.join("d").join("secret.txt"), "inside").unwrap();
        fs::write(outside.join("secret.txt"), "OUTSIDE-SECRET").unwrap();
        if !dir_link(&outside, &scope.join("j")) {
            eprintln!("skipped: no directory link could be made here");
            return;
        }
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flipper = {
            let (stop, scope) = (stop.clone(), scope.clone());
            std::thread::spawn(move || {
                let (d, real, j) = (scope.join("d"), scope.join("d_real"), scope.join("j"));
                let mut flips = 0u64;
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    let _ = fs::rename(&d, &real);
                    let _ = fs::rename(&j, &d);
                    let _ = fs::rename(&d, &j);
                    let _ = fs::rename(&real, &d);
                    flips += 1;
                }
                flips
            })
        };
        let (mut wrote, mut refused, mut leaked) = (0u32, 0u32, 0u32);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while std::time::Instant::now() < deadline {
            match write_text(&scope, &scope.join("d").join("f.txt"), "PWNED", false) {
                Outcome::Done(()) => wrote += 1,
                Outcome::Escaped(_) => refused += 1,
                Outcome::Io(_) => {}
            }
            if let Outcome::Done(s) = read_text(&scope, &scope.join("d").join("secret.txt")) {
                if s == "OUTSIDE-SECRET" {
                    leaked += 1;
                }
            }
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let flips = flipper.join().unwrap();
        eprintln!("race: {flips} flips, {wrote} writes inside, {refused} refused");
        assert!(!outside.join("f.txt").exists(), "a write landed OUTSIDE the grant ({wrote} inside, {refused} refused)");
        assert_eq!(leaked, 0, "the outside secret was read through the grant");
        let _ = fs::remove_dir_all(&b);
    }
}
