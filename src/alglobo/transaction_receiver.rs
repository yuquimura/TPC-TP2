use std::collections::HashMap;
use std::convert::TryInto;
use std::mem::size_of;
use std::sync::{Condvar, Mutex, Arc};

use crate::alglobo::transaction_error::TransactionError;
use crate::services::service_name::ServiceName;
use crate::sockets::socket_error::SocketError;
use crate::sockets::udp_socket_receiver::UdpSocketReceiver;
use crate::transaction_messages::transaction_code::TransactionCode;
use crate::transaction_messages::transaction_info::TransactionInfo;
use crate::transaction_messages::transaction_response::TransactionResponse;
use crate::transaction_messages::types::{LOG_BYTE, RESPONSE_BYTE};

use super::transaction_state::TransactionState;
use super::types::CurrentTransaction;

#[allow(dead_code)]
pub struct TransactionReceiver {
    id: u64,
    udp_receiver: Box<dyn UdpSocketReceiver + Send>,
    services_addrs: HashMap<String, String>,
    curr_transaction: CurrentTransaction,
    is_eof: Arc<(Mutex<bool>, Condvar)>,
}

impl TransactionReceiver {
    #[must_use]
    pub fn new(
        id: u64,
        udp_receiver: Box<dyn UdpSocketReceiver + Send>,
        services_addrs_str: &HashMap<&str, String>,
        curr_transaction: CurrentTransaction,
        is_eof: Arc<(Mutex<bool>, Condvar)>,
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
            is_eof
        }
    }

    pub fn process_response(
        &mut self,
        response: Vec<u8>,
        addr: String,
    ) -> Result<(), TransactionError> {
        let transaction_code = TransactionResponse::transaction_code(response[1]);
        let id_bytes: [u8; size_of::<u64>()] = response[2..2 + size_of::<u64>()]
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
                transaction.accept(service_name.to_string(), None);
            }
            TransactionCode::Abort => {
                transaction.abort(service_name.to_string(), None);
            }
            TransactionCode::Commit => {
                transaction.commit(service_name.to_string(), None);
            }
            _ => println!("Codigo de transaccion no esperado: {}", transaction_code),
        }
        self.curr_transaction.1.notify_all();
        Ok(())
    }

    pub fn update_transaction_state(
        &mut self,
        message: &[u8],
        service_name: String,
        idx_state: usize,
        idx_fee: usize,
    ) -> Result<(), TransactionError> {
        let mut opt_transaccion = self
            .curr_transaction
            .0
            .lock()
            .expect("[Transaction Receiver] Lock de transaccion envenenado");
        let transaction = match opt_transaccion.as_mut() {
            Some(value) => value,
            None => return Err(TransactionError::None),
        };

        let state = TransactionState::from_byte(message[idx_state]);
        let fee_bytes: [u8; size_of::<f64>()] = message[idx_fee..idx_fee + size_of::<f64>()]
            .try_into()
            .expect("[Transaction Receiver] Los pagos deberian ocupar 8 bytes");
        let fee = f64::from_be_bytes(fee_bytes);

        println!(
            "[Transaction Receiver] Actualizar transaccion del servicioo {}",
            service_name
        );
        println!(
            "[Transaction Receiver] Actualizar transaccion con estado {}",
            state
        );
        println!(
            "[Transaction Receiver] Actualizar transaccion con pago {}",
            fee
        );

        match state {
            TransactionState::Waiting => transaction.wait(service_name, Some(fee)),
            TransactionState::Accepted => transaction.accept(service_name, Some(fee)),
            TransactionState::Aborted => transaction.abort(service_name, Some(fee)),
            TransactionState::Commited => transaction.commit(service_name, Some(fee)),
        };
        Ok(())
    }

    pub fn process_log(&mut self, message: Vec<u8>) -> Result<(), TransactionError> {
        const IDX_ID: usize = 1;
        const IDX_AIRLINE_STATE: usize = IDX_ID + size_of::<u64>();
        const IDX_AIRLINE_FEE: usize = IDX_AIRLINE_STATE + 1;

        const IDX_HOTEL_STATE: usize = IDX_AIRLINE_FEE + size_of::<u64>();
        const IDX_HOTEL_FEE: usize = IDX_HOTEL_STATE + 1;

        const IDX_BANK_STATE: usize = IDX_HOTEL_FEE + size_of::<u64>();
        const IDX_BANK_FEE: usize = IDX_BANK_STATE + 1;
        {
            let mut opt_transaccion = self
                .curr_transaction
                .0
                .lock()
                .expect("[Transaction Receiver] Lock de transaccion envenenado");
            let transaction = match opt_transaccion.as_mut() {
                Some(value) => value,
                None => return Err(TransactionError::None),
            };

            let id_bytes: [u8; size_of::<u64>()] = message[IDX_ID..IDX_AIRLINE_STATE]
                .try_into()
                .expect("[Transaction Receiver] Los ids deberian ocupar 8 bytes");
            let transaction_id = u64::from_be_bytes(id_bytes);
            println!(
                "[Transaction Receiver] Actualizar transaccion de id {}",
                transaction_id
            );
            transaction.set_id(transaction_id);
        }

        self.update_transaction_state(
            &message,
            ServiceName::Airline.string_name(),
            IDX_AIRLINE_STATE,
            IDX_AIRLINE_FEE,
        )
        .expect("[Transaction Receiver] Actualiza la transaccion no deberia fallar");

        self.update_transaction_state(
            &message,
            ServiceName::Hotel.string_name(),
            IDX_HOTEL_STATE,
            IDX_HOTEL_FEE,
        )
        .expect("[Transaction Receiver] Actualiza la transaccion no deberia fallar");

        self.update_transaction_state(
            &message,
            ServiceName::Bank.string_name(),
            IDX_BANK_STATE,
            IDX_BANK_FEE,
        )
        .expect("[Transaction Receiver] Actualiza la transaccion no deberia fallar");
        self.curr_transaction.1.notify_all();
        Ok(())
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
        match info_type {
            RESPONSE_BYTE => self.process_response(message, addr),
            LOG_BYTE => self.process_log(message),
            _ => panic!("Byte de informacion desconocido"),
        }
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
            0,
            Box::new(mock_socket),
            &services_addrs,
            Arc::new((Mutex::new(Some(Box::new(mock_transaction))), Condvar::new())),
            Arc::new((Mutex::new(false), Condvar::new()))
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
            0,
            Box::new(mock_socket),
            &services_addrs,
            Arc::new((Mutex::new(Some(Box::new(mock_transaction))), Condvar::new())),
            Arc::new((Mutex::new(false), Condvar::new()))
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

        let mut mock_transaction = MockTransactionable::new();
        mock_transaction
            .expect_set_id()
            .withf(move |id| id == &transaction_id)
            .times(1)
            .returning(move |_| true);
        mock_transaction
            .expect_wait()
            .withf(move |name, opt_fee| {
                name == &ServiceName::Airline.string_name() && opt_fee.unwrap() == airline_info.1
            })
            .times(1)
            .returning(|_, _| true);
        mock_transaction
            .expect_accept()
            .withf(move |name, opt_fee| {
                name == &ServiceName::Hotel.string_name() && opt_fee.unwrap() == hotel_info.1
            })
            .times(1)
            .returning(|_, _| true);
        mock_transaction
            .expect_abort()
            .withf(move |name, opt_fee| {
                name == &ServiceName::Bank.string_name() && opt_fee.unwrap() == bank_info.1
            })
            .times(1)
            .returning(|_, _| true);

        let mut receiver = TransactionReceiver::new(
            0,
            Box::new(mock_socket),
            &services_addrs,
            Arc::new((Mutex::new(Some(Box::new(mock_transaction))), Condvar::new())),
            Arc::new((Mutex::new(false), Condvar::new()))
        );

        assert!(receiver.recv().is_ok());
    }
}
