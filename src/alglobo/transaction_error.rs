use std::fmt;

#[allow(dead_code)]
type Result<T> = std::result::Result<T, TransactionError>;

#[derive(Debug, PartialEq)]
pub enum TransactionError {
    None,
    Timeout,
    WrongId,
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TransactionError::None => write!(f, "No hay transaccion siendo procesada"),
            TransactionError::Timeout => write!(f, "Timeout en transaccion"),
            TransactionError::WrongId => write!(f, "No es la transaccion actual"),
        }
    }
}
