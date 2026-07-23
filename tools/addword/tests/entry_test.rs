use adw::dict::entry::Entry;

#[test]
fn test_parse_full_entry() {
    // Given: a tab-separated line with word, code, and weight
    let line = "工\ta\t20";
    // When: parsing the line
    let entry = Entry::parse(line).unwrap();
    // Then: all fields are correctly extracted
    assert_eq!(entry.word, "工");
    assert_eq!(entry.code, "a");
    assert_eq!(entry.weight, 20);
    assert_eq!(entry.group, None);
}

#[test]
fn test_parse_entry_without_weight() {
    // Given: a line with only word and code (no weight)
    let line = "朗逸\tyvqk";
    // When: parsing the line
    let entry = Entry::parse(line).unwrap();
    // Then: weight defaults to 0
    assert_eq!(entry.word, "朗逸");
    assert_eq!(entry.code, "yvqk");
    assert_eq!(entry.weight, 0);
}

#[test]
fn test_parse_group_header() {
    // Given: a group header line starting with ##
    let line = "## 汽车";
    // When: parsing the line
    let entry = Entry::parse(line).unwrap();
    // Then: it identifies as a group header
    assert_eq!(entry.word, "## 汽车");
    assert!(entry.is_group());
}

#[test]
fn test_parse_comment_ignored() {
    // Given: a comment line starting with #
    let line = "# 这是注释";
    // When: parsing
    let entry = Entry::parse(line);
    // Then: returns None
    assert!(entry.is_none());
}

#[test]
fn test_parse_empty_line() {
    // Given: an empty line
    let line = "";
    // When: parsing
    let entry = Entry::parse(line);
    // Then: returns None
    assert!(entry.is_none());
}

#[test]
fn test_format_full_entry() {
    // Given: an entry with word, code, and weight
    let entry = Entry::new("工".into(), "a".into(), 20);
    // When: formatting
    let formatted = entry.format();
    // Then: produces tab-separated output
    assert_eq!(formatted, "工\ta\t20");
}

#[test]
fn test_format_entry_without_weight() {
    // Given: an entry with zero weight
    let entry = Entry::new("工".into(), "a".into(), 0);
    // When: formatting
    let formatted = entry.format();
    // Then: produces word + code (weight omitted when 0)
    assert_eq!(formatted, "工\ta");
}

#[test]
fn test_format_group_header() {
    // Given: a group entry
    let entry = Entry::group("汽车".into());
    // When: formatting
    let formatted = entry.format();
    // Then: produces ## prefix
    assert_eq!(formatted, "## 汽车");
}

#[test]
fn test_entry_parse_roundtrip() {
    // Given: a valid entry line
    let original = "你好\twqvb\t30";
    // When: parsing and re-formatting
    let entry = Entry::parse(original).unwrap();
    let formatted = entry.format();
    // Then: the result should match the original
    assert_eq!(formatted, original);
}
