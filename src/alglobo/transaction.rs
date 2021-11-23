use std::{collections::HashMap};

use super::transaction_state::TransactionState;

#[allow(dead_code)]
pub struct Transaction {
    clients_info: HashMap<String, f64>,
    clients_state: HashMap<TransactionState, Vec::<String>>
}

impl Transaction {
    #[must_use]
    pub fn new(clients_info: HashMap<String, f64>) -> Self {
        let clients_names = clients_info.clone().into_keys().collect();
        let clients_state = HashMap::from([
            (TransactionState::Waiting, clients_names)
        ]);
        Transaction {
            clients_info,
            clients_state
        }
    }

    pub fn waiting_clients(&self) -> Vec<(String, f64)> {
        let opt_client_names = self.clients_state.get(&TransactionState::Waiting);
        let client_names = match opt_client_names {
            Some(vec) => vec,
            None => return vec![]
        };
        let mut res = vec![];
        for name in client_names {
            let fee = self.clients_info.get(name).expect("[Transaction] Nombre de cliente deberia existir");
            res.push((name.clone(), fee.clone()));
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use crate::clients::client_name::ClientName;

    use super::*;
    use std::collections::HashMap;

    #[test]
    fn waiting_clients_should_return_waiting_clients_name_and_fee() {
        let airline = (ClientName::airline(), 100.0);
        let bank = (ClientName::bank(), 300.0);
        let hotel = (ClientName::hotel(), 200.0);
        let mut clients = [airline, bank, hotel];
        let transaction = Transaction::new(HashMap::from(clients.clone()));

        let mut waiting_clients = transaction.waiting_clients();

        clients.sort_by(|a, b| a.0.cmp(&b.0));
        waiting_clients.sort_by(|a, b| a.0.cmp(&b.0));

        assert_eq!(waiting_clients, clients);
        
    }
}