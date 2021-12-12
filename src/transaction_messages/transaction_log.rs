use crate::{alglobo::transaction_state::TransactionState};

use super::types::LOG_BYTE;

pub struct TransactionLog;

impl TransactionLog {
    #[must_use]
    pub fn size() -> usize {
        let id = 4000;
        let airline_state = TransactionState::Waiting;
        let airline_fee = 100.0;
        let hotel_state = TransactionState::Accepted;
        let hotel_fee = 200.0;
        let bank_state = TransactionState::Aborted;
        let bank_fee = 300.0;
        let log_msg = TransactionLog::build(
            id,
            (airline_state, airline_fee),
            (hotel_state, hotel_fee),
            (bank_state, bank_fee)
        );
        log_msg.len()
    }


    #[must_use]
    pub fn build(
        id: u64,
        airline_info: (TransactionState, f64),
        hotel_info: (TransactionState, f64), 
        bank_info: (TransactionState, f64)
    ) -> Vec<u8> {
        let mut message = vec![LOG_BYTE];
        message.append(&mut id.to_be_bytes().to_vec());
        
        message.push(airline_info.0.byte_code());
        message.append(&mut airline_info.1.to_be_bytes().to_vec());

        message.push(hotel_info.0.byte_code());
        message.append(&mut hotel_info.1.to_be_bytes().to_vec());

        message.push(bank_info.0.byte_code());
        message.append(&mut bank_info.1.to_be_bytes().to_vec());

        message
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_should_return_array_with_id_entities_states_and_fees() {
        let id = 4000;
        let airline_state = TransactionState::Waiting;
        let airline_fee = 100.0;
        let hotel_state = TransactionState::Accepted;
        let hotel_fee = 200.0;
        let bank_state = TransactionState::Aborted;
        let bank_fee = 300.0;
        let log_msg = TransactionLog::build(
            id,
            (airline_state, airline_fee),
            (hotel_state, hotel_fee),
            (bank_state, bank_fee)
        );

        let mut expected = vec![LOG_BYTE];
        expected.append(&mut id.to_be_bytes().to_vec());
        expected.push(airline_state.byte_code());
        expected.append(&mut airline_fee.to_be_bytes().to_vec());
        expected.push(hotel_state.byte_code());
        expected.append(&mut hotel_fee.to_be_bytes().to_vec());
        expected.push(bank_state.byte_code());
        expected.append(&mut bank_fee.to_be_bytes().to_vec());

        assert_eq!(log_msg, expected);
    }

    #[test]
    fn size_should_be_the_len_of_result_of_build() {

        let id = 4000;
        let airline_state = TransactionState::Waiting;
        let airline_fee = 100.0;
        let hotel_state = TransactionState::Accepted;
        let hotel_fee = 200.0;
        let bank_state = TransactionState::Aborted;
        let bank_fee = 300.0;
        let log_msg = TransactionLog::build(
            id,
            (airline_state, airline_fee),
            (hotel_state, hotel_fee),
            (bank_state, bank_fee)
        );

        assert_eq!(TransactionLog::size(), log_msg.len())
    }

}