use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    DictNotFound(PathBuf),
    HeaderMissing(String),
    RimeDirNotFound,
    EncodeFailed(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO 错误: {e}"),
            AppError::DictNotFound(path) => write!(f, "码表不存在: {}", path.display()),
            AppError::HeaderMissing(dict) => write!(f, "{dict} 缺少 YAML header 结束符"),
            AppError::RimeDirNotFound => write!(f, "未找到 Rime 用户目录，请通过 -d 指定或设置 RIME_USER_DIR 环境变量"),
            AppError::EncodeFailed(word) => write!(f, "存在未收录的字，无法自动编码: {word}"),
        }
    }
}

impl From<io::Error> for AppError {
    fn from(e: io::Error) -> Self {
        AppError::Io(e)
    }
}

pub type AppResult<T> = Result<T, AppError>;
