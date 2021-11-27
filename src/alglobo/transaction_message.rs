pub struct TransactionMessage;

impl TransactionMessage {
    #[must_use]
    pub fn prepare(id: u64, fee: f64) -> Vec<u8> {
        let mut message = vec![b'P'];
        TransactionMessage::add_id_and_fee_to_message(&mut message, id, fee);
        message
    }

    #[must_use]
    pub fn abort(id: u64, fee: f64) -> Vec<u8> {
        let mut message = vec![b'A'];
        TransactionMessage::add_id_and_fee_to_message(&mut message, id, fee);
        message
    }

    fn add_id_and_fee_to_message(message: &mut Vec<u8>, id: u64, fee: f64) {
        message.append(&mut id.to_be_bytes().to_vec());
        message.append(&mut fee.to_be_bytes().to_vec());
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn prepare_should_return_message_p_with_id_and_fee() {
        let id = 0;
        let fee = 100.0;
        let message = TransactionMessage::prepare(id, fee);

        let mut expected = vec![b'P'];
        expected.append(&mut id.to_be_bytes().to_vec());
        expected.append(&mut fee.to_be_bytes().to_vec());

        assert_eq!(message, expected);
    }

    #[test]
    fn abort_should_return_a_with_id_and_fee() {
        let id = 0;
        let fee = 100.0;
        let message = TransactionMessage::abort(id, fee);

        let mut expected = vec![b'A'];
        expected.append(&mut id.to_be_bytes().to_vec());
        expected.append(&mut fee.to_be_bytes().to_vec());

        assert_eq!(message, expected);
    }
}
