use std::collections::{HashMap};

use crate::{transaction_messages::{transaction_info::TransactionInfo, transaction_log::TransactionLog}, services::service_name::ServiceName};

use super::{transaction_state::TransactionState, transactionable::Transactionable};

#[allow(dead_code)]
pub struct Transaction {
    id: u64,
    services: HashMap<String, (TransactionState, f64)>
}

impl Transaction {
    #[must_use]
    pub fn new(id: u64, services_info: HashMap<String, f64>) -> Self {
        let services = services_info
            .iter()
            .map(|(name, fee)| (name.clone(), (TransactionState::Waiting, fee.clone())))
            .collect();

        Transaction {
            id,
            services
        }
    }

    fn update_state(
        &mut self, 
        name: String, 
        state: TransactionState, 
        pre_states: Vec<TransactionState>, 
        opt_fee: Option<f64>
    ) -> bool {
        let service = self
        .services
        .get_mut(&name)
        .expect("[Transaction] Nombre de servicio deberia existir");

        let is_valid ;
        if let Some(fee) = opt_fee {
            service.1 = fee;
            is_valid = true;
        } else {
            is_valid = pre_states.contains(&service.0);
        }

        if is_valid {
            service.0 = state
        }

        is_valid
    }

    fn is_state(&self, state: TransactionState) -> bool {
        for (_, (curr_state, _)) in self.services.clone() {
            if curr_state != state {
                return false
            }
        }
        true
    }
}

impl Transactionable for Transaction {
    fn get_id(&self) -> u64 {
        self.id
    }

    fn set_id(&mut self, id: u64) -> bool {
        self.id = id;
        true
    }

    fn wait(&mut self, name: String, opt_fee: Option<f64>) -> bool {
        let pre_states = vec![];
        self.update_state(
            name,
            TransactionState::Waiting,
            pre_states,
            opt_fee
        )
    }

    fn accept(&mut self, name: String, opt_fee: Option<f64>) -> bool {
        let pre_states = vec![
            TransactionState::Waiting
        ];
        self.update_state(
            name,
            TransactionState::Accepted,
            pre_states,
            opt_fee
        )
    }

    fn abort(&mut self, name: String, opt_fee: Option<f64>) -> bool {
        let pre_states = vec![
            TransactionState::Waiting,
            TransactionState::Accepted,
        ];
        self.update_state(
            name,
            TransactionState::Aborted,
            pre_states,
            opt_fee
        )
    }

    fn commit(&mut self, name: String, opt_fee: Option<f64>) -> bool {
        let pre_states = vec![
            TransactionState::Accepted,
        ];
        self.update_state(
            name,
            TransactionState::Commited,
            pre_states,
            opt_fee
        )
    }

    fn waiting_services(&self) -> HashMap<String, f64> {
        let mut result = HashMap::new();
        for (name, (state, fee)) in self.services.clone() {
            if state == TransactionState::Waiting {
                result.insert(name, fee);
            }
        }
        result
    }

    fn not_aborted_services(&self) -> HashMap<String, f64> {
        let pre_states = vec![
            TransactionState::Waiting,
            TransactionState::Accepted
        ];
        let mut result = HashMap::new();
        for (name, (state, fee)) in self.services.clone() {
            if pre_states.contains(&state) {
                result.insert(name, fee);
            }
        }
        result
    }

    fn accepted_services(&self) -> HashMap<String, f64> {
        let pre_states = vec![
            TransactionState::Accepted
        ];
        let mut result = HashMap::new();
        for (name, (state, fee)) in self.services.clone() {
            if pre_states.contains(&state) {
                result.insert(name, fee);
            }
        }
        result
    }

    fn all_services(&self) -> HashMap<String, f64> {
        let mut result = HashMap::new();
        for (name, (_, fee)) in self.services.clone() {
            result.insert(name, fee);
        }
        result
    }

    fn is_any_waiting(&self) -> bool {
        for (_, (state, _)) in self.services.clone() {
            if state == TransactionState::Waiting {
                return true
            }
        }
        false
    }

    fn is_accepted(&self) -> bool {
        self.is_state(TransactionState::Accepted)
    }

    fn is_aborted(&self) -> bool {
        self.is_state(TransactionState::Aborted)
    }

    fn is_commited(&self) -> bool {
        self.is_state(TransactionState::Commited)
    }

    fn log(&self) -> Vec<u8> {
        let airline_info = self
            .services
            .get(&ServiceName::Airline.string_name())
            .expect("[Transaction] Nombre de servicio deberia existir");
        let hotel_info = self
            .services
            .get(&ServiceName::Hotel.string_name())
            .expect("[Transaction] Nombre de servicio deberia existir");
        let bank_info = self
            .services
            .get(&ServiceName::Bank.string_name())
            .expect("[Transaction] Nombre de servicio deberia existir");
        let mut log = TransactionLog::build(
            self.id,
            airline_info.clone(),
            hotel_info.clone(),
            bank_info.clone()
        );
        TransactionInfo::add_padding(&mut log);
        log
    }
}

#[cfg(test)]
mod tests {
    use crate::services::service_name::ServiceName;

    use super::*;
    use std::collections::HashMap;

    #[test]
    fn waiting_services_should_return_waiting_services_name_and_fee() {
        let services = HashMap::from([
            (ServiceName::Airline.string_name(), 100.0),
            (ServiceName::Bank.string_name(), 300.0),
            (ServiceName::Hotel.string_name(), 200.0),
        ]);
        let transaction = Transaction::new(0, services.clone());

        let waiting_services = transaction.waiting_services();

        assert_eq!(waiting_services, services);
    }

    #[test]
    fn is_accepted_should_return_false_if_any_service_is_not_accepted() {
        let airline = (ServiceName::Airline.string_name(), 100.0);
        let bank = (ServiceName::Bank.string_name(), 300.0);
        let hotel = (ServiceName::Hotel.string_name(), 200.0);
        let services = [airline, bank, hotel];
        let mut transaction = Transaction::new(0, HashMap::from(services));

        transaction.accept(ServiceName::Airline.string_name(),None);
        transaction.accept(ServiceName::Bank.string_name(), None);

        assert!(!transaction.is_accepted())
    }

    #[test]
    fn is_accepted_should_return_true_if_all_service_are_accepted() {
        let airline = (ServiceName::Airline.string_name(), 100.0);
        let bank = (ServiceName::Bank.string_name(), 300.0);
        let hotel = (ServiceName::Hotel.string_name(), 200.0);
        let services = [airline, bank, hotel];
        let mut transaction = Transaction::new(0, HashMap::from(services));

        transaction.accept(ServiceName::Airline.string_name(), None);
        transaction.accept(ServiceName::Bank.string_name(), None);
        transaction.accept(ServiceName::Hotel.string_name(), None);

        assert!(transaction.is_accepted())
    }

    #[test]
    fn it_should_be_able_to_set_new_id() {
        let id = 0;
        let airline = (ServiceName::Airline.string_name(), 100.0);
        let bank = (ServiceName::Bank.string_name(), 300.0);
        let hotel = (ServiceName::Hotel.string_name(), 200.0);
        let services = [airline, bank, hotel];
        let mut transaction = Transaction::new(id, HashMap::from(services));

        let new_id = 1;

        transaction.set_id(new_id);
        assert_eq!(transaction.get_id(), new_id);
    }

    #[test]
    fn it_should_be_able_to_force_accept() {
        let id = 0;
        let airline = (ServiceName::Airline.string_name(), 100.0);
        let hotel = (ServiceName::Hotel.string_name(), 200.0);
        let bank = (ServiceName::Bank.string_name(), 300.0);
        let services = [airline, bank, hotel];
        let mut transaction = Transaction::new(id, HashMap::from(services));

        let new_airline_fee = 200.0;
        let new_hotel_fee = 300.0;
        let new_bank_fee = 500.0;

        transaction.accept(ServiceName::Airline.string_name(), Some(new_airline_fee));
        transaction.accept(ServiceName::Hotel.string_name(), Some(new_hotel_fee));
        transaction.accept(ServiceName::Bank.string_name(), Some(new_bank_fee));

        let all_services = transaction.all_services();
        assert_eq!(all_services.get(&ServiceName::Airline.string_name()).unwrap(), &new_airline_fee);
        assert_eq!(all_services.get(&ServiceName::Hotel.string_name()).unwrap(), &new_hotel_fee);
        assert_eq!(all_services.get(&ServiceName::Bank.string_name()).unwrap(), &new_bank_fee);

        assert!(transaction.is_accepted());
    }

    #[test]
    fn it_should_be_able_to_force_abort() {
        let id = 0;
        let airline = (ServiceName::Airline.string_name(), 100.0);
        let hotel = (ServiceName::Hotel.string_name(), 200.0);
        let bank = (ServiceName::Bank.string_name(), 300.0);
        let services = [airline, bank, hotel];
        let mut transaction = Transaction::new(id, HashMap::from(services));

        let new_airline_fee = 200.0;
        let new_hotel_fee = 300.0;
        let new_bank_fee = 500.0;

        transaction.abort(ServiceName::Airline.string_name(), Some(new_airline_fee));
        transaction.abort(ServiceName::Hotel.string_name(), Some(new_hotel_fee));
        transaction.abort(ServiceName::Bank.string_name(), Some(new_bank_fee));

        let all_services = transaction.all_services();
        assert_eq!(all_services.get(&ServiceName::Airline.string_name()).unwrap(), &new_airline_fee);
        assert_eq!(all_services.get(&ServiceName::Hotel.string_name()).unwrap(), &new_hotel_fee);
        assert_eq!(all_services.get(&ServiceName::Bank.string_name()).unwrap(), &new_bank_fee);

        assert!(transaction.is_aborted());
    }

    #[test]
    fn it_should_be_able_to_force_commit() {
        let id = 0;
        let airline = (ServiceName::Airline.string_name(), 100.0);
        let hotel = (ServiceName::Hotel.string_name(), 200.0);
        let bank = (ServiceName::Bank.string_name(), 300.0);
        let services = [airline, bank, hotel];
        let mut transaction = Transaction::new(id, HashMap::from(services));

        let new_airline_fee = 200.0;
        let new_hotel_fee = 300.0;
        let new_bank_fee = 500.0;

        transaction.commit(ServiceName::Airline.string_name(), Some(new_airline_fee));
        transaction.commit(ServiceName::Hotel.string_name(), Some(new_hotel_fee));
        transaction.commit(ServiceName::Bank.string_name(), Some(new_bank_fee));

        let all_services = transaction.all_services();
        assert_eq!(all_services.get(&ServiceName::Airline.string_name()).unwrap(), &new_airline_fee);
        assert_eq!(all_services.get(&ServiceName::Hotel.string_name()).unwrap(), &new_hotel_fee);
        assert_eq!(all_services.get(&ServiceName::Bank.string_name()).unwrap(), &new_bank_fee);

        assert!(transaction.is_commited());
    }

    #[test]
    fn it_should_be_able_to_force_wait() {
        let id = 0;
        let airline = (ServiceName::Airline.string_name(), 100.0);
        let hotel = (ServiceName::Hotel.string_name(), 200.0);
        let bank = (ServiceName::Bank.string_name(), 300.0);
        let services = [airline, bank, hotel];
        let mut transaction = Transaction::new(id, HashMap::from(services));

        let new_airline_fee = 200.0;
        let new_hotel_fee = 300.0;
        let new_bank_fee = 500.0;

        transaction.commit(ServiceName::Airline.string_name(), Some(0.0));
        transaction.commit(ServiceName::Hotel.string_name(), Some(0.0));
        transaction.commit(ServiceName::Bank.string_name(), Some(0.0));

        transaction.wait(ServiceName::Airline.string_name(), Some(new_airline_fee));
        transaction.wait(ServiceName::Hotel.string_name(), Some(new_hotel_fee));
        transaction.wait(ServiceName::Bank.string_name(), Some(new_bank_fee));

        let all_services = transaction.all_services();
        assert_eq!(all_services.get(&ServiceName::Airline.string_name()).unwrap(), &new_airline_fee);
        assert_eq!(all_services.get(&ServiceName::Hotel.string_name()).unwrap(), &new_hotel_fee);
        assert_eq!(all_services.get(&ServiceName::Bank.string_name()).unwrap(), &new_bank_fee);

        assert!(!transaction.is_accepted());
        assert!(!transaction.is_aborted());
        assert!(!transaction.is_commited());
    }
}
