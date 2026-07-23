use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::dict::entry::Entry;
use crate::dict::header::DictHeader;

#[derive(Debug, Clone)]
pub struct DictFile {
    pub header: DictHeader,
    pub entries: Vec<Entry>,
}

impl DictFile {
    pub fn add_entry(&mut self, entry: Entry) -> bool {
        if self.entries.iter().any(|e| e.word == entry.word && e.code == entry.code) {
            return false;
        }
        self.entries.push(entry);
        true
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let mut file = fs::File::create(path)?;
        let header_lines = self.header.to_lines().to_vec();
        for line in &header_lines {
            writeln!(file, "{line}")?;
        }
        for entry in &self.entries {
            writeln!(file, "{}", entry.format())?;
        }
        Ok(())
    }
}

pub fn load_dict(path: &Path) -> io::Result<DictFile> {
    let content = fs::read_to_string(path)?;
    let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    let header = DictHeader::parse(&lines)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "缺少 YAML header 结束符 ..."))?;

    let header_end = header.header_end;
    let body_lines = &lines[header_end + 1..];

    let entries: Vec<Entry> = body_lines.iter()
        .filter_map(|l| Entry::parse(l))
        .collect();

    Ok(DictFile { header, entries })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_minimal() {
        let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/header_minimal.yaml"));
        let dict = load_dict(path).unwrap();
        assert_eq!(dict.header.name, "minimal_dict");
        assert!(dict.entries.is_empty());
    }

    #[test]
    fn load_full() {
        let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/header_full.yaml"));
        let dict = load_dict(path).unwrap();
        assert_eq!(dict.entries.len(), 4);
    }

    #[test]
    fn add_duplicate() {
        let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/header_full.yaml"));
        let mut dict = load_dict(path).unwrap();
        let e = Entry::new("工".into(), "a".into(), 20);
        assert!(!dict.add_entry(e));
    }

    #[test]
    fn save_and_reload() {
        let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/extra_with_groups.yaml"));
        let dict = load_dict(path).unwrap();
        let tmp = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/_tmp_test.yaml"));
        dict.save(tmp).unwrap();
        let reloaded = load_dict(tmp).unwrap();
        assert_eq!(reloaded.entries.len(), dict.entries.len());
        let _ = std::fs::remove_file(tmp);
    }
}
