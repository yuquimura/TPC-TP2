use std::fmt;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum TransactionState {
    Waiting,
    Accepted,
    Aborted,
    Commited
}

impl fmt::Display for TransactionState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TransactionState::Waiting => write!(f, "EN ESPERA"),
            TransactionState::Accepted => write!(f, "ACEPTADOS"),
            TransactionState::Aborted => write!(f, "ABORTADOS"),
            TransactionState::Commited => write!(f, "COMMITEADOS"),
        }
    }
}
