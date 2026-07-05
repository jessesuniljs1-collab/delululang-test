use crate::span::FileId;

/// One registered source file: name, contents, and a precomputed line index.
pub struct SourceFile {
    pub name: String,
    pub src: String,
    /// Byte offset of the start of each line (line_starts[0] == 0).
    line_starts: Vec<u32>,
}

/// All source files of a compilation. Diagnostics carry byte spans; the map
/// converts them to 1-based line/column positions for humans and for JSON.
#[derive(Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn new() -> Self {
        SourceMap { files: Vec::new() }
    }

    pub fn add_file(&mut self, name: impl Into<String>, src: impl Into<String>) -> FileId {
        let src = src.into();
        let mut line_starts = vec![0u32];
        for (i, b) in src.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push((i + 1) as u32);
            }
        }
        self.files.push(SourceFile { name: name.into(), src, line_starts });
        (self.files.len() - 1) as FileId
    }

    pub fn file(&self, id: FileId) -> &SourceFile {
        &self.files[id as usize]
    }

    pub fn name(&self, id: FileId) -> &str {
        &self.files[id as usize].name
    }

    /// 1-based (line, column) for a byte offset. Column counts Unicode scalar
    /// values on the line, so it is stable for humans; `byte` stays the truth.
    pub fn position(&self, id: FileId, byte: u32) -> (u32, u32) {
        let f = self.file(id);
        let byte = byte.min(f.src.len() as u32);
        let line_idx = match f.line_starts.binary_search(&byte) {
            Ok(i) => i,
            Err(i) => i - 1,
        };
        let line_start = f.line_starts[line_idx] as usize;
        let col = f.src[line_start..byte as usize].chars().count() as u32 + 1;
        (line_idx as u32 + 1, col)
    }

    /// The full text of the (1-based) line, without its trailing newline.
    pub fn line_text(&self, id: FileId, line: u32) -> &str {
        let f = self.file(id);
        let idx = (line - 1) as usize;
        let start = f.line_starts[idx] as usize;
        let end = f
            .line_starts
            .get(idx + 1)
            .map(|&e| e as usize)
            .unwrap_or(f.src.len());
        f.src[start..end].trim_end_matches(['\n', '\r'])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_are_one_based_and_line_aware() {
        let mut map = SourceMap::new();
        let id = map.add_file("t.delulu", "ab\ncd\n");
        assert_eq!(map.position(id, 0), (1, 1));
        assert_eq!(map.position(id, 1), (1, 2));
        assert_eq!(map.position(id, 3), (2, 1));
        assert_eq!(map.position(id, 4), (2, 2));
        assert_eq!(map.line_text(id, 2), "cd");
    }
}
