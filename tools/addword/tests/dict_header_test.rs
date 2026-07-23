use adw::dict::header::DictHeader;

fn full_header_lines() -> Vec<String> {
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

fn minimal_header_lines() -> Vec<String> {
    r#"---
name: minimal_dict
version: "1.0"
...
"#.lines().map(|l| l.to_string()).collect()
}

fn extra_dict_lines() -> Vec<String> {
    r#"---
name: wubi86_jidian_extra
version: "2019-09-06"
...
"#.lines().map(|l| l.to_string()).collect()
}

#[test]
fn test_find_header_end_index() {
    // Given: a full header with --- at start and ... at end
    let lines = full_header_lines();
    // When: parsing the header
    let header = DictHeader::parse(&lines).unwrap();
    // Then: the header end index points to the ... line
    let raw = header.to_lines();
    assert_eq!(raw[header.header_end], "...");
}

#[test]
fn test_parse_header_name() {
    // Given: a header with name field
    let lines = full_header_lines();
    // When: parsing
    let header = DictHeader::parse(&lines).unwrap();
    // Then: name is extracted
    assert_eq!(header.name, "test_dict");
}

#[test]
fn test_list_import_tables() {
    // Given: a header with import_tables
    let lines = full_header_lines();
    // When: parsing
    let header = DictHeader::parse(&lines).unwrap();
    // Then: import_tables list is populated
    assert!(header.has_import("wubi86_jidian_extra"));
    assert!(!header.has_import("nonexistent"));
}

#[test]
fn test_minimal_header() {
    // Given: a minimal header with only name and version
    let lines = minimal_header_lines();
    // When: parsing
    let header = DictHeader::parse(&lines).unwrap();
    // Then: basic fields are extracted
    assert_eq!(header.name, "minimal_dict");
    assert_eq!(header.version, Some("1.0".to_string()));
    assert_eq!(header.header_end, 3);
}

#[test]
fn test_add_import_table() {
    // Given: a header without import_tables
    let lines = extra_dict_lines();
    let mut header = DictHeader::parse(&lines).unwrap();
    // When: adding an import
    let added = header.add_import("wubi86_jidian_extra");
    // Then: import was added
    assert!(added);
    assert!(header.has_import("wubi86_jidian_extra"));
}

#[test]
fn test_add_existing_import_is_noop() {
    // Given: a header that already has the import
    let lines = full_header_lines();
    let mut header = DictHeader::parse(&lines).unwrap();
    // When: adding the same import again
    let added = header.add_import("wubi86_jidian_extra");
    // Then: no change
    assert!(!added);
}

#[test]
fn test_render_header_to_lines() {
    // Given: a parsed header
    let lines = full_header_lines();
    let header = DictHeader::parse(&lines).unwrap();
    // When: rendering back to lines
    let rendered = header.to_lines();
    // Then: the result includes the full original header
    assert_eq!(rendered[0], "---");
    assert_eq!(rendered[header.header_end], "...");
    // Rendered lines must match original header section
    assert_eq!(rendered.len(), header.header_end + 1);
}
