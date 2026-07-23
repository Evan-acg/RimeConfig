use adw::encoder::char_map::CharMap;
use adw::encoder::wubi86::Wubi86Encoder;

fn make_char_map() -> CharMap {
    // Simulated 五笔86 单字编码
    let lines = vec![
        "一\tg\t11".to_string(),
        "地\tf\t12".to_string(),
        "在\td\t13".to_string(),
        "要\ts\t14".to_string(),
        "工\ta\t15".to_string(),
        "上\th\t21".to_string(),
        "是\tj\t22".to_string(),
        "中\tk\t23".to_string(),
        "国\tl\t24".to_string(),
        "同\tm\t25".to_string(),
        "人\tw\t34".to_string(),
        "们\tw\t34".to_string(),
        "好\tvb\t100".to_string(),
        "你\twq\t100".to_string(),
        "他\twb\t100".to_string(),
        "我\ttrnt\t100".to_string(),
        "世\tan\t100".to_string(),
        "界\tlwj\t100".to_string(),
        "大\tdd\t100".to_string(),
        "会\twfc\t100".to_string(),
    ];
    let entries: Vec<_> = lines.iter()
        .filter_map(|l| adw::dict::entry::Entry::parse(l))
        .collect();
    CharMap::from_entries(&entries)
}

fn encode(word: &str, map: &CharMap) -> adw::encoder::wubi86::EncodeResult {
    Wubi86Encoder.encode(word, map)
}

#[test]
fn test_two_char_word() {
    // Given: a 2-character word with known codes
    let map = make_char_map();
    // When: encoding
    let result = encode("我们", &map);
    // Then: AaAbBaBb — take2(我) + take2(们)
    // 我=trnt → tr, 们=w → w
    assert_eq!(result.code, Some("trw".to_string()));
    assert!(result.missing_chars.is_empty());
}

#[test]
fn test_three_char_word() {
    // Given: a 3-character word
    let map = make_char_map();
    // When: encoding
    let result = encode("工人好", &map);
    // Aa=工→a, Ba=人→w, Ca=好→v, Cb=好→vb
    assert_eq!(result.code, Some("awvb".to_string()));
    assert!(result.missing_chars.is_empty());
}

#[test]
fn test_four_char_word() {
    // Given: a 4-character word
    let map = make_char_map();
    // When: encoding
    let result = encode("工地上人", &map);
    // Then: AaBaCaZa — take1(工)=a, take1(地)=f, take1(上)=h, take1(人)=w
    assert_eq!(result.code, Some("afhw".to_string()));
    assert!(result.missing_chars.is_empty());
}

#[test]
fn test_long_word() {
    // Given: a word with >4 characters
    let map = make_char_map();
    // When: encoding
    let result = encode("世界大国你好", &map);
    // Then: AaBaCaZa — take1(世)+take1(界)+take1(大)+take1(好)
    // 世=a, 界=l, 大=d, 好=v
    assert_eq!(result.code, Some("aldv".to_string()));
    assert!(result.missing_chars.is_empty());
}

#[test]
fn test_less_than_two_chars() {
    // Given: a single character
    let map = make_char_map();
    // When: encoding
    let result = encode("工", &map);
    // Then: returns None (minimum 2 chars), missing contains the char
    assert!(result.code.is_none());
    assert_eq!(result.missing_chars, vec!['工']);
}

#[test]
fn test_missing_char_in_map() {
    // Given: a word containing a char not in the map
    let map = make_char_map();
    // When: encoding
    let result = encode("你好啊", &map);
    // Then: returns None (啊 is not in map)
    assert!(result.code.is_none());
    assert_eq!(result.missing_chars, vec!['啊']);
}

#[test]
fn test_empty_string() {
    // Given: an empty string
    let map = make_char_map();
    // When: encoding
    let result = encode("", &map);
    // Then: returns None, missing is empty
    assert!(result.code.is_none());
    assert!(result.missing_chars.is_empty());
}
