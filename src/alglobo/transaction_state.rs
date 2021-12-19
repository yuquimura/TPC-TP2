use std::fmt;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum TransactionState {
    Waiting,
    Accepted,
    Aborted,
    Commited,
}

impl fmt::Display for TransactionState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TransactionState::Waiting => write!(f, "EN ESPERA"),
            TransactionState::Accepted => write!(f, "ACEPTADO"),
            TransactionState::Aborted => write!(f, "ABORTADO"),
            TransactionState::Commited => write!(f, "COMMITEADO"),
        }
    }
}

impl TransactionState {
    #[must_use]
    pub fn byte_code(&self) -> u8 {
        match *self {
            TransactionState::Waiting => b'W',
            TransactionState::Accepted => b'O',
            TransactionState::Aborted => b'A',
            TransactionState::Commited => b'C',
        }
    }

    /// # Panics
    ///
    /// Esta funcion paniquea cuando se recibe un byte
    /// para el cual no existe un estado de transaccion
    #[must_use]
    pub fn from_byte(byte: u8) -> Self {
        let err = format!("[TransactionState] No hay estado para el byte: {}", byte);
        match byte {
            b'W' => TransactionState::Waiting,
            b'O' => TransactionState::Accepted,
            b'A' => TransactionState::Aborted,
            b'C' => TransactionState::Commited,
            _ => panic!("{}", err),
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

    #[test]
    fn from_byte_should_return_a_transaction_state_for_each_valid_byte() {
        assert_eq!(TransactionState::from_byte(b'W'), TransactionState::Waiting);
        assert_eq!(
            TransactionState::from_byte(b'O'),
            TransactionState::Accepted
        );
        assert_eq!(TransactionState::from_byte(b'A'), TransactionState::Aborted);
        assert_eq!(
            TransactionState::from_byte(b'C'),
            TransactionState::Commited
        );
    }
}
