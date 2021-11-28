use std::collections::{HashMap, HashSet};

use super::transaction_state::TransactionState;

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
        ]);
        Transaction {
            id,
            services_info,
            services_state,
        }
    }

    #[must_use]
    pub fn get_id(&self) -> u64 {
        self.id
    }

    #[must_use]
    pub fn waiting_services(&self) -> Vec<(String, f64)> {
        let opt_service_names = self.services_state.get(&TransactionState::Waiting);
        let service_names = match opt_service_names {
            Some(vec) => vec,
            None => return vec![],
        };
        let mut res = vec![];
        for name in service_names {
            let fee = self
                .services_info
                .get(name)
                .expect("[Transaction] Nombre de servicee deberia existir");
            res.push((name.clone(), *fee));
        }
        res
    }

    pub fn accept(&mut self, name: String) -> bool {
        {
            let waiting_services = self
                .services_state
                .get_mut(&TransactionState::Waiting)
                .expect("[Transaccion] Los servicios en espera deberian existir");
            if !waiting_services.remove(&name) {
                return false;
            }
        }
        let accepted_services = self
            .services_state
            .get_mut(&TransactionState::Accepted)
            .expect("[Transaccion] Los servicios en aceptados deberian existir");
        accepted_services.insert(name);
        true
    }

    #[must_use]
    pub fn is_accepted(&self) -> bool {
        let accepted_services = self
            .services_state
            .get(&TransactionState::Accepted)
            .expect("[Transaccion] Los servicios en aceptados deberian existir");

        accepted_services.len() == self.services_info.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::services::service_name::ServiceName;

    use super::*;
    use std::collections::HashMap;

    #[test]
    fn waiting_services_should_return_waiting_services_name_and_fee() {
        let airline = (ServiceName::airline(), 100.0);
        let bank = (ServiceName::bank(), 300.0);
        let hotel = (ServiceName::hotel(), 200.0);
        let mut services = [airline, bank, hotel];
        let transaction = Transaction::new(0, HashMap::from(services.clone()));

        let mut waiting_services = transaction.waiting_services();

        services.sort_by(|a, b| a.0.cmp(&b.0));
        waiting_services.sort_by(|a, b| a.0.cmp(&b.0));

        assert_eq!(waiting_services, services);
    }

    #[test]
    fn is_accepted_should_return_false_if_any_service_is_not_accepted() {
        let airline = (ServiceName::airline(), 100.0);
        let bank = (ServiceName::bank(), 300.0);
        let hotel = (ServiceName::hotel(), 200.0);
        let services = [airline, bank, hotel];
        let mut transaction = Transaction::new(0, HashMap::from(services.clone()));

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
        let mut transaction = Transaction::new(0, HashMap::from(services.clone()));

        transaction.accept(ServiceName::airline());
        transaction.accept(ServiceName::bank());
        transaction.accept(ServiceName::hotel());

        assert!(transaction.is_accepted())
    }
}
