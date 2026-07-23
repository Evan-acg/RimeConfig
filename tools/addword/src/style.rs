use colored::*;
use std::fmt::Display;

pub fn success(msg: impl Display) {
    eprintln!("  {}", format!("{}", msg).green().bold());
}

pub fn info(msg: impl Display) {
    eprintln!("  {}", format!("{}", msg).cyan());
}

pub fn attention(msg: impl Display) {
    eprintln!("  {}", format!("{}", msg).yellow());
}

pub fn detail(msg: impl Display) {
    eprintln!("  {}", format!("{}", msg).dimmed());
}

pub fn error(msg: impl Display) {
    eprintln!("  {}", format!("{}", msg).red().bold());
}

pub fn deploy(msg: impl Display) {
    eprintln!("  {}", format!("{}", msg).magenta());
}
