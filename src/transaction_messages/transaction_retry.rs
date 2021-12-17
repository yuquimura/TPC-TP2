use super::types::RETRY_BYTE;

pub struct TransactionRetry;

impl TransactionRetry {
    #[must_use]
    pub fn size() -> usize {
        let id = 4000;
        let airline_fee = 100.0;
        let hotel_fee = 200.0;
        let bank_fee = 300.0;
        let msg = TransactionRetry::build(
            id,
            airline_fee,
            hotel_fee,
            bank_fee,
        );
        msg.len()
    }

    #[must_use]
    pub fn build(
        id: u64,
        airline_fee: f64,
        hotel_fee: f64,
        bank_fee: f64
    ) -> Vec<u8> {
        let mut message = vec![RETRY_BYTE];
        message.append(&mut id.to_be_bytes().to_vec());
        message.append(&mut airline_fee.to_be_bytes().to_vec());
        message.append(&mut hotel_fee.to_be_bytes().to_vec());
        message.append(&mut bank_fee.to_be_bytes().to_vec());
        message
    }
}