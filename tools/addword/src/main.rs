use adw::dict::entry::Entry;
use adw::dict::file::load_dict;
use adw::dict::header::DictHeader;
use adw::encoder::char_map::CharMap;
use adw::encoder::wubi86::Wubi86Encoder;
use adw::error::{AppError, AppResult};
use adw::rime::deploy;
use adw::rime::dir::RimeDirDetector;
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

    #[arg(short = 'y', long = "yes", help = "交互模式自动编码时跳过确认")]
    yes: bool,

    #[arg(long = "no-deploy", help = "添加后不触发重新部署")]
    no_deploy: bool,
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

fn ensure_import_tables(lines: &mut Vec<String>) -> bool {
    let old_header_end = lines.iter().position(|l| l.trim() == "...")
        .filter(|&i| i > 0)
        .unwrap_or(0);

    let Some(mut header) = DictHeader::parse(lines) else {
        eprintln!("  [warn] 未找到 YAML header，跳过");
        return false;
    };

    if header.add_import(EXTRA_NAME) {
        let new_header = header.to_lines().to_vec();
        lines.splice(0..old_header_end + 1, new_header);
        eprintln!("  [info] 已将 {EXTRA_NAME} 添加到 import_tables");
        true
    } else {
        false
    }
}

fn create_extra_dict(path: &Path) -> AppResult<()> {
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

fn build_char_map(paths: &[PathBuf]) -> CharMap {
    let mut combined = Vec::new();
    for path in paths {
        if path.exists() {
            if let Ok(lines) = read_lines(path) {
                if let Some(header) = DictHeader::parse(&lines) {
                    for line in &lines[header.header_end + 1..] {
                        if let Some(entry) = Entry::parse(line) {
                            combined.push(entry);
                        }
                    }
                }
            }
        }
    }
    CharMap::from_entries(&combined)
}

fn run_cli(args: &Args, rime_dir: &Path) -> AppResult<()> {
    let main_path = rime_dir.join(MAIN_DICT);
    let extra_path = rime_dir.join(EXTRA_DICT);

    if !main_path.exists() {
        return Err(AppError::DictNotFound(main_path));
    }

    if !extra_path.exists() {
        create_extra_dict(&extra_path)?;
    }

    let word = args.word.as_deref().unwrap_or("");
    let code = args.code.as_deref().unwrap_or("");
    let weight = args.weight.unwrap_or(10);

    if word.is_empty() || code.is_empty() {
        return Err(AppError::EncodeFailed("需要提供词和编码".into()));
    }

    let mut dict = load_dict(&extra_path)?;
    let entry = Entry::new(word.to_string(), code.to_string(), weight);

    if dict.add_entry(entry.clone()) {
        eprintln!("  [ok] {}  {}  {}", entry.word, entry.code, entry.weight);
    } else {
        if !args.silent {
            let existing = dict.entries.iter()
                .find(|e| e.word == entry.word && e.code == entry.code);
            if let Some(e) = existing {
                eprintln!("  [skip] {}  {}  [权重 {}]", entry.word, entry.code, e.weight);
            }
        }
        try_deploy(args.no_deploy);
        return Ok(());
    }

    dict.save(&extra_path)?;
    try_deploy(args.no_deploy);
    Ok(())
}

fn run_interactive(args: &Args, rime_dir: &Path) -> AppResult<()> {
    let main_path = rime_dir.join(MAIN_DICT);
    let extra_path = rime_dir.join(EXTRA_DICT);

    if !main_path.exists() {
        return Err(AppError::DictNotFound(main_path));
    }

    if !extra_path.exists() {
        create_extra_dict(&extra_path)?;
    }

    let char_map = build_char_map(&[main_path, extra_path.clone()]);
    let mut dict = load_dict(&extra_path)?;

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

        let code = if code.is_empty() {
            let encoder = Wubi86Encoder;
            match encoder.encode(&word, &char_map) {
                Some(encoded) => {
                    print!("  -> 自动生成编码：{word} -> {encoded}");
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

        let entry = Entry::new(word.clone(), code.clone(), weight);
        if dict.add_entry(entry) {
            count += 1;
            eprintln!("  [ok] {}  {}  {}", word, code, weight);
        } else if !args.silent {
            let existing = dict.entries.iter()
                .find(|e| e.word == word && e.code == code);
            if let Some(e) = existing {
                eprintln!("  [skip] {}  {}  [权重 {}]", word, code, e.weight);
            }
        }
    }

    if count > 0 {
        dict.save(&extra_path)?;
        println!("\n完成，共添加 {count} 条");
    }
    try_deploy(args.no_deploy);
    Ok(())
}

fn try_deploy(no_deploy: bool) {
    deploy::try_deploy(no_deploy);
}

fn main() {
    let args = Args::parse();
    let detector = RimeDirDetector;

    let rime_dir = match &args.dir {
        Some(dir) => {
            let path = PathBuf::from(dir);
            if !path.exists() {
                eprintln!("[error] 指定的目录不存在：{dir}");
                std::process::exit(1);
            }
            path
        }
        None => match detector.detect() {
            Some(path) => path,
            None => {
                eprintln!("[error] 未找到 Rime 用户目录，请通过 -d 指定或设置 RIME_USER_DIR 环境变量");
                std::process::exit(1);
            }
        },
    };

    let main_path = rime_dir.join(MAIN_DICT);
    if main_path.exists() {
        if let Ok(mut main_lines) = read_lines(&main_path) {
            if ensure_import_tables(&mut main_lines) {
                let _ = write_lines(&main_path, &main_lines);
            }
        }
    }

    let is_interactive = args.interactive
        || (args.word.is_none() && args.code.is_none());

    let result = if is_interactive {
        run_interactive(&args, &rime_dir)
    } else {
        run_cli(&args, &rime_dir)
    };

    if let Err(e) = result {
        eprintln!("[error] {e}");
        std::process::exit(1);
    }
}
