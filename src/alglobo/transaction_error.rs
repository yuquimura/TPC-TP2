use std::fmt;

#[allow(dead_code)]
type Result<T> = std::result::Result<T, TransactionError>;

#[derive(Debug, PartialEq)]
pub enum TransactionError {
    Timeout,
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TransactionError::Timeout => write!(f, "Timeout en transacción"),
        }
    }
}
