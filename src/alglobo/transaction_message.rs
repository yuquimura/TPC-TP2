pub struct TransactionMessage;

impl TransactionMessage {
    #[must_use]
    pub fn prepare() -> Vec<u8> {
        vec![b'P']
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn message_prepare_should_return_p() {
        let message = TransactionMessage::prepare();

        let mut expected = Vec::new();
        expected.push(b'P');

        assert_eq!(message, expected);
    }
}
