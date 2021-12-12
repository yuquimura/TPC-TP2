use std::collections::{HashMap, HashSet};

use super::{transaction_state::TransactionState, transactionable::Transactionable};

#[allow(dead_code)]
pub struct Transaction {
    id: u64,
    services_info: HashMap<String, f64>,
    services_state: HashMap<TransactionState, HashSet<String>>,
}

impl Transaction {
    #[must_use]
    pub fn new(id: u64, services_info: HashMap<String, f64>) -> Self {
        let services_names: HashSet<String> = services_info.keys().cloned().collect();
        let services_state = HashMap::from([
            (TransactionState::Waiting, services_names),
            (TransactionState::Accepted, HashSet::new()),
            (TransactionState::Aborted, HashSet::new()),
            (TransactionState::Commited, HashSet::new()),
        ]);
        Transaction {
            id,
            services_info,
            services_state,
        }
    }

    fn update_state(&mut self, name: String, state: TransactionState) {
        let accepted_services = self.get_mut_state_services(&state);
        accepted_services.insert(name);
    }

    fn is_state(&self, state: TransactionState) -> bool {
        let services = self.get_state_services(&state);
        services.len() == self.services_info.len()
    }

    fn get_state_services(&self, state: &TransactionState) -> &HashSet<String> {
        let err_msg = format!("[Transaccion] Los servicios {} deberian existir", state);
        self.services_state.get(state).expect(&err_msg)
    }

    fn get_mut_state_services(&mut self, state: &TransactionState) -> &mut HashSet<String> {
        let err_msg = format!("[Transaccion] Los servicios {} deberian existir", state);
        self.services_state.get_mut(state).expect(&err_msg)
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

    fn wait(&mut self, name: String, _opt_fee: Option<f64>) -> bool {
        {
            let waiting_services = self.get_mut_state_services(&TransactionState::Waiting);
            if !waiting_services.remove(&name) {
                return false;
            }
        }
        self.update_state(name, TransactionState::Accepted);
        true
    }

    fn accept(&mut self, name: String, _opt_fee: Option<f64>) -> bool {
        {
            let waiting_services = self.get_mut_state_services(&TransactionState::Waiting);
            if !waiting_services.remove(&name) {
                return false;
            }
        }
        self.update_state(name, TransactionState::Accepted);
        true
    }

    fn abort(&mut self, name: String, _opt_fee: Option<f64>) -> bool {
        let mut is_valid = false;
        {
            let waiting_services = self.get_mut_state_services(&TransactionState::Waiting);
            if waiting_services.remove(&name) {
                is_valid = true;
            }
        }
        {
            let accepted_services = self.get_mut_state_services(&TransactionState::Accepted);
            if accepted_services.remove(&name) {
                is_valid = true;
            }
        }
        if is_valid {
            self.update_state(name, TransactionState::Aborted);
        }
        is_valid
    }

    fn commit(&mut self, name: String, _opt_fee: Option<f64>) -> bool {
        {
            let accepted_services = self.get_mut_state_services(&TransactionState::Accepted);
            if !accepted_services.remove(&name) {
                return false;
            }
        }
        self.update_state(name, TransactionState::Commited);
        true
    }

    fn waiting_services(&self) -> HashMap<String, f64> {
        let waiting_services = self.get_state_services(&TransactionState::Waiting);

        let mut res = HashMap::new();
        for name in waiting_services.iter() {
            let fee = self
                .services_info
                .get(name)
                .expect("[Transaction] Nombre de servicee deberia existir");
            res.insert(name.clone(), *fee);
        }
        res
    }

    fn all_services(&self) -> HashMap<String, f64> {
        self.services_info.clone()
    }

    fn is_any_waiting(&self) -> bool {
        let services = self.get_state_services(&TransactionState::Waiting);
        services.len() > 0
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
        // let new_airline = (ServiceName::Airline.string_name(), 100.0);
        // let new_bank = (ServiceName::Bank.string_name(), 300.0);
        // let new_hotel = (ServiceName::Hotel.string_name(), 200.0);

        transaction.set_id(new_id);
        assert_eq!(transaction.get_id(), new_id);
    }

    // #[test]
    // fn it_should_be_able_to_force_accept() {
    //     let id = 0;
    //     let airline = (ServiceName::Airline.string_name(), 100.0);
    //     let hotel = (ServiceName::Hotel.string_name(), 200.0);
    //     let bank = (ServiceName::Bank.string_name(), 300.0);
    //     let services = [airline, bank, hotel];
    //     let mut transaction = Transaction::new(id, HashMap::from(services));

    //     let new_airline_fee = 200.0;
    //     let new_hotel_fee = 300.0;
    //     let new_bank_fee = 500.0;

    //     transaction.accept(ServiceName::Airline.string_name(), Some(new_airline_fee));
    //     transaction.accept(ServiceName::Hotel.string_name(), Some(new_hotel_fee));
    //     transaction.accept(ServiceName::Bank.string_name(), Some(new_bank_fee));

    //     let all_services = transaction.all_services();
    //     assert_eq!(all_services.get(&ServiceName::Airline.string_name()).unwrap(), &new_airline_fee);
    //     assert_eq!(all_services.get(&ServiceName::Hotel.string_name()).unwrap(), &new_hotel_fee);
    //     assert_eq!(all_services.get(&ServiceName::Bank.string_name()).unwrap(), &new_bank_fee);

    //     assert!(transaction.is_accepted());
    // }
}
