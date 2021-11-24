pub struct TransactionMessage;

impl TransactionMessage {
    pub fn prepare(addr: &str) -> Vec<u8> {
        let mut message = Vec::new();
        message.push(b'P');
        message.append(&mut addr.as_bytes().to_vec());
        message
    }
}

mod tests {
    use super::*;

    #[test]
    fn message_prepare_should_return_p_plus_an_address() {
        let addr = "127.0.0.1:49153";
        let message = TransactionMessage::prepare(addr);
        
        let mut expected = Vec::new();
        expected.push(b'P');
        expected.append(&mut addr.as_bytes().to_vec());

        assert_eq!(message, expected);
    }
}