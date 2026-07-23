use adw::dict::entry::Entry;
use adw::dict::file::load_dict;
use std::path::Path;

fn fixture_path(name: &str) -> String {
    let dir = env!("CARGO_MANIFEST_DIR");
    format!("{dir}/tests/fixtures/{name}")
}

#[test]
fn test_load_minimal_dict() {
    // Given: a minimal dictionary file
    let path = fixture_path("header_minimal.yaml");
    // When: loading it
    let dict = load_dict(Path::new(&path)).unwrap();
    // Then: header is parsed and entries are empty
    assert_eq!(dict.header.name, "minimal_dict");
    assert!(dict.entries.is_empty());
}

#[test]
fn test_load_full_dict() {
    // Given: a full dictionary with header and entries
    let path = fixture_path("header_full.yaml");
    // When: loading it
    let dict = load_dict(Path::new(&path)).unwrap();
    // Then: header and entries are parsed
    assert_eq!(dict.header.name, "test_dict");
    assert_eq!(dict.entries.len(), 4);
    assert_eq!(dict.entries[0].word, "工");
    assert_eq!(dict.entries[0].code, "a");
}

#[test]
fn test_load_extra_with_groups() {
    // Given: an extra dictionary with group headers
    let path = fixture_path("extra_with_groups.yaml");
    // When: loading it
    let dict = load_dict(Path::new(&path)).unwrap();
    // Then: group headers and entries are preserved
    assert_eq!(dict.entries.len(), 6);
    // Group headers are kept as entries with is_group() == true
    let groups: Vec<_> = dict.entries.iter().filter(|e| e.is_group()).collect();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].word, "## 汽车");
    assert_eq!(groups[1].word, "## 歌手");
}

#[test]
fn test_load_nonexistent_file() {
    // Given: a path that doesn't exist
    let path = fixture_path("nonexistent.yaml");
    // When: loading it
    let result = load_dict(Path::new(&path));
    // Then: returns an error
    assert!(result.is_err());
}

#[test]
fn test_add_entry_to_dict() {
    // Given: a loaded dictionary
    let path = fixture_path("empty_dict.yaml");
    let mut dict = load_dict(Path::new(&path)).unwrap();
    // When: adding a new entry
    let entry = Entry::new("测试".into(), "test".into(), 10);
    let added = dict.add_entry(entry);
    // Then: entry is added
    assert!(added);
    assert_eq!(dict.entries.len(), 1);
}

#[test]
fn test_add_duplicate_entry() {
    // Given: a loaded dictionary
    let path = fixture_path("header_full.yaml");
    let mut dict = load_dict(Path::new(&path)).unwrap();
    let original_len = dict.entries.len();
    // When: adding an entry that already exists
    let entry = Entry::new("工".into(), "a".into(), 20);
    let added = dict.add_entry(entry);
    // Then: duplicate is not added
    assert!(!added);
    assert_eq!(dict.entries.len(), original_len);
}

#[test]
fn test_save_and_reload_is_idempotent() {
    use std::fs;
    // Given: a loaded dictionary
    let path = fixture_path("extra_with_groups.yaml");
    let dict = load_dict(Path::new(&path)).unwrap();
    // When: saving to a temp file
    let tmp = fixture_path("_tmp_idempotent.yaml");
    dict.save(Path::new(&tmp)).unwrap();
    // Then: reloading gives the same data
    let reloaded = load_dict(Path::new(&tmp)).unwrap();
    assert_eq!(reloaded.entries.len(), dict.entries.len());
    for (a, b) in dict.entries.iter().zip(reloaded.entries.iter()) {
        assert_eq!(a.word, b.word);
        assert_eq!(a.code, b.code);
        assert_eq!(a.weight, b.weight);
    }
    let _ = fs::remove_file(&tmp);
}
