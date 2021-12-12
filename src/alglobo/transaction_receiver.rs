use std::collections::HashMap;
use std::convert::TryInto;
use std::mem::size_of;

use crate::alglobo::transaction_error::TransactionError;
use crate::sockets::socket_error::SocketError;
use crate::sockets::udp_socket_receiver::UdpSocketReceiver;
use crate::transactions::transaction_code::TransactionCode;
use crate::transactions::transaction_response::TransactionResponse;

use super::types::CurrentTransaction;

#[allow(dead_code)]
pub struct TransactionReceiver {
    id: u64,
    udp_receiver: Box<dyn UdpSocketReceiver + Send>,
    services_addrs: HashMap<String, String>,
    curr_transaction: CurrentTransaction,
}

impl TransactionReceiver {
    #[must_use]
    pub fn new(
        id: u64,
        udp_receiver: Box<dyn UdpSocketReceiver + Send>,
        services_addrs_str: &HashMap<&str, String>,
        curr_transaction: CurrentTransaction,
    ) -> Self {
        let services_addrs = services_addrs_str
            .iter()
            .map(|(addr, name)| ((*addr).to_string(), name.clone()))
            .collect();
        TransactionReceiver {
            id,
            udp_receiver,
            services_addrs,
            curr_transaction,
        }
    }
    
    /// # Errors
    ///
    /// `TransactionError::None` => Se recibio una transaccion,
    /// pero no existe ninguna que este siendo procesada al
    /// mismo tiempo
    /// `TransactionError::WrongId` => La transaccion
    /// recibida no es la transaccion siendo procesada
    ///
    /// # Panics
    ///
    /// Esta funcion paniquea si
    /// - Ocurrio un error irreversible en canal de lectura
    pub fn recv(&mut self) -> Result<(), TransactionError> {
        let result = self.udp_receiver.recv(TransactionResponse::size());
        let (response, addr) = match result {
            Ok(value) => value,
            Err(err) => match err {
                SocketError::Timeout => return Ok(()),
                _ => panic!("{}", err),
            },
        };
        let transaction_code = TransactionResponse::transaction_code(response[0]);
        let id_bytes: [u8; size_of::<u64>()] = response[1..]
            .try_into()
            .expect("[Transaction Receiver] Los ids deberian ocupar 8 bytes");
        let transaction_id = u64::from_be_bytes(id_bytes);
        let service_name = self
            .services_addrs
            .get(&addr)
            .expect("[Transaction Receiver] Direccion de servicio web desconocida");
        println!(
            "[Transaction Receiver] Id-Transaccion: {}, Operacion: {}, Entidad: {}",
            transaction_id, transaction_code, service_name
        );

        let mut opt_transaccion = self
            .curr_transaction
            .0
            .lock()
            .expect("[Transaction Receiver] Lock de transaccion envenenado");
        let transaction = match opt_transaccion.as_mut() {
            Some(value) => value,
            None => return Err(TransactionError::None),
        };

        if transaction_id != transaction.get_id() {
            return Err(TransactionError::WrongId);
        }

        match transaction_code {
            TransactionCode::Accept => {
                transaction.accept(service_name.to_string());
            },
            TransactionCode::Commit => {
                transaction.commit(service_name.to_string());
            }
            _ => println!("Codigo de transaccion no esperado: {}", transaction_code),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        alglobo::transactionable::MockTransactionable,
        services::service_name::ServiceName,
        sockets::udp_socket_receiver::MockUdpSocketReceiver,
        transactions::{
            transaction_code::TransactionCode, transaction_response::TransactionResponse,
        },
    };

    use std::{
        collections::HashMap,
        sync::{Arc, Condvar, Mutex},
    };

    #[test]
    fn it_should_change_transaction_service_state_to_accepted_when_recv_accept_from_it() {
        let airline = ("127.0.0.1:49156", ServiceName::Airline.string_name());
        let mut airline_clone;

        airline_clone = airline.clone();
        let services_addrs = HashMap::from([airline_clone]);

        let transaction_id = 0;
        let response = TransactionResponse::build(TransactionCode::Accept, transaction_id);

        let mut mock_socket = MockUdpSocketReceiver::new();
        airline_clone = airline.clone();
        let response_len = response.len();
        mock_socket
            .expect_recv()
            .withf(move |n_bytes| n_bytes == &response_len)
            .times(1)
            .returning(move |_| Ok((response.clone(), airline_clone.0.to_string())));

        let mut mock_transaction = MockTransactionable::new();
        mock_transaction
            .expect_get_id()
            .times(1)
            .returning(move || transaction_id);
        mock_transaction
            .expect_accept()
            .withf(move |name| name == &airline.1)
            .times(1)
            .returning(|_| true);

        let mut receiver = TransactionReceiver::new(
            0,
            Box::new(mock_socket),
            &services_addrs,
            Arc::new((Mutex::new(Some(Box::new(mock_transaction))), Condvar::new())),
        );

        assert!(receiver.recv().is_ok());
    }

    #[test]
    fn it_should_change_transaction_service_state_to_commited_when_recv_commit_from_it() {
        let airline = ("127.0.0.1:49156", ServiceName::Airline.string_name());
        let mut airline_clone;

        airline_clone = airline.clone();
        let services_addrs = HashMap::from([airline_clone]);

        let transaction_id = 0;
        let response = TransactionResponse::build(TransactionCode::Commit, transaction_id);

        let mut mock_socket = MockUdpSocketReceiver::new();
        airline_clone = airline.clone();
        let response_len = response.len();
        mock_socket
            .expect_recv()
            .withf(move |n_bytes| n_bytes == &response_len)
            .times(1)
            .returning(move |_| Ok((response.clone(), airline_clone.0.to_string())));

        let mut mock_transaction = MockTransactionable::new();
        mock_transaction
            .expect_get_id()
            .times(1)
            .returning(move || transaction_id);
        mock_transaction
            .expect_commit()
            .withf(move |name| name == &airline.1)
            .times(1)
            .returning(|_| true);

        let mut receiver = TransactionReceiver::new(
            0,
            Box::new(mock_socket),
            &services_addrs,
            Arc::new((Mutex::new(Some(Box::new(mock_transaction))), Condvar::new())),
        );

        assert!(receiver.recv().is_ok());
    }
}
