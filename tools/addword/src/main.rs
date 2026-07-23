use clap::Parser;
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

    let all_lines = read_lines(&extra_path)?;
    let header_end = find_header_end(&all_lines).expect("extra 字典缺少 ... 结束符");
    let mut body_lines: Vec<String> = all_lines[header_end + 1..].to_vec();

    let stdin = io::stdin();
    let mut count = 0u32;

    println!("\n交互式造词（输入空词退出）");
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
        if code.is_empty() {
            eprintln!("  [error] 编码不能为空");
            continue;
        }

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
