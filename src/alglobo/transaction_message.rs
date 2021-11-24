pub struct TransactionMessage;

impl TransactionMessage {
    pub fn prepare() -> Vec<u8> {
        let mut message = Vec::new();
        message.push(b'P');
        message
    }
}

mod tests {
    use super::*;

    #[test]
    fn message_prepare_should_return_p() {
        let message = TransactionMessage::prepare();
        
        let mut expected = Vec::new();
        expected.push(b'P');

        assert_eq!(message, expected);
    }
}