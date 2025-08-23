use std::fmt;

#[derive(Debug, Clone)]
pub enum StateEvents{
    TransactionError,
    TransactionSuccess,
    InvalidExtension,
    UrlExists,
    UrlError
}