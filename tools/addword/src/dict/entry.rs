#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub word: String,
    pub code: String,
    pub weight: u32,
    pub group: Option<String>,
}

impl Entry {
    pub fn new(word: String, code: String, weight: u32) -> Self {
        Entry { word, code, weight, group: None }
    }

    pub fn group(name: String) -> Self {
        Entry { word: format!("## {name}"), code: String::new(), weight: 0, group: Some(name) }
    }

    pub fn parse(line: &str) -> Option<Self> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        // Comment lines
        if trimmed.starts_with('#') && !trimmed.starts_with("##") {
            return None;
        }

        // Group header
        if trimmed.starts_with("##") {
            let name = trimmed.trim_start_matches("##").trim().to_string();
            return Some(Entry { word: trimmed.to_string(), code: String::new(), weight: 0, group: Some(name) });
        }

        let parts: Vec<&str> = trimmed.split('\t').collect();
        if parts.len() < 2 {
            return None;
        }

        let word = parts[0].to_string();
        let code = parts[1].to_string();
        let weight = parts.get(2).and_then(|w| w.parse::<u32>().ok()).unwrap_or(0);

        Some(Entry { word, code, weight, group: None })
    }

    pub fn is_group(&self) -> bool {
        self.group.is_some()
    }

    pub fn format(&self) -> String {
        if self.is_group() {
            return self.word.clone();
        }
        if self.weight > 0 {
            format!("{}\t{}\t{}", self.word, self.code, self.weight)
        } else {
            format!("{}\t{}", self.word, self.code)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_entry() {
        let e = Entry::parse("工\ta\t20").unwrap();
        assert_eq!(e.word, "工");
        assert_eq!(e.code, "a");
        assert_eq!(e.weight, 20);
    }
}
