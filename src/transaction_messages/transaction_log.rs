use std::{convert::TryInto, mem::size_of, collections::HashMap};

use crate::{
    alglobo::{
        transaction_state::TransactionState, 
        transaction::Transaction,
        transactionable::Transactionable
    }, 
    services::service_name::ServiceName
};

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
            (bank_state, bank_fee),
        );
        log_msg.len()
    }

    #[must_use]
    pub fn build(
        id: u64,
        airline_info: (TransactionState, f64),
        hotel_info: (TransactionState, f64),
        bank_info: (TransactionState, f64),
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

    pub fn new_transaction(message: &[u8]) -> Transaction {
        let mut begin = 1;
        let id_bytes: [u8; size_of::<u64>()] = message[begin..begin+size_of::<u64>()]
            .try_into()
            .expect("[Transaction Receiver] Los ids deberian ocupar 8 bytes");
        let id = u64::from_be_bytes(id_bytes);
        begin += size_of::<u64>();

        let services_info = HashMap::from([
            (ServiceName::Airline.string_name(),0.0),
            (ServiceName::Hotel.string_name(),0.0),
            (ServiceName::Bank.string_name(),0.0),
        ]);
        let mut transaction = Transaction::new(id, services_info.clone());

        for (name, _) in services_info.iter() {
            let state = TransactionState::from_byte(message[begin]);
            begin += 1;
            let fee_bytes: [u8; size_of::<u64>()] = message[begin..begin+size_of::<u64>()]
                .try_into()
                .expect("[Transaction Receiver] Los pagos deberian ocupar 8 bytes");
            let fee = f64::from_be_bytes(fee_bytes);
            begin += size_of::<u64>();
            match state {
                TransactionState::Waiting => transaction.wait(name.clone(), Some(fee)),
                TransactionState::Accepted => transaction.accept(name.clone(), Some(fee)),
                TransactionState::Aborted => transaction.abort(name.clone(), Some(fee)),
                TransactionState::Commited => transaction.commit(name.clone(), Some(fee)),
            };
        };
        transaction
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
            (bank_state, bank_fee),
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
            (bank_state, bank_fee),
        );

        assert_eq!(TransactionLog::size(), log_msg.len())
    }
}
