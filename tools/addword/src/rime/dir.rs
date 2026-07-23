use std::path::PathBuf;

pub struct RimeDirDetector;

impl RimeDirDetector {
    /// Check a specific env var for Rime directory
    pub fn from_env_var(&self, var: &str) -> Option<PathBuf> {
        if let Ok(dir) = std::env::var(var) {
            let path = PathBuf::from(dir);
            if path.exists() {
                return Some(path);
            }
        }
        None
    }

    /// Return a candidate path regardless of whether it exists
    pub fn from_explicit_path(&self, dir: &str) -> Option<PathBuf> {
        let path = PathBuf::from(dir);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// Detect Rime directory using platform defaults
    pub fn detect(&self) -> Option<PathBuf> {
        // Try env var first
        if let Some(path) = self.from_env_var("RIME_USER_DIR") {
            return Some(path);
        }

        #[cfg(target_os = "windows")]
        if let Ok(appdata) = std::env::var("APPDATA") {
            let path = PathBuf::from(appdata).join("Rime");
            if path.exists() {
                return Some(path);
            }
        }

        #[cfg(target_os = "macos")]
        if let Ok(home) = std::env::var("HOME") {
            let path = PathBuf::from(home).join("Library/Rime");
            if path.exists() {
                return Some(path);
            }
        }

        #[cfg(target_os = "linux")]
        if let Ok(home) = std::env::var("HOME") {
            let candidates = [
                PathBuf::from(&home).join(".config/fcitx/rime"),
                PathBuf::from(&home).join(".local/share/fcitx5/rime"),
                PathBuf::from(&home).join(".config/ibus/rime"),
            ];
            for p in candidates {
                if p.exists() {
                    return Some(p);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_from_env() {
        std::env::set_var("RIME_USER_DIR_TEST", std::env::current_dir().unwrap());
        let detector = RimeDirDetector;
        let result = detector.from_env_var("RIME_USER_DIR_TEST");
        assert!(result.is_some());
        std::env::remove_var("RIME_USER_DIR_TEST");
    }

    #[test]
    fn detect_from_explicit_valid() {
        let detector = RimeDirDetector;
        let cwd = std::env::current_dir().unwrap();
        let result = detector.from_explicit_path(cwd.to_str().unwrap());
        assert!(result.is_some());
    }

    #[test]
    fn detect_from_explicit_invalid() {
        let detector = RimeDirDetector;
        let result = detector.from_explicit_path(r"C:\This\Path\Does\Not\Exist\12345");
        assert!(result.is_none());
    }
}
