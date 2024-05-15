use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error")]
    IOError(#[from] io::Error),
    #[error("DB error")]
    DBError(#[from] sea_orm::DbErr),
    #[error("input error")]
    InputError(String),
    #[error("unknown error")]
    Unknown,
}