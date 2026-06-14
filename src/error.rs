use thiserror::Error;

#[derive(Error, Debug)]
pub enum PolytypeError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml parse error: {0}")]
    TomlDe(#[from] toml::de::Error),
    #[error("toml serialize error: {0}")]
    TomlSer(#[from] toml::ser::Error),
    #[error("invalid layout '{name}': {reason}")]
    InvalidLayout { name: String, reason: String },
    #[error("unknown layout '{0}'")]
    UnknownLayout(String),
}

pub type Result<T> = std::result::Result<T, PolytypeError>;
