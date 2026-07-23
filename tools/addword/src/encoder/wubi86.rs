use crate::encoder::char_map::CharMap;

fn take1(s: &str) -> String {
    s.chars().take(1).collect()
}

fn take2(s: &str) -> String {
    s.chars().take(2).collect()
}

pub struct Wubi86Encoder;

impl Wubi86Encoder {
    pub fn encode(&self, word: &str, char_map: &CharMap) -> Option<String> {
        let chars: Vec<char> = word.chars().collect();
        let n = chars.len();
        if n < 2 {
            return None;
        }

        let codes: Vec<&str> = chars
            .iter()
            .map(|c| char_map.get(*c))
            .collect::<Option<Vec<_>>>()?;

        match n {
            2 => {
                Some(format!("{}{}", take2(codes[0]), take2(codes[1])))
            }
            3 => {
                Some(format!("{}{}{}", take1(codes[0]), take1(codes[1]), take2(codes[2])))
            }
            _ => {
                Some(format!(
                    "{}{}{}{}",
                    take1(codes[0]),
                    take1(codes[1]),
                    take1(codes[2]),
                    take1(codes[n - 1])
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dict::entry::Entry;

    fn make_char_map() -> CharMap {
        let entries = vec![
            Entry::new("一".into(), "g".into(), 11),
            Entry::new("地".into(), "f".into(), 12),
            Entry::new("工".into(), "a".into(), 15),
            Entry::new("上".into(), "h".into(), 21),
            Entry::new("人".into(), "w".into(), 34),
            Entry::new("好".into(), "vb".into(), 100),
            Entry::new("我".into(), "trnt".into(), 100),
            Entry::new("世".into(), "an".into(), 100),
            Entry::new("界".into(), "lwj".into(), 100),
            Entry::new("大".into(), "dd".into(), 100),
            Entry::new("们".into(), "w".into(), 34),
        ];
        CharMap::from_entries(&entries)
    }

    #[test]
    fn encode_two_chars() {
        let map = make_char_map();
        let encoder = Wubi86Encoder;
        assert_eq!(encoder.encode("我们", &map), Some("trw".to_string()));
    }

    #[test]
    fn encode_three_chars() {
        let map = make_char_map();
        let encoder = Wubi86Encoder;
        assert_eq!(encoder.encode("工人们", &map), Some("aww".to_string()));
    }

    #[test]
    fn encode_four_chars() {
        let map = make_char_map();
        let encoder = Wubi86Encoder;
        assert_eq!(encoder.encode("工上人好", &map), Some("ahwv".to_string()));
    }

    #[test]
    fn encode_too_short() {
        let map = make_char_map();
        let encoder = Wubi86Encoder;
        assert!(encoder.encode("工", &map).is_none());
    }

    #[test]
    fn encode_missing_char() {
        let map = make_char_map();
        let encoder = Wubi86Encoder;
        assert!(encoder.encode("你好啊", &map).is_none());
    }
}
