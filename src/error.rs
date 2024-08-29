
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to parse input because {0}")]
    ParsingError(String),
    #[error("Connection interrupted")]
    Interrupt(),
    #[error("I/O error occurred: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[error("{0}")]
    Invalid(String),
}

pub type Result<T> = core::result::Result<T, Error>;

