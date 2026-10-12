// mod error

#[derive(Debug)]
pub enum LsmError {
    Io(std::io::Error),
    Corrupt(&'static str),
}

impl From<std::io::Error> for LsmError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

pub type Result<T> = std::result::Result<T, LsmError>;
