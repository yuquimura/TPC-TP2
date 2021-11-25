use std::collections::HashMap;

use super::transaction_state::TransactionState;

#[allow(dead_code)]
pub struct Transaction {
    services_info: HashMap<String, f64>,
    services_state: HashMap<TransactionState, Vec<String>>,
}

impl Transaction {
    #[must_use]
    pub fn new(services_info: HashMap<String, f64>) -> Self {
        let services_names = services_info.clone().into_keys().collect();
        let services_state = HashMap::from([(TransactionState::Waiting, services_names)]);
        Transaction {
            services_info,
            services_state,
        }
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
        let transaction = Transaction::new(HashMap::from(services.clone()));

        let mut waiting_services = transaction.waiting_services();

        services.sort_by(|a, b| a.0.cmp(&b.0));
        waiting_services.sort_by(|a, b| a.0.cmp(&b.0));

        assert_eq!(waiting_services, services);
    }
}
