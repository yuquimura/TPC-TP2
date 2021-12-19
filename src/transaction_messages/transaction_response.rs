use std::{mem::size_of, convert::TryInto};

use super::{transaction_code::TransactionCode, types::RESPONSE_BYTE};

const ACCEPT_BYTE: u8 = b'o';
const ABORT_BYTE: u8 = b'A';
const COMMIT_BYTE: u8 = b'C';

pub struct TransactionResponse;

impl TransactionResponse {
    #[must_use]
    pub fn size() -> usize {
        TransactionResponse::build(TransactionCode::Accept, 0).len()
    }

    #[must_use]
    pub fn build(code: TransactionCode, id: u64) -> Vec<u8> {
        let mut message = vec![RESPONSE_BYTE];
        message.push(TransactionResponse::map_transaction_code(code));
        message.append(&mut id.to_be_bytes().to_vec());
        message
    }

    /// # Panics
    ///
    /// Esta funcion paniquea si:
    /// - se recibio un codigo de transaccion desconocido
    #[must_use]
    pub fn transaction_code(code: u8) -> TransactionCode {
        let err_msg = format!(
            "[Transaction Response] Codigo de transaccion desconocido: {}",
            code
        );
        match code {
            ACCEPT_BYTE => TransactionCode::Accept,
            ABORT_BYTE => TransactionCode::Abort,
            COMMIT_BYTE => TransactionCode::Commit,
            _ => panic!("{}", err_msg),
        }
    }

    fn map_transaction_code(code: TransactionCode) -> u8 {
        let err_msg = format!("[Transaction Response] No hay respuesta para {}", code);
        match code {
            TransactionCode::Accept => ACCEPT_BYTE,
            TransactionCode::Abort => ABORT_BYTE,
            TransactionCode::Commit => COMMIT_BYTE,
            TransactionCode::Prepare => panic!("{}", err_msg),
        }
    }

    pub fn parse(message: &[u8]) -> (TransactionCode, u64) {
        let code = TransactionResponse::transaction_code(message[1]);
        let id_bytes: [u8; size_of::<u64>()] = message[2..2 + size_of::<u64>()]
            .try_into()
            .expect("[Transaction Response] Los ids deberian ocupar 8 bytes");
        let id = u64::from_be_bytes(id_bytes);
        (code, id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_should_return_ok_with_id() {
        let id = 0;
        let message = TransactionResponse::build(TransactionCode::Accept, id);
        let mut expected = vec![RESPONSE_BYTE, b'o'];
        expected.append(&mut id.to_be_bytes().to_vec());

        assert_eq!(message, expected);
    }

    #[test]
    fn transaction_code_should_return_accept_when_it_is_o() {
        let code = TransactionResponse::transaction_code(b'o');

        assert_eq!(code, TransactionCode::Accept);
    }

    #[test]
    fn size_should_be_the_len_of_result_of_build() {
        let id = 0;
        let message = TransactionResponse::build(TransactionCode::Accept, id);
        assert_eq!(TransactionResponse::size(), message.len());
    }
}
