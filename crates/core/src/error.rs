use thiserror::Error;

#[derive(Debug, Error)]
pub enum NovalynError {
    #[error("config error: {0}")]
    Config(String),
    #[error("git error: {0}")]
    Git(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("semantic error: {0}")]
    Semantic(String),
}

impl From<anyhow::Error> for NovalynError {
    fn from(e: anyhow::Error) -> Self {
        Self::Semantic(e.to_string())
    }
}

impl From<git2::Error> for NovalynError {
    fn from(e: git2::Error) -> Self {
        Self::Git(e.to_string())
    }
}

impl From<std::io::Error> for NovalynError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl NovalynError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Config(_) => 2,
            Self::Git(_) => 4,
            Self::Io(_) => 5,
            Self::Semantic(_) => 6,
        }
    }
}
