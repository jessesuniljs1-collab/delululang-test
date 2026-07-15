/// Identifies a file registered in a [`crate::SourceMap`].
pub type FileId = u32;

/// A byte-offset region of one source file. Byte offsets are the machine truth
/// (Stage-1 spec §10.2); line/column are derived at render time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Span {
    pub file: FileId,
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(file: FileId, start: u32, end: u32) -> Self {
        Span { file, start, end }
    }

    /// The smallest span covering both `self` and `other` (same file assumed).
    pub fn to(self, other: Span) -> Span {
        Span {
            file: self.file,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// A zero-width span at this span's start (used for pure insertions).
    pub fn start_point(self) -> Span {
        Span { file: self.file, start: self.start, end: self.start }
    }
}
