use crate::dict::entry::Entry;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CharMap {
    inner: HashMap<char, String>,
}

impl CharMap {
    pub fn from_entries(entries: &[Entry]) -> Self {
        let mut inner: HashMap<char, (String, u32)> = HashMap::new();

        for entry in entries {
            if entry.is_group() || entry.word.chars().count() != 1 {
                continue;
            }
            let ch = entry.word.chars().next().unwrap();
            let weight = entry.weight;
            let code = entry.code.clone();

            let better = inner.get(&ch)
                .map(|(_, w)| weight > *w)
                .unwrap_or(true);

            if better {
                inner.insert(ch, (code, weight));
            }
        }

        CharMap { inner: inner.into_iter().map(|(k, (code, _))| (k, code)).collect() }
    }

    pub fn from_lines(lines: &[String]) -> Self {
        let entries: Vec<Entry> = lines.iter()
            .filter_map(|l| Entry::parse(l))
            .collect();
        Self::from_entries(&entries)
    }

    pub fn get(&self, ch: char) -> Option<&str> {
        self.inner.get(&ch).map(|s| s.as_str())
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn contains(&self, ch: char) -> bool {
        self.inner.contains_key(&ch)
    }

    pub fn into_inner(self) -> HashMap<char, String> {
        self.inner
    }
}

impl From<CharMap> for HashMap<char, String> {
    fn from(map: CharMap) -> Self {
        map.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_highest_weight() {
        let entries = vec![
            Entry::new("行".into(), "q".into(), 10),
            Entry::new("行".into(), "tf".into(), 30),
            Entry::new("行".into(), "hh".into(), 20),
        ];
        let map = CharMap::from_entries(&entries);
        assert_eq!(map.get('行'), Some("tf"));
    }
}
