use std::result::Result as StdResult;
use std::error::Error as StdError;
use std::io;
use std::fmt;
use std::path::Path;
use std::path::StripPrefixError;

use git2;
use walkdir;
use yaml_rust::emitter::EmitError;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Format,
    Git(String),
    Editor,
    Build(String),
    PluginNotInstalled,
    PluginInstalled(String),
    CopyDir(String),
    SaveYaml,
}

impl Error {
    pub fn copy_dir(s: &str) -> Error {
        Error::CopyDir(format!("Fail to copy directory: {}", s))
    }

    pub fn build<T: AsRef<str>>(s: T) -> Error {
        Error::Build(format!("Fail to build plugin: {}", s.as_ref()))
    }

    pub fn plugin_installed<T: AsRef<Path>>(s: T) -> Error {
        Error::PluginInstalled(format!("Plugin already installed under {:?}", s.as_ref()))
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<git2::Error> for Error {
    fn from(err: git2::Error) -> Error {
        Error::Git(err.description().to_string())
    }
}

impl From<walkdir::Error> for Error {
    fn from(err: walkdir::Error) -> Error {
        Error::copy_dir(err.description())
    }
}

impl From<StripPrefixError> for Error {
    fn from(err: StripPrefixError) -> Error {
        Error::copy_dir(err.description())
    }
}

impl From<EmitError> for Error {
    fn from(_: EmitError) -> Error {
        Error::SaveYaml
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Format => "Invalid format",
            Error::SaveYaml => "Fail to save packfile",
            Error::Editor => "Can not open editor",
            Error::PluginNotInstalled => "Plugin not installed",
            Error::Io(ref e) => e.description(),
            Error::Build(ref s) => s,
            Error::Git(ref s) => s,
            Error::CopyDir(ref s) => s,
            Error::PluginInstalled(ref s) => s,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}
