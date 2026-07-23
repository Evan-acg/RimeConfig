use clap::Parser;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

const MAIN_DICT: &str = "evan_mixied_wubi_pinyin.dict.yaml";
const EXTRA_DICT: &str = "wubi86_jidian_extra.dict.yaml";
const EXTRA_NAME: &str = "wubi86_jidian_extra";

#[derive(Parser)]
#[command(name = "rime-addword", about = "快速向 Rime 码表添加词条")]
struct Args {
    word: Option<String>,
    code: Option<String>,
    weight: Option<u32>,

    #[arg(short = 'd', long = "dir", help = "Rime 用户目录（默认自动检测）")]
    dir: Option<String>,

    #[arg(short = 's', long = "silent", help = "静默模式，不显示重复提示")]
    silent: bool,

    #[arg(short = 'i', long = "interactive", help = "强制交互模式")]
    interactive: bool,

    #[arg(short = 'y', long = "yes", help = "交互模式自动编码时跳过确认")]
    yes: bool,

    #[arg(long = "no-deploy", help = "添加后不触发重新部署")]
    no_deploy: bool,
}

fn find_rime_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("RIME_USER_DIR") {
        let path = PathBuf::from(dir);
        if path.exists() {
            return Some(path);
        }
    }

    #[cfg(target_os = "windows")]
    if let Ok(appdata) = std::env::var("APPDATA") {
        let path = PathBuf::from(appdata).join("Rime");
        if path.exists() {
            return Some(path);
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let path = PathBuf::from(home).join("Library/Rime");
            if path.exists() {
                return Some(path);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
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
    }

    None
}

fn read_lines(path: &Path) -> io::Result<Vec<String>> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    reader.lines().collect()
}

fn write_lines(path: &Path, lines: &[String]) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    for line in lines {
        writeln!(file, "{line}")?;
    }
    Ok(())
}

fn find_header_end(lines: &[String]) -> Option<usize> {
    // Find the position of `...` that marks end of YAML header
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "..." && i > 0 {
            return Some(i);
        }
    }
    None
}

fn has_import(header_section: &[String], import_name: &str) -> bool {
    let target = format!("- {import_name}");
    header_section
        .iter()
        .any(|l| l.trim_start().starts_with(&target))
}

fn ensure_import_tables(lines: &mut Vec<String>) -> bool {
    let target_line = format!("  - {EXTRA_NAME}");

    if has_import(lines, EXTRA_NAME) {
        return false;
    }

    // Find `import_tables:` and the end of its block
    let mut insert_at = None;
    for i in 0..lines.len() {
        let trimmed = lines[i].trim_start();
        if trimmed.starts_with("import_tables:") || trimmed.starts_with("# import_tables:") {
            // Find where the import block ends
            let mut j = i + 1;
            while j < lines.len() {
                let t = lines[j].trim_start();
                if t.starts_with("- ") || lines[j].trim().is_empty() || t.starts_with('#') {
                    j += 1;
                } else {
                    break;
                }
            }
            // j is the first line after the import block
            insert_at = Some(j);
            break;
        }
    }

    if let Some(pos) = insert_at {
        lines.insert(pos, target_line);
        eprintln!("  [info] 已将 {EXTRA_NAME} 添加到 import_tables");
        true
    } else {
        // No import_tables found, insert before `columns:` or `encoder:` or `...`
        // This is unusual but we handle it
        eprintln!("  [warn] 未找到 import_tables，跳过");
        false
    }
}

fn create_extra_dict(path: &Path) -> io::Result<()> {
    let content = format!(
        "---\n\
         name: {EXTRA_NAME}\n\
         version: \"0.0.1\"\n\
         sort: by_weight\n\
         columns:\n\
         \x20 - text\n\
         \x20 - code\n\
         \x20 - weight\n\
         \x20 - stem\n\
         ...\n"
    );
    fs::write(path, content)?;
    eprintln!("  [info] 已创建 {EXTRA_DICT}");
    Ok(())
}

fn parse_weight(line: &str) -> u32 {
    let parts: Vec<&str> = line.rsplit('\t').collect();
    parts.first().and_then(|w| w.trim().parse().ok()).unwrap_or(0)
}

fn entry_exists<'a>(lines: &'a [String], word: &str, code: &str) -> Option<&'a String> {
    let needle = format!("{word}\t{code}");
    lines.iter().find(|l| {
        let trimmed = l.trim();
        trimmed.starts_with(&needle) && {
            let after = &trimmed[needle.len()..];
            after.is_empty() || after.starts_with('\t')
        }
    })
}

fn format_entry(word: &str, code: &str, weight: u32) -> String {
    format!("{word}\t{code}\t{weight}")
}

fn build_char_map(paths: &[PathBuf]) -> HashMap<char, String> {
    let mut map: HashMap<char, (String, u32)> = HashMap::new();

    for path in paths {
        if !path.exists() {
            continue;
        }
        if let Ok(lines) = read_lines(path) {
            if let Some(end) = find_header_end(&lines) {
                for line in &lines[end + 1..] {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') || line.starts_with("##") {
                        continue;
                    }
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() < 2 {
                        continue;
                    }
                    let text = parts[0];
                    let code = parts[1];
                    if text.chars().count() == 1 {
                        let weight = parts.get(2).and_then(|w| w.parse::<u32>().ok()).unwrap_or(0);
                        let ch = text.chars().next().unwrap();
                        let entry = map.entry(ch).or_insert_with(|| (code.to_string(), weight));
                        if weight > entry.1 {
                            *entry = (code.to_string(), weight);
                        }
                    }
                }
            }
        }
    }

    map.into_iter().map(|(k, v)| (k, v.0)).collect()
}

fn take1(s: &str) -> String {
    s.chars().take(1).collect()
}

fn take2(s: &str) -> String {
    s.chars().take(2).collect()
}

fn encode_word(word: &str, char_map: &HashMap<char, String>) -> Option<String> {
    let chars: Vec<char> = word.chars().collect();
    let n = chars.len();
    if n < 2 {
        return None;
    }

    let codes: Vec<&str> = chars
        .iter()
        .map(|c| char_map.get(c).map(|s| s.as_str()))
        .collect::<Option<Vec<_>>>()?;

    match n {
        2 => {
            let code = format!("{}{}", take2(codes[0]), take2(codes[1]));
            Some(code)
        }
        3 => {
            let code = format!("{}{}{}", take1(codes[0]), take1(codes[1]), take2(codes[2]));
            Some(code)
        }
        _ => {
            let code = format!(
                "{}{}{}{}",
                take1(codes[0]),
                take1(codes[1]),
                take1(codes[2]),
                take1(codes[n - 1])
            );
            Some(code)
        }
    }
}

fn add_entry_main(lines: &mut Vec<String>, word: &str, code: &str, weight: u32, silent: bool) -> bool {
    if let Some(existing) = entry_exists(lines, word, code) {
        if !silent {
            let w = parse_weight(existing);
            eprintln!("  [skip] 已存在：{word}  {code}  [权重 {w}]");
        }
        return false;
    }
    lines.push(format_entry(word, code, weight));
    eprintln!("  [ok] {word}  {code}  {weight}");
    true
}

fn deployer_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "WeaselDeployer.exe"
    }
    #[cfg(not(target_os = "windows"))]
    {
        "rime_deployer"
    }
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

fn hardcoded_paths() -> Vec<PathBuf> {
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

fn deploy() -> bool {
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

fn try_deploy(no_deploy: bool) {
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

fn run_cli(args: &Args, rime_dir: &Path) -> io::Result<()> {
    let main_path = rime_dir.join(MAIN_DICT);
    let extra_path = rime_dir.join(EXTRA_DICT);

    if !main_path.exists() {
        eprintln!("[error] 未找到主码表：{}", main_path.display());
        std::process::exit(1);
    }

    // Ensure extra dictionary exists
    if !extra_path.exists() {
        create_extra_dict(&extra_path)?;
    }

    // Add entry to extra dictionary
    let word = args.word.as_deref().unwrap_or("");
    let code = args.code.as_deref().unwrap_or("");
    let weight = args.weight.unwrap_or(10);

    if word.is_empty() || code.is_empty() {
        eprintln!("[error] 需要提供词和编码");
        std::process::exit(1);
    }

    let lines = read_lines(&extra_path)?;
    let header_end = find_header_end(&lines).expect("extra 字典缺少 ... 结束符");
    let mut body: Vec<String> = lines[header_end + 1..].to_vec();

    add_entry_main(&mut body, word, code, weight, args.silent);

    let mut out: Vec<String> = lines[..=header_end].to_vec();
    out.append(&mut body);
    write_lines(&extra_path, &out)?;
    try_deploy(args.no_deploy);
    Ok(())
}

fn run_interactive(args: &Args, rime_dir: &Path) -> io::Result<()> {
    let main_path = rime_dir.join(MAIN_DICT);
    let extra_path = rime_dir.join(EXTRA_DICT);

    if !main_path.exists() {
        eprintln!("[error] 未找到主码表：{}", main_path.display());
        std::process::exit(1);
    }

    if !extra_path.exists() {
        create_extra_dict(&extra_path)?;
    }

    // Build character→code map from main + extra dictionaries
    let char_map = build_char_map(&[main_path.clone(), extra_path.clone()]);

    let all_lines = read_lines(&extra_path)?;
    let header_end = find_header_end(&all_lines).expect("extra 字典缺少 ... 结束符");
    let mut body_lines: Vec<String> = all_lines[header_end + 1..].to_vec();

    let stdin = io::stdin();
    let mut count = 0u32;

    println!("\n交互式造词（输入空词退出，编码为空时自动生成）");
    loop {
        print!("词: ");
        io::stdout().flush()?;
        let mut word = String::new();
        stdin.lock().read_line(&mut word)?;
        let word = word.trim().to_string();
        if word.is_empty() {
            break;
        }

        print!("编码: ");
        io::stdout().flush()?;
        let mut code = String::new();
        stdin.lock().read_line(&mut code)?;
        let code = code.trim().to_string();

        // Auto-encode when code is empty
        let code = if code.is_empty() {
            match encode_word(&word, &char_map) {
                Some(encoded) => {
                    print!("  → 自动生成编码：{word} → {encoded}");
                    if args.yes {
                        println!();
                        encoded
                    } else {
                        print!("  确认添加？(Y/n): ");
                        io::stdout().flush()?;
                        let mut confirm = String::new();
                        stdin.lock().read_line(&mut confirm)?;
                        let confirm = confirm.trim().to_lowercase();
                        if confirm == "n" || confirm == "no" {
                            eprintln!("  [skip] 已取消");
                            continue;
                        }
                        encoded
                    }
                }
                None => {
                    eprintln!("  [error] 存在未收录的字，请手动输入编码");
                    continue;
                }
            }
        } else {
            code
        };

        print!("权重（默认10）: ");
        io::stdout().flush()?;
        let mut weight_input = String::new();
        stdin.lock().read_line(&mut weight_input)?;
        let weight = weight_input.trim().parse::<u32>().unwrap_or(10);

        if add_entry_main(&mut body_lines, &word, &code, weight, args.silent) {
            count += 1;
        }
    }

    // Write back
    let mut out: Vec<String> = all_lines[..=header_end].to_vec();
    out.append(&mut body_lines);
    write_lines(&extra_path, &out)?;

    if count > 0 {
        println!("\n完成，共添加 {count} 条");
    }
    try_deploy(args.no_deploy);
    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let rime_dir = if let Some(dir) = &args.dir {
        let path = PathBuf::from(dir);
        if !path.exists() {
            eprintln!("[error] 指定的目录不存在：{dir}");
            std::process::exit(1);
        }
        path
    } else if let Some(path) = find_rime_dir() {
        path
    } else {
        eprintln!("[error] 未找到 Rime 用户目录，请通过 -d 指定或设置 RIME_USER_DIR 环境变量");
        std::process::exit(1);
    };

    // Ensure import_tables in main dictionary references extra dict
    let main_path = rime_dir.join(MAIN_DICT);
    if main_path.exists() {
        let mut main_lines = read_lines(&main_path)?;
        if ensure_import_tables(&mut main_lines) {
            write_lines(&main_path, &main_lines)?;
        }
    }

    let is_interactive = args.interactive
        || (args.word.is_none() && args.code.is_none());

    if is_interactive {
        run_interactive(&args, &rime_dir)
    } else {
        run_cli(&args, &rime_dir)
    }
}
