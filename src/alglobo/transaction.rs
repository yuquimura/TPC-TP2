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

    fn accept(&mut self, name: String) -> bool {
        {
            let waiting_services = self.get_mut_state_services(&TransactionState::Waiting);
            if !waiting_services.remove(&name) {
                return false;
            }
        }
        self.update_state(name, TransactionState::Accepted);
        true
    }

    fn abort(&mut self, name: String) -> bool {
        {
            let waiting_services = self.get_mut_state_services(&TransactionState::Waiting);
            if !waiting_services.remove(&name) {
                return false;
            }
        }
        self.update_state(name, TransactionState::Aborted);
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

    fn is_accepted(&self) -> bool {
        self.is_state(TransactionState::Accepted)
    }

    fn is_aborted(&self) -> bool {
        self.is_state(TransactionState::Aborted)
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
            (ServiceName::airline(), 100.0),
            (ServiceName::bank(), 300.0),
            (ServiceName::hotel(), 200.0),
        ]);
        let transaction = Transaction::new(0, services.clone());

        let waiting_services = transaction.waiting_services();

        assert_eq!(waiting_services, services);
    }

    #[test]
    fn is_accepted_should_return_false_if_any_service_is_not_accepted() {
        let airline = (ServiceName::airline(), 100.0);
        let bank = (ServiceName::bank(), 300.0);
        let hotel = (ServiceName::hotel(), 200.0);
        let services = [airline, bank, hotel];
        let mut transaction = Transaction::new(0, HashMap::from(services));

        transaction.accept(ServiceName::airline());
        transaction.accept(ServiceName::bank());

        assert!(!transaction.is_accepted())
    }

    #[test]
    fn is_accepted_should_return_true_if_all_service_are_accepted() {
        let airline = (ServiceName::airline(), 100.0);
        let bank = (ServiceName::bank(), 300.0);
        let hotel = (ServiceName::hotel(), 200.0);
        let services = [airline, bank, hotel];
        let mut transaction = Transaction::new(0, HashMap::from(services));

        transaction.accept(ServiceName::airline());
        transaction.accept(ServiceName::bank());
        transaction.accept(ServiceName::hotel());

        assert!(transaction.is_accepted())
    }
}
