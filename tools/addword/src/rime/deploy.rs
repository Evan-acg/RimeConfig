use std::path::{Path, PathBuf};

pub fn deployer_name() -> &'static str {
    #[cfg(target_os = "windows")]
    { "WeaselDeployer.exe" }
    #[cfg(not(target_os = "windows"))]
    { "rime_deployer" }
}

pub fn hardcoded_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "windows")]
    {
        paths.push(PathBuf::from(r"C:\Program Files\Weasel\WeaselDeployer.exe"));
        paths.push(PathBuf::from(r"C:\Program Files (x86)\Weasel\WeaselDeployer.exe"));
        paths.push(PathBuf::from(r"D:\Program Files\Weasel\WeaselDeployer.exe"));
    }
    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/usr/local/bin/rime_deployer"));
        paths.push(PathBuf::from("/Library/Input Methods/Squirrel.app/Contents/MacOS/Squirrel"));
    }
    #[cfg(target_os = "linux")]
    {
        paths.push(PathBuf::from("/usr/bin/rime_deployer"));
        paths.push(PathBuf::from("/usr/local/bin/rime_deployer"));
    }

    paths
}

fn fd_search_roots() -> Vec<&'static str> {
    let mut roots = Vec::new();

    #[cfg(target_os = "windows")]
    {
        roots.push(r"C:\Program Files");
        roots.push(r"C:\Program Files (x86)");
        roots.push(r"D:\Program Files");
        roots.push(r"E:\Program Files");
    }
    #[cfg(target_os = "macos")]
    {
        roots.push("/Applications");
        roots.push("/usr/local");
        roots.push("/opt/homebrew");
    }
    #[cfg(target_os = "linux")]
    {
        roots.push("/usr");
        roots.push("/usr/local");
        roots.push("/opt");
    }

    roots
}

fn search_deployer_with_fd(name: &str) -> Option<PathBuf> {
    if which::which("fd").is_err() {
        return None;
    }

    for root in fd_search_roots() {
        let root_path = Path::new(root);
        if !root_path.exists() {
            continue;
        }

        let output = std::process::Command::new("fd")
            .arg("-t")
            .arg("f")
            .arg("--max-depth")
            .arg("6")
            .arg(name)
            .arg(root)
            .output()
            .ok()?;

        if output.status.success() {
            let line = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()?
                .trim()
                .to_string();
            if !line.is_empty() {
                return Some(PathBuf::from(line));
            }
        }
    }
    None
}

fn run_deployer(path: &Path) -> bool {
    let name = path.file_stem().unwrap_or_default().to_string_lossy();
    let is_weasel = name.contains("WeaselDeployer");

    let mut cmd = std::process::Command::new(path);
    if is_weasel {
        cmd.arg("/deploy");
    }

    eprintln!("  [deploy] 正在运行部署程序...");
    cmd.spawn()
        .map(|_| {
            eprintln!("  [deploy] 部署已触发");
            true
        })
        .unwrap_or_else(|e| {
            eprintln!("  [deploy] 启动失败：{e}");
            false
        })
}

pub fn deploy() -> bool {
    let name = deployer_name();

    // 1. 先用 fd 搜索
    if let Some(path) = search_deployer_with_fd(name) {
        return run_deployer(&path);
    }

    // 2. 再检查硬编码路径
    for path in hardcoded_paths() {
        if path.exists() {
            return run_deployer(&path);
        }
    }

    // 3. 最后检查 PATH
    if let Ok(path) = which::which(name) {
        return run_deployer(&path);
    }

    false
}

pub fn try_deploy(no_deploy: bool) {
    if no_deploy {
        return;
    }

    if deploy() {
        eprintln!("\n[info] 正在重新部署 Rime，请稍候...");
    } else {
        eprintln!("\n[info] 词条已添加，请手动重新部署 Rime 使其生效。");
        eprintln!("  Windows: 右键托盘图标 → 重新部署");
        eprintln!("  macOS:   系统语言菜单 → 重新部署");
        eprintln!("  Linux:   输入法菜单 → 重新部署");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deployer_name_not_empty() {
        assert!(!deployer_name().is_empty());
    }

    #[test]
    fn hardcoded_paths_not_empty() {
        let paths = hardcoded_paths();
        assert!(!paths.is_empty());
    }

    #[test]
    fn fd_search_roots_not_empty() {
        let roots = fd_search_roots();
        assert!(!roots.is_empty());
    }
}
