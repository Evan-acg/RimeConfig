#[derive(Debug, Clone)]
pub struct DictHeader {
    pub header_end: usize,
    pub name: String,
    pub version: Option<String>,
    pub sort: Option<String>,
    pub import_tables: Vec<String>,
    /// All header lines preserved verbatim (including --- and ...)
    raw: Vec<String>,
    /// Indexes of import_tables list item lines within `raw`
    import_line_indices: Vec<usize>,
    /// Index of the `import_tables:` key line, if present
    import_key_index: Option<usize>,
}

impl DictHeader {
    pub fn parse(lines: &[String]) -> Option<Self> {
        let header_end = find_header_end(lines)?;
        let raw: Vec<String> = lines[..=header_end].to_vec();

        let name = extract_field(&raw, "name").unwrap_or_default();
        let version = extract_field(&raw, "version");
        let sort = extract_field(&raw, "sort");

        // Find import_tables section
        let mut import_tables = Vec::new();
        let mut import_line_indices = Vec::new();
        let mut import_key_index = None;

        for (i, line) in raw.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("import_tables:") || trimmed.starts_with("# import_tables:") {
                import_key_index = Some(i);
                // Collect subsequent lines that are list items
                for j in (i + 1)..raw.len() {
                    let t = raw[j].trim_start();
                    if t.starts_with("- ") {
                        let val = t.trim_start_matches("- ").trim();
                        // Extract just the table name (before any comment)
                        let name = val.split('#').next().unwrap_or(val).trim().to_string();
                        if !name.is_empty() {
                            import_tables.push(name);
                        }
                        import_line_indices.push(j);
                    } else if t.is_empty() || t.starts_with('#') {
                        continue;
                    } else {
                        break;
                    }
                }
                break;
            }
        }

        Some(DictHeader {
            header_end,
            name,
            version,
            sort,
            import_tables,
            raw,
            import_line_indices,
            import_key_index,
        })
    }

    pub fn has_import(&self, name: &str) -> bool {
        let target = format!("- {name}");
        self.import_tables.iter().any(|t| *t == name)
            || self.raw.iter().any(|l| l.trim_start().starts_with(&target))
    }

    pub fn add_import(&mut self, name: &str) -> bool {
        if self.has_import(name) {
            return false;
        }

        let target = format!("  - {name}");

        if let Some(key_idx) = self.import_key_index {
            // Insert after the last import line, or after the key line if no imports
            let insert_at = self.import_line_indices.last().copied()
                .map(|i| i + 1)
                .unwrap_or(key_idx + 1);
            self.raw.insert(insert_at, target.clone());
            self.header_end += 1;
            self.import_tables.push(name.to_string());
            self.import_line_indices.push(insert_at);
        } else {
            // Find the right place to insert import_tables block
            // Before columns: or encoder: or ...
            let insert_at = self.raw.iter().position(|l| {
                let t = l.trim_start();
                t.starts_with("columns:") || t.starts_with("encoder:") || t == "..."
            }).unwrap_or(self.raw.len() - 1);

            let import_block = vec![
                "import_tables:".to_string(),
                target.clone(),
            ];
            for (offset, line) in import_block.into_iter().enumerate() {
                self.raw.insert(insert_at + offset, line);
            }
            self.header_end += 2;
            self.import_tables.push(name.to_string());
            self.import_key_index = Some(insert_at);
            self.import_line_indices = vec![insert_at + 1];
        }

        true
    }

    pub fn to_lines(&self) -> &[String] {
        &self.raw
    }
}

fn find_header_end(lines: &[String]) -> Option<usize> {
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "..." && i > 0 {
            return Some(i);
        }
    }
    None
}

fn extract_field(lines: &[String], field: &str) -> Option<String> {
    let target = format!("{field}:");
    for line in lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with(&target) {
            let val = trimmed.trim_start_matches(&target).trim();
            // Remove quotes if present
            let val = val.trim_matches('"').trim_matches('\'').to_string();
            if !val.is_empty() {
                return Some(val);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_full_header() -> Vec<String> {
        r#"---
name: test_dict
version: "0.1"
sort: by_weight
import_tables:
  - wubi86_jidian_extra
columns:
  - text
  - code
  - weight
  - stem
encoder:
  exclude_patterns:
    - '^z.*$'
...
"#.lines().map(|l| l.to_string()).collect()
    }

    #[test]
    fn parse_full_header() {
        let lines = make_full_header();
        let h = DictHeader::parse(&lines).unwrap();
        assert_eq!(h.name, "test_dict");
        assert_eq!(h.version, Some("0.1".to_string()));
        assert_eq!(h.sort, Some("by_weight".to_string()));
        assert!(h.has_import("wubi86_jidian_extra"));
    }

    #[test]
    fn find_header_end_marker() {
        let lines = make_full_header();
        let h = DictHeader::parse(&lines).unwrap();
        assert_eq!(h.raw[h.header_end], "...");
    }

    #[test]
    fn add_new_import() {
        let lines = r#"---
name: test
version: "1"
...
"#.lines().map(|l| l.to_string()).collect::<Vec<_>>();
        // Need to construct via parse to get proper indices
        let mut h = DictHeader::parse(&lines).unwrap();
        let added = h.add_import("extra_dict");
        assert!(added);
        assert!(h.has_import("extra_dict"));
    }

    #[test]
    fn dont_add_duplicate_import() {
        let lines = make_full_header();
        let mut h = DictHeader::parse(&lines).unwrap();
        let added = h.add_import("wubi86_jidian_extra");
        assert!(!added);
    }
}
