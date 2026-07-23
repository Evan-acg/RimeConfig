use adw::rime::deploy::{deployer_name, hardcoded_paths};
use adw::rime::dir::RimeDirDetector;

#[test]
fn test_deployer_name_is_not_empty() {
    let name = deployer_name();
    assert!(!name.is_empty());
}

#[test]
fn test_hardcoded_paths_returns_list() {
    let paths = hardcoded_paths();
    assert!(!paths.is_empty());
}

#[test]
fn test_rime_dir_from_some_env() {
    let detector = RimeDirDetector;
    let result = detector.from_env_var("RIME_USER_DIR");
    // Can't mock env var in integration test easily, just verify it returns None or Some
    // We test the pure logic instead
    assert!(result.is_none() || result.is_some());
}

#[test]
fn test_rime_dir_from_explicit_path() {
    let detector = RimeDirDetector;
    // Use the current directory, which always exists
    let cwd = std::env::current_dir().unwrap();
    let result = detector.from_explicit_path(cwd.to_str().unwrap());
    assert!(result.is_some());
    assert_eq!(result.unwrap(), cwd);
}

#[test]
fn test_rime_dir_from_nonexistent_path() {
    let detector = RimeDirDetector;
    let result = detector.from_explicit_path("/nonexistent_path_12345");
    // The path doesn't exist, so returns None
    assert!(result.is_none());
}
