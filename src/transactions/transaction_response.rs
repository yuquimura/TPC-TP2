use super::transaction_code::TransactionCode;

const ACCEPT_BYTE:u8 = b'o';

pub struct TransactionResponse;

impl TransactionResponse {
    #[must_use]
    pub fn size() -> usize {
        TransactionResponse::build(TransactionCode::Accept, 0).len()
    }

    #[must_use]
    pub fn build(code: TransactionCode, id: u64) -> Vec<u8> {
        let code = TransactionResponse::map_transaction_code(code);
        let mut message = vec![code];
        message.append(&mut id.to_be_bytes().to_vec());
        message
    }

    #[must_use]
    pub fn transaction_code(code: u8) -> TransactionCode {
        let err_msg = format!("[Transaction Response] Codigo de transacción desconocido: {}", code);
        match code {
            ACCEPT_BYTE => TransactionCode::Accept,
            _ => panic!("{}", err_msg)

        } 
    }

    fn map_transaction_code(code: TransactionCode) -> u8 {
        let err_msg = format!("[Transaction Response] No hay respuesta para {}", code);
        match code {
            TransactionCode::Accept => ACCEPT_BYTE,
            _ => panic!("{}", err_msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_should_return_ok_with_id() {
        let id = 0;
        let message = TransactionResponse::build(TransactionCode::Accept, id);
        let mut expected = vec![b'o'];
        expected.append(&mut id.to_be_bytes().to_vec());

        assert_eq!(message, expected);
    }

    #[test]
    fn transaction_code_should_return_accept_when_it_is_o() {
        let code = TransactionResponse::transaction_code(b'o');

        assert_eq!(code, TransactionCode::Accept);
    }
}
