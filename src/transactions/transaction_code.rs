use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TransactionCode {
    Prepare,
    Abort,
    Accept,
    Quit,
}

impl fmt::Display for TransactionCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TransactionCode::Prepare => write!(f, "PREPARAR"),
            TransactionCode::Abort => write!(f, "ABORTAR"),
            TransactionCode::Accept => write!(f, "ACEPTAR"),
            TransactionCode::Quit => write!(f, "SALIR"),
        }
    }
}
