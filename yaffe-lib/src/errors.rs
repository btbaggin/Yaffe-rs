use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum PluginError {
    MissingSetting(String),
    External(String),
    Other(String),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::MissingSetting(setting) => write!(f, "Missing required setting: {setting}"),
            PluginError::External(error) => write!(f, "{error}"),
            PluginError::Other(s) => write!(f, "{s}"),
        }
    }
}

impl PluginError {
    pub fn external<E: Error>(error: E) -> Self { Self::External(error.to_string()) }
    pub fn other<S: Into<String>>(error: S) -> Self { Self::Other(error.into()) }
}

pub trait PluginErrorExt<T> {
    fn fail(self) -> Result<T, PluginError>;
}

impl<T, E: Error> PluginErrorExt<T> for Result<T, E> {
    fn fail(self) -> Result<T, PluginError> { self.map_err(PluginError::external) }
}
