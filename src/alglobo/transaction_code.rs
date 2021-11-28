use std::fmt;

#[derive(Clone, Copy)]
pub enum TransactionCode {
    Prepare,
    Abort,
}

impl fmt::Display for TransactionCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TransactionCode::Prepare => write!(f, "PREPARAR"),
            TransactionCode::Abort => write!(f, "ABORTAR"),
        }
    }
}