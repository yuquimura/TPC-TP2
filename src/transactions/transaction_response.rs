use super::transaction_code::TransactionCode;

pub struct TransactionResponse;

impl TransactionResponse {
    #[must_use]
    pub fn build(code: TransactionCode, id: u64) -> Vec<u8> {
        let code = TransactionResponse::map_transaction_code(code);
        let mut message = vec![code];
        message.append(&mut id.to_be_bytes().to_vec());
        message
    }

    fn map_transaction_code(code: TransactionCode) -> u8 {
        let err_msg = format!("[Transaction Response] No hay respuesta para {}", code);
        match code {
            TransactionCode::Accept => b'o',
            _ => panic!("{}", err_msg)
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
}