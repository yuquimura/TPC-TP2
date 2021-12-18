use std::{mem::size_of, convert::TryInto, collections::HashMap};

use crate::{alglobo::transaction::Transaction, services::service_name::ServiceName};

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

    pub fn new_transaction(array: &[u8]) -> Transaction {
        let mut begin = 1;
        let id_bytes: [u8; size_of::<u64>()] = array[begin..begin+size_of::<u64>()]
            .try_into()
            .expect("[Transaction Retry] Los ids deberian ocupar 8 bytes");
        let id = u64::from_be_bytes(id_bytes);
        begin += size_of::<u64>();

        let services_names = vec![
            ServiceName::Airline.string_name(),
            ServiceName::Hotel.string_name(),
            ServiceName::Bank.string_name()
        ];
        let mut services_info = HashMap::new(); 
        
        for name in services_names {
            let fee_bytes: [u8; size_of::<f64>()] = array[begin..begin+size_of::<f64>()]
                .try_into()
                .expect("[Transaction Retry] Los pagos deberian ocupar 8 bytes");
            let fee = f64::from_be_bytes(fee_bytes);
            services_info.insert(name, fee);
            begin += size_of::<f64>();
        }
        Transaction::new(id, services_info)
    }
}

#[cfg(test)]
mod tests {
    use crate::alglobo::transactionable::Transactionable;

    use super::*;

    #[test]
    fn if_should_reconstruct_a_transaction_from_message() {
        let id = 0;
        let services_info = [
            (ServiceName::Airline.string_name(), 100.0),
            (ServiceName::Hotel.string_name(), 200.0),
            (ServiceName::Bank.string_name(), 300.0),
        ];
        let message = TransactionRetry::build(
            id, 
            services_info[0].1,
            services_info[1].1,
            services_info[2].1,
        );
        let transaction = TransactionRetry::new_transaction(&message);
        let all_services = transaction.all_services();
        assert_eq!(
            all_services,
            HashMap::from(services_info)
        );
    }
}