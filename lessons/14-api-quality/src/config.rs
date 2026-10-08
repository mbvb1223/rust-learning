use std::fmt;
use std::net::SocketAddr;

pub const DEFAULT_DATABASE_URL: &str = "sqlite:tasks.db";
pub const DEFAULT_BIND_ADDR: &str = "127.0.0.1:3000";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: SocketAddr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// The variable is set, but empty or only whitespace. Holds the variable name.
    Empty(&'static str),
    /// `BIND_ADDR` is not an `ip:port` socket address. Holds the trimmed value.
    InvalidBindAddr(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Empty(name) => write!(f, "{name} is set but empty"),
            ConfigError::InvalidBindAddr(value) => write!(
                f,
                "BIND_ADDR must be an ip:port address such as 127.0.0.1:3000, got {value:?}"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// Builds the config from `DATABASE_URL` and `BIND_ADDR`, read through `lookup`.
    /// `lookup(name)` returns the variable's value, or `None` when it is unset.
    ///
    /// - Values are trimmed before use.
    /// - Unset → the default: [`DEFAULT_DATABASE_URL`], [`DEFAULT_BIND_ADDR`].
    /// - Set but empty after trimming → `ConfigError::Empty(name)`, not the default.
    /// - `BIND_ADDR` must parse as a [`SocketAddr`]: `127.0.0.1:3000`, `0.0.0.0:8080`, `[::1]:3000`.
    ///   Anything else, including host names such as `localhost:3000`, →
    ///   `ConfigError::InvalidBindAddr(trimmed value)`.
    /// - `DATABASE_URL` is checked first: when both are invalid, its error is the one returned.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Config, ConfigError> {
        todo!()
    }

    /// The real process environment. Unset and non-UTF-8 variables both count as unset.
    pub fn from_env() -> Result<Config, ConfigError> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }
}
