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

impl TransactionState {
    pub fn byte_code(&self) -> u8 {
        match *self {
            TransactionState::Waiting => b'W',
            TransactionState::Accepted => b'O',
            TransactionState::Aborted => b'A',
            TransactionState::Commited => b'C',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_should_return_a_byte_representation_for_each_state() {
        assert_eq!(TransactionState::Waiting.byte_code(), b'W');
        assert_eq!(TransactionState::Accepted.byte_code(), b'O');
        assert_eq!(TransactionState::Aborted.byte_code(), b'A');
        assert_eq!(TransactionState::Commited.byte_code(), b'C');
    }
}   