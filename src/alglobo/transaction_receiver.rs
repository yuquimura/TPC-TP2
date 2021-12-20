use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};

use crate::alglobo::transaction_error::TransactionError;
use crate::sockets::socket_error::SocketError;
use crate::sockets::udp_socket_receiver::UdpSocketReceiver;
use crate::transaction_messages::transaction_code::TransactionCode;
use crate::transaction_messages::transaction_info::TransactionInfo;
use crate::transaction_messages::transaction_log::TransactionLog;
use crate::transaction_messages::transaction_response::TransactionResponse;
use crate::transaction_messages::transaction_retry::TransactionRetry;
use crate::transaction_messages::types::{LOG_BYTE, RESPONSE_BYTE, RETRY_BYTE};

use super::transactionable::Transactionable;
use super::types::CurrentTransaction;

pub struct TransactionReceiver {
    udp_receiver: Box<dyn UdpSocketReceiver + Send>,
    services_addrs: HashMap<String, String>,
    curr_transaction: CurrentTransaction,
    ended: Arc<(Mutex<bool>, Condvar)>,
}

impl TransactionReceiver {
    #[must_use]
    pub fn new(
        udp_receiver: Box<dyn UdpSocketReceiver + Send>,
        services_addrs_str: &HashMap<&str, String>,
        curr_transaction: CurrentTransaction,
        ended: Arc<(Mutex<bool>, Condvar)>,
    ) -> Self {
        let services_addrs = services_addrs_str
            .iter()
            .map(|(addr, name)| ((*addr).to_string(), name.clone()))
            .collect();
        TransactionReceiver {
            udp_receiver,
            services_addrs,
            curr_transaction,
            ended,
        }
    }

    /// # Errors
    ///
    /// `TransactionError::WrongId` => La respuesta
    /// recibida corresponde a una transaccion antigua
    /// `TransactionError::None` => La tranaccion actual
    /// no existe
    pub fn process_response(
        &mut self,
        response: &[u8],
        addr: &str,
    ) -> Result<(), TransactionError> {
        let (transaction_code, transaction_id) = TransactionResponse::parse(response);
        let service_name = self
            .services_addrs
            .get(addr)
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
                transaction.accept(service_name.to_string(), None);
            }
            TransactionCode::Abort => {
                transaction.abort(service_name.to_string(), None);
            }
            TransactionCode::Commit => {
                transaction.commit(service_name.to_string(), None);
            }
            TransactionCode::Prepare => {
                println!("Codigo de transaccion no esperado: {}", transaction_code);
            }
        }
        self.curr_transaction.1.notify_all();
        Ok(())
    }

    pub fn process_log(&mut self, message: &[u8]) {
        let mut opt_transaccion = self
            .curr_transaction
            .0
            .lock()
            .expect("[Transaction Receiver] Lock de transaccion envenenado");
        let new_transaction = TransactionLog::new_transaction(message);
        let repr = new_transaction.representation(true);
        println!("[Transaction Receiver] Actualizacion: {}", repr);
        *opt_transaccion = Some(Box::new(new_transaction));
        self.curr_transaction.1.notify_all();
    }

    fn process_retry(&mut self, message: &[u8]) {
        let new_transaction = TransactionRetry::new_transaction(message);
        let repr = new_transaction.representation(false);

        let mut ended = self
            .ended
            .0
            .lock()
            .expect("[Transaction Receiver] Lock de finilizacion envenenado");
        if !*ended {
            println!(
                "[Transaction Receiver] Reintento {} DENEGADO: otra transaccion en ejecucion",
                repr
            );
            return;
        }

        let mut opt_transaction = self
            .curr_transaction
            .0
            .lock()
            .expect("[Transaction Manager] Lock de transaccion envenenado");

        if let Some(transaction) = opt_transaction.as_ref() {
            let curr_id = transaction.get_id();
            let new_id = new_transaction.get_id();
            if curr_id >= new_id {
                println!(
                    "[Transaction Receiver] Reintento {} DENEGADO: ID bajo",
                    repr
                );
                return;
            }
        }

        println!("[Transaction Receiver] Reintento {} CONCEDIDO", repr);
        *opt_transaction = Some(Box::new(new_transaction));

        *ended = false;
        self.ended.1.notify_all();
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
        let result = self.udp_receiver.recv(TransactionInfo::size());
        let (message, addr) = match result {
            Ok(value) => value,
            Err(err) => match err {
                SocketError::Timeout => return Ok(()),
                _ => panic!("{}", err),
            },
        };
        let info_type = message[0];
        let mut res: Result<(), TransactionError> = Ok(());
        match info_type {
            RESPONSE_BYTE => res = self.process_response(&message, &addr),
            LOG_BYTE => self.process_log(&message),
            RETRY_BYTE => self.process_retry(&message),
            _ => panic!("Byte de informacion desconocido"),
        };
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        alglobo::{transaction_state::TransactionState, transactionable::MockTransactionable},
        services::service_name::ServiceName,
        sockets::udp_socket_receiver::MockUdpSocketReceiver,
        transaction_messages::{
            transaction_code::TransactionCode, transaction_info::TransactionInfo,
            transaction_log::TransactionLog, transaction_response::TransactionResponse,
            transaction_retry::TransactionRetry,
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
        let mut response = TransactionResponse::build(TransactionCode::Accept, transaction_id);
        TransactionInfo::add_padding(&mut response);

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
            .withf(move |name, _| name == &airline.1)
            .times(1)
            .returning(|_, _| true);

        let mut receiver = TransactionReceiver::new(
            Box::new(mock_socket),
            &services_addrs,
            Arc::new((Mutex::new(Some(Box::new(mock_transaction))), Condvar::new())),
            Arc::new((Mutex::new(false), Condvar::new())),
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
        let mut response = TransactionResponse::build(TransactionCode::Commit, transaction_id);
        TransactionInfo::add_padding(&mut response);

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
            .withf(move |name, _| name == &airline.1)
            .times(1)
            .returning(|_, _| true);

        let mut receiver = TransactionReceiver::new(
            Box::new(mock_socket),
            &services_addrs,
            Arc::new((Mutex::new(Some(Box::new(mock_transaction))), Condvar::new())),
            Arc::new((Mutex::new(false), Condvar::new())),
        );

        assert!(receiver.recv().is_ok());
    }

    #[test]
    fn it_should_update_the_whole_transaction_when_recv_log_message() {
        let services_addrs = HashMap::from([
            ("127.0.0.1:49156", ServiceName::Airline.string_name()),
            ("127.0.0.1:49157", ServiceName::Hotel.string_name()),
            ("127.0.0.1:49158", ServiceName::Bank.string_name()),
        ]);

        let transaction_id = 0;
        let airline_info = (TransactionState::Waiting, 100.0);
        let hotel_info = (TransactionState::Accepted, 200.0);
        let bank_info = (TransactionState::Aborted, 300.0);
        let mut message =
            TransactionLog::build(transaction_id, airline_info, hotel_info, bank_info);
        TransactionInfo::add_padding(&mut message);

        let mut mock_socket = MockUdpSocketReceiver::new();

        let msg_len = message.len();
        mock_socket
            .expect_recv()
            .withf(move |n_bytes| n_bytes == &msg_len)
            .times(1)
            .returning(move |_| Ok((message.clone(), "".to_string())));

        let curr_transaction: CurrentTransaction = Arc::new((Mutex::new(None), Condvar::new()));
        let mut receiver = TransactionReceiver::new(
            Box::new(mock_socket),
            &services_addrs,
            curr_transaction.clone(),
            Arc::new((Mutex::new(false), Condvar::new())),
        );

        assert!(receiver.recv().is_ok());
        let opt_transaction = curr_transaction.0.lock().unwrap();
        let transaction = opt_transaction.as_ref().unwrap();
        let waiting_services = transaction.waiting_services();
        let accepted_services = transaction.accepted_services();
        let not_aborted_services = transaction.not_aborted_services();
        assert_eq!(
            waiting_services,
            HashMap::from([(ServiceName::Airline.string_name(), 100.0)])
        );
        assert_eq!(
            accepted_services,
            HashMap::from([(ServiceName::Hotel.string_name(), 200.0)])
        );
        assert_eq!(
            not_aborted_services,
            HashMap::from([
                (ServiceName::Airline.string_name(), 100.0),
                (ServiceName::Hotel.string_name(), 200.0),
            ])
        );
    }

    #[test]
    fn it_should_update_current_transaction_and_set_ended_false_if_retry_msg_and_ended() {
        let services_addrs = HashMap::from([
            ("127.0.0.1:49156", ServiceName::Airline.string_name()),
            ("127.0.0.1:49157", ServiceName::Hotel.string_name()),
            ("127.0.0.1:49158", ServiceName::Bank.string_name()),
        ]);

        let transaction_id = 0;
        let services_info_vec = [
            (ServiceName::Airline.string_name(), 100.0),
            (ServiceName::Hotel.string_name(), 200.0),
            (ServiceName::Bank.string_name(), 300.0),
        ];

        let mut message = TransactionRetry::build(
            transaction_id,
            services_info_vec[0].1,
            services_info_vec[1].1,
            services_info_vec[2].1,
        );
        TransactionInfo::add_padding(&mut message);

        let mut mock_socket = MockUdpSocketReceiver::new();

        let msg_len = message.len();
        mock_socket
            .expect_recv()
            .withf(move |n_bytes| n_bytes == &msg_len)
            .times(1)
            .returning(move |_| Ok((message.clone(), "".to_string())));

        let curr_transaction = Arc::new((Mutex::new(None), Condvar::new()));
        let curr_transaction_clone = curr_transaction.clone();

        let ended = Arc::new((Mutex::new(true), Condvar::new()));
        let ended_clone = ended.clone();

        let mut receiver = TransactionReceiver::new(
            Box::new(mock_socket),
            &services_addrs,
            curr_transaction_clone,
            ended_clone,
        );

        assert!(receiver.recv().is_ok());
        let opt_transaction = curr_transaction.0.lock().unwrap();
        assert!(!opt_transaction.is_none());
        let transaction = opt_transaction.as_ref().unwrap();
        let services_info = transaction.all_services();
        assert_eq!(services_info, HashMap::from(services_info_vec));
        assert!(!*ended.0.lock().unwrap());
    }

    #[test]
    fn it_should_ignore_transaction_retry_if_not_ended() {
        let services_addrs = HashMap::from([
            ("127.0.0.1:49156", ServiceName::Airline.string_name()),
            ("127.0.0.1:49157", ServiceName::Hotel.string_name()),
            ("127.0.0.1:49158", ServiceName::Bank.string_name()),
        ]);

        let transaction_id = 0;
        let services_info_vec = [
            (ServiceName::Airline.string_name(), 100.0),
            (ServiceName::Hotel.string_name(), 200.0),
            (ServiceName::Bank.string_name(), 300.0),
        ];

        let mut message = TransactionRetry::build(
            transaction_id,
            services_info_vec[0].1,
            services_info_vec[1].1,
            services_info_vec[2].1,
        );
        TransactionInfo::add_padding(&mut message);

        let mut mock_socket = MockUdpSocketReceiver::new();

        let msg_len = message.len();
        mock_socket
            .expect_recv()
            .withf(move |n_bytes| n_bytes == &msg_len)
            .times(1)
            .returning(move |_| Ok((message.clone(), "".to_string())));

        let curr_transaction = Arc::new((Mutex::new(None), Condvar::new()));
        let curr_transaction_clone = curr_transaction.clone();

        let ended = Arc::new((Mutex::new(false), Condvar::new()));
        let ended_clone = ended.clone();

        let mut receiver = TransactionReceiver::new(
            Box::new(mock_socket),
            &services_addrs,
            curr_transaction_clone,
            ended_clone,
        );

        assert!(receiver.recv().is_ok());
        let opt_transaction = curr_transaction.0.lock().unwrap();
        assert!(opt_transaction.is_none());
        assert!(!*ended.0.lock().unwrap());
    }

    #[test]
    fn it_should_ignore_transaction_if_id_is_less_than_current() {
        let services_addrs = HashMap::from([
            ("127.0.0.1:49156", ServiceName::Airline.string_name()),
            ("127.0.0.1:49157", ServiceName::Hotel.string_name()),
            ("127.0.0.1:49158", ServiceName::Bank.string_name()),
        ]);

        let transaction_id = 0;
        let services_info_vec = [
            (ServiceName::Airline.string_name(), 100.0),
            (ServiceName::Hotel.string_name(), 200.0),
            (ServiceName::Bank.string_name(), 300.0),
        ];

        let mut message = TransactionRetry::build(
            transaction_id,
            services_info_vec[0].1,
            services_info_vec[1].1,
            services_info_vec[2].1,
        );
        TransactionInfo::add_padding(&mut message);

        let mut mock_socket = MockUdpSocketReceiver::new();

        let msg_len = message.len();
        mock_socket
            .expect_recv()
            .withf(move |n_bytes| n_bytes == &msg_len)
            .times(1)
            .returning(move |_| Ok((message.clone(), "".to_string())));

        let curr_id = transaction_id + 1;
        let mut mock_transaction = MockTransactionable::new();
        mock_transaction
            .expect_get_id()
            .times(2)
            .returning(move || curr_id);

        let curr_transaction: CurrentTransaction =
            Arc::new((Mutex::new(Some(Box::new(mock_transaction))), Condvar::new()));
        let curr_transaction_clone = curr_transaction.clone();

        let ended = Arc::new((Mutex::new(true), Condvar::new()));
        let ended_clone = ended.clone();

        let mut receiver = TransactionReceiver::new(
            Box::new(mock_socket),
            &services_addrs,
            curr_transaction_clone,
            ended_clone,
        );

        assert!(receiver.recv().is_ok());
        let opt_transaction = curr_transaction.0.lock().unwrap();
        assert!(!opt_transaction.is_none());
        let transaction = opt_transaction.as_ref().unwrap();
        assert_eq!(transaction.get_id(), curr_id);
        assert!(*ended.0.lock().unwrap());
    }
}
