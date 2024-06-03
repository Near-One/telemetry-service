use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error")]
    IOError(#[from] io::Error),
    #[error("DB error")]
    DBError(#[from] sea_orm::DbErr),
    #[error("input error ({0})/n{1}")]
    InputError(String, String),
    #[error("database not found error")]
    DatabaseNotFound,
    #[error("unknown error")]
    Unknown,
}
