use super::transaction_code::TransactionCode;

pub struct TransactionRequest;

impl TransactionRequest {
    #[must_use]
    pub fn size() -> usize {
        TransactionRequest::build(TransactionCode::Prepare, 0, 0.0).len()
    }

    #[must_use]
    pub fn build(code: TransactionCode, id: u64, fee: f64) -> Vec<u8> {
        let code = TransactionRequest::map_transaction_code(code);
        let mut message = vec![code];
        message.append(&mut id.to_be_bytes().to_vec());
        message.append(&mut fee.to_be_bytes().to_vec());
        message
    }

    pub(crate) fn map_transaction_code(code: TransactionCode) -> u8 {
        let err_msg = format!("[Transaction Response] No hay solicitud para {}", code);
        match code {
            TransactionCode::Prepare => b'P',
            TransactionCode::Abort => b'A',
            TransactionCode::Commit => b'C',
            TransactionCode::Accept => panic!("{}", err_msg),
        }
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn build_should_return_message_p_with_id_and_fee_when_code_is_prepare() {
        let id = 0;
        let fee = 100.0;
        let message = TransactionRequest::build(TransactionCode::Prepare, id, fee);

        let mut expected = vec![b'P'];
        expected.append(&mut id.to_be_bytes().to_vec());
        expected.append(&mut fee.to_be_bytes().to_vec());

        assert_eq!(message, expected);
    }

    #[test]
    fn build_should_return_message_a_with_id_and_fee_when_code_is_abort() {
        let id = 0;
        let fee = 100.0;
        let message = TransactionRequest::build(TransactionCode::Abort, id, fee);

        let mut expected = vec![b'A'];
        expected.append(&mut id.to_be_bytes().to_vec());
        expected.append(&mut fee.to_be_bytes().to_vec());

        assert_eq!(message, expected);
    }
}
