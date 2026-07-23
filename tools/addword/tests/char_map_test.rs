use adw::dict::entry::Entry;
use adw::encoder::char_map::CharMap;

fn make_entry(line: &str) -> Entry {
    Entry::parse(line).unwrap()
}

#[test]
fn test_build_from_entries() {
    // Given: a list of single-character entries
    let entries = vec![
        make_entry("工\ta\t20"),
        make_entry("戈\ta\t10"),
        make_entry("式\taa\t20"),
    ];
    // When: building a CharMap
    let map = CharMap::from_entries(&entries);
    // Then: each character maps to its code
    assert_eq!(map.get('工'), Some("a"));
    assert_eq!(map.get('戈'), Some("a"));
    assert_eq!(map.get('式'), Some("aa"));
}

#[test]
fn test_skips_multi_char_entries() {
    // Given: entries with multi-character words
    let entries = vec![
        make_entry("你好\twqvb\t20"),
        make_entry("工\ta\t20"),
    ];
    // When: building a CharMap
    let map = CharMap::from_entries(&entries);
    // Then: only single-character words are included
    assert_eq!(map.get('你'), None);
    assert_eq!(map.get('工'), Some("a"));
}

#[test]
fn test_picks_highest_weight() {
    // Given: the same character with multiple codes
    let entries = vec![
        make_entry("行\tq\t10"),
        make_entry("行\ttf\t30"),
        make_entry("行\thh\t20"),
    ];
    // When: building a CharMap
    let map = CharMap::from_entries(&entries);
    // Then: the code with highest weight wins
    assert_eq!(map.get('行'), Some("tf"));
}

#[test]
fn test_skips_comments_and_groups() {
    // Given: entries including comments and group headers
    let entries = vec![
        make_entry("工\ta\t20"),
    ];
    // When: building a CharMap
    let map = CharMap::from_entries(&entries);
    // Then: only valid single-char entries are included
    assert_eq!(map.get('工'), Some("a"));
    // Group headers and comments are filtered out during parse
    assert!(Entry::parse("## 汽车").map_or(true, |e| e.is_group()));
    assert!(Entry::parse("# 注释").is_none());
}

#[test]
fn test_empty_entries() {
    // Given: an empty list
    let entries: Vec<Entry> = vec![];
    // When: building a CharMap
    let map = CharMap::from_entries(&entries);
    // Then: the map is empty
    assert!(map.is_empty());
}

#[test]
fn test_build_from_dict_lines() {
    // Given: raw dictionary lines (after header)
    let lines = vec![
        "工\ta\t20".to_string(),
        "戈\ta\t10".to_string(),
        "式\taa\t20".to_string(),
    ];
    // When: building from lines
    let map = CharMap::from_lines(&lines);
    // Then: characters are mapped correctly
    assert_eq!(map.get('工'), Some("a"));
}
