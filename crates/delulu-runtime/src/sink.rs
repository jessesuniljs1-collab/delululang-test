//! The effect seam (PS-A-01): the ONE trait every capability operation passes through.
//!
//! Today the interpreter calls the primitive table directly, so "where does an effect actually
//! happen" is spread across call sites. PS-A's guest runs the interpreter with no OS authority of
//! its own and sends each operation to the host, which performs it under the checks it already
//! makes. That is only safe if there is exactly one place an effect can leave the interpreter, and
//! this trait is it.
//!
//! [`LocalSink`] is today's path and must stay byte-identical: the core-invariance snapshot does
//! not move, and `delulu-fuzz` in local mode is unchanged. `ChannelSink` (PS-A-02) will serialize
//! the same call and block on the host's reply.
//!
//! The seam carries capability METHODS, the operations that perform effects. Minting a capability
//! (`root.fs_read(…)`) and declassifying a secret keep their own paths for now: in guest mode they
//! become host-minted handles, which PS-A-03 adds with the hello.

use delulu_diag::Span;

use crate::value::{CapVal, Fault, Value};

/// Where an effect goes when the interpreter performs it.
///
/// Implementations must not widen authority: the scope check in `prim::call_cap_method` is the
/// host-side half of invariant 7 and runs on every use, wherever the sink sends the call.
pub trait EffectSink {
    /// Perform one capability operation, or refuse it exactly as the primitive table would.
    fn cap_method(&self, cap: &CapVal, method: &str, args: &[Value], span: Span) -> Result<Value, Fault>;

    /// A short name for the run report and the audit record (`inproc` for the local path).
    fn backend(&self) -> &'static str {
        "inproc"
    }
}

/// The in-process path: call the primitive table, exactly as the interpreter did before the seam.
#[derive(Debug, Default, Clone, Copy)]
pub struct LocalSink;

impl EffectSink for LocalSink {
    fn cap_method(&self, cap: &CapVal, method: &str, args: &[Value], span: Span) -> Result<Value, Fault> {
        crate::prim::call_cap_method(cap, method, args, span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The seam does not become a second authority: a scope the primitive table refuses is refused
    /// through the sink too, with the same code. (The guest-mode half arrives with PS-A-02.)
    #[test]
    fn the_local_sink_refuses_what_the_primitive_table_refuses() {
        let cap = CapVal {
            kind: delulu_check::ResourceKind::FsRead,
            scope: crate::value::CapScope::Fs { root: std::path::PathBuf::from("."), write: false },
        };
        let span = Span::new(0, 0, 0);
        let direct = crate::prim::call_cap_method(&cap, "no_such_method", &[], span);
        let through = LocalSink.cap_method(&cap, "no_such_method", &[], span);
        assert_eq!(format!("{direct:?}"), format!("{through:?}"), "the seam must not change the answer");
        assert_eq!(LocalSink.backend(), "inproc");
    }
}
