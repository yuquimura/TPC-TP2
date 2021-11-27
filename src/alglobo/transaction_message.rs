pub struct TransactionMessage;

impl TransactionMessage {
    #[must_use]
    pub fn prepare(id: u64, fee: f64) -> Vec<u8> {
        let mut message = vec![b'P'];
        message.append(&mut id.to_be_bytes().to_vec());
        message.append(&mut fee.to_be_bytes().to_vec());
        message
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn message_prepare_should_return_p_with_id_and_fee() {
        let id = 0;
        let fee = 100.0;
        let message = TransactionMessage::prepare(id, fee);

        let mut expected = vec![b'P'];
        expected.append(&mut id.to_be_bytes().to_vec());
        expected.append(&mut fee.to_be_bytes().to_vec());

        assert_eq!(message, expected);
    }
}
