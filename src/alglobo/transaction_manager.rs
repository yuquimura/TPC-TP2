use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, RwLock};
use std::thread::sleep;
use std::{collections::HashMap, time::Duration};

use crate::candidates::constants::{DEFAULT_IP, END_TIMEOUT, SLEEP_MANAGER};
use crate::file_reader::file_iterator::FileIterator;
use crate::{
    sockets::udp_socket_sender::UdpSocketSender,
    transaction_messages::{
        transaction_code::TransactionCode, transaction_request::TransactionRequest,
    },
};

use super::{
    transaction::Transaction, transaction_error::TransactionError,
    transactionable::Transactionable, types::CurrentTransaction,
};

pub struct TransactionManager {
    my_port: u64,
    udp_sender: Box<dyn UdpSocketSender + Send>,
    curr_transaction: CurrentTransaction,
    ended: Arc<(Mutex<bool>, Condvar)>,
    services_addrs: HashMap<String, String>,
    replicas_addrs: Vec<String>,
    timeout: Duration,
    abort_file_opt: Option<File>,
}

impl TransactionManager {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        my_port: u64,
        udp_sender: Box<dyn UdpSocketSender + Send>,
        curr_transaction: CurrentTransaction,
        ended: Arc<(Mutex<bool>, Condvar)>,
        services_addrs_str: &HashMap<&str, String>,
        replicas_addrs_str: &[String],
        timeout: Duration,
        path_opt: Option<String>,
    ) -> Self {
        let services_addrs = services_addrs_str
            .clone()
            .iter()
            .map(|(addr, name)| (name.clone(), (*addr).to_string()))
            .collect();
        let replicas_addrs = replicas_addrs_str
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        let mut abort_file_opt = None;
        if let Some(path) = path_opt {
            let mut options = OpenOptions::new();
            abort_file_opt = Some(
                options
                    .create(true)
                    .append(true)
                    .open(path)
                    .expect("[Transaction Manager] Error al abrir archivo de fallas"),
            );
        }

        TransactionManager {
            my_port,
            udp_sender,
            curr_transaction,
            ended,
            services_addrs,
            replicas_addrs,
            timeout,
            abort_file_opt,
        }
    }

    pub fn process(&mut self, opt_transaction: Option<Transaction>) -> u64 {
        if let Some(transaction) = opt_transaction {
            self.update_current(transaction);
        }
        if self.prepare() {
            // Seguir commiteando hasta que
            // todos los servicios respondan
            while !self.commit() {}
        } else {
            self.abort();
            self.persist_aborted();
        }

        let opt_transaction = self.get_current();
        let transaction = opt_transaction
            .as_ref()
            .expect("[Transaction Manager] La transaccion actual deberia exitir");
        transaction.get_id()
    }

    pub fn update_current(&mut self, transaction: Transaction) {
        let mut opt_transaction = self
            .curr_transaction
            .0
            .lock()
            .expect("[Transaction Manager] Lock de transaccion envenenado");
        *opt_transaction = Some(Box::new(transaction));
    }

    pub fn prepare(&mut self) -> bool {
        let transaction_id;
        let waiting_services;
        {
            let opt_transaction = self.get_current();
            let transaction = opt_transaction
                .as_ref()
                .expect("[Transaction Manager] La transaccion actual deberia exitir");
            transaction_id = transaction.get_id();
            waiting_services = transaction.waiting_services();
        }
        self.send_messages(TransactionCode::Prepare, transaction_id, waiting_services);
        let _ = self.wait_update(|opt_transaction| {
            opt_transaction
                .as_ref()
                .expect("[Transaction Manager] La transaccion actual deberia existir")
                .is_any_waiting()
        });
        self.send_transaction_logs();
        let opt_transaction = self.get_current();
        let transaction = opt_transaction
            .as_ref()
            .expect("[Transaction Manager] La transaccion actual deberia exitir");
        transaction.is_accepted() || transaction.is_commited()
    }

    pub fn abort(&mut self) -> bool {
        let transaction_id;
        let all_services;
        {
            let opt_transaction = self.get_current();
            let transaction = opt_transaction
                .as_ref()
                .expect("[Transaction Manager] La transaccion actual deberia exitir");
            transaction_id = transaction.get_id();
            all_services = transaction.not_aborted_services();
        }
        self.send_messages(TransactionCode::Abort, transaction_id, all_services);
        let _ = self.wait_update(|opt_transaction| {
            !opt_transaction
                .as_ref()
                .expect("[Transaction Manager] La transaccion actual deberia existir")
                .is_aborted()
        });
        self.send_transaction_logs();
        let opt_transaction = self.get_current();
        let transaction = opt_transaction
            .as_ref()
            .expect("[Transaction Manager] La transaccion actual deberia exitir");
        transaction.is_aborted()
    }

    pub fn commit(&mut self) -> bool {
        let transaction_id;
        let all_services;
        {
            let opt_transaction = self.get_current();
            let transaction = opt_transaction
                .as_ref()
                .expect("[Transaction Manager] La transaccion actual deberia exitir");
            transaction_id = transaction.get_id();
            all_services = transaction.accepted_services();
        }
        self.send_messages(TransactionCode::Commit, transaction_id, all_services);
        let _ = self.wait_update(|opt_transaction| {
            !opt_transaction
                .as_ref()
                .expect("[Transaction Manager] La transacci\u{f3}n actual deberia existir")
                .is_commited()
        });
        self.send_transaction_logs();
        let opt_transaction = self.get_current();
        let transaction = opt_transaction
            .as_ref()
            .expect("[Transaction Manager] La transaccion actual deberia exitir");
        transaction.is_commited()
    }

    fn wait_update(
        &self,
        condition: fn(&mut Option<Box<dyn Transactionable + Send>>) -> bool,
    ) -> Result<(), TransactionError> {
        let err_msg = "[Transaction Manager] Lock de transaccion envenenado";
        let res = self
            .curr_transaction
            .1
            .wait_timeout_while(
                self.curr_transaction.0.lock().expect(err_msg),
                self.timeout,
                condition,
            )
            .expect(err_msg);
        if res.1.timed_out() {
            return Err(TransactionError::Timeout);
        }
        Ok(())
    }

    fn send_messages(
        &mut self,
        code: TransactionCode,
        id: u64,
        services_info: HashMap<String, f64>,
    ) {
        for (name, fee) in services_info {
            let addr = self.services_addrs.get(&name).expect(
                "[Transaction Manager] La direcci\u{f3}n IP del servicio web deberia existir",
            );
            println!(
                "[Transaction Manager] Transaccion: {} - Entidad: {} - Operacion: {}",
                id, name, code
            );

            self.udp_sender
                .send_to(&TransactionRequest::build(code, id, fee), addr)
                .expect(
                    "[Transaction Manager] Enviar mensaje de transacci\u{f3}n no deberia fallar",
                );
        }
    }

    fn send_transaction_logs(&mut self) {
        let transaction_log;
        let transaction_id;
        {
            let opt_transaction = self.get_current();
            let transaction = opt_transaction
                .as_ref()
                .expect("[Transaction Manager] La transaccion actual deberia exitir");
            transaction_log = transaction.log();
            transaction_id = transaction.get_id();
        }
        let my_addr = DEFAULT_IP.to_string() + self.my_port.to_string().as_str();
        for addr in self.replicas_addrs.clone() {
            if addr == my_addr {
                continue;
            }
            println!(
                "[Transaction Manager] Log Transaccion: {} - Addr: {}",
                transaction_id, addr
            );
            self.udp_sender
                .send_to(&transaction_log, &addr)
                .expect("[Transaction Manager] Enviar mensaje de log no deberia fallar");
        }
    }

    fn persist_aborted(&mut self) {
        if let Some(abort_file) = &mut self.abort_file_opt {
            let opt_transaction = self
                .curr_transaction
                .0
                .lock()
                .expect("[Transaction Manager] Lock de transaccion envenenado");
            let transaction = opt_transaction
                .as_ref()
                .expect("[Transaction Manager] La transaccion actual deberia exitir");
            let mut representation = transaction.representation(false);
            representation.push('\n');
            abort_file
                .write_all(representation.as_bytes())
                .expect("[Transaction Manager] Persistir transaccion abortada no deberia fallar");
            println!("[Transaction Manager] Transaccion abortada persistida");
        }
    }

    fn get_current(&self) -> MutexGuard<'_, Option<Box<(dyn Transactionable + Send + 'static)>>> {
        self.curr_transaction
            .0
            .lock()
            .expect("[Transaction Manager] Lock de transaccion envenenado")
    }

    #[allow(clippy::mutex_atomic)]
    fn wait_end_while(&mut self, dur: Duration) -> Result<(), TransactionError> {
        let lock_err_msg = "[Transaction Manager] Lock de espera de finalizacion envenenado";
        {
            let mut ended = self.ended.0.lock().expect(lock_err_msg);
            *ended = true;
        }
        println!(
            "[Transaction Manager] Esperando {:#?} antes de finalizar",
            dur
        );
        let res = self
            .ended
            .1
            .wait_timeout_while(self.ended.0.lock().expect(lock_err_msg), dur, |ended| {
                *ended
            })
            .expect(lock_err_msg);
        if res.1.timed_out() {
            return Err(TransactionError::Timeout);
        }
        Ok(())
    }

    pub fn run(&mut self, path: &str, finish_lock: &Arc<RwLock<bool>>) {
        let start_line = self.process(None);
        if let Ok(mut reader) = FileIterator::new(path) {
            while !reader.ended() {
                if let Some(transaction) = reader.next() {
                    if transaction.get_id() > start_line {
                        sleep(SLEEP_MANAGER);
                        self.process(Some(transaction));
                    }
                }
            }
        }

        while self.wait_end_while(END_TIMEOUT).is_ok() {
            println!("[Transaction Manager] Reintentando transaccion");
            self.process(None);
        }

        let mut finish = finish_lock
            .write()
            .expect("[Transaction Manager] Lock de fin envenenado");
        *finish = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        alglobo::{transaction_receiver::TransactionReceiver, transaction_state::TransactionState},
        services::service_name::ServiceName,
        sockets::{
            socket_error::SocketError, udp_socket_receiver::MockUdpSocketReceiver,
            udp_socket_sender::MockUdpSocketSender,
        },
        transaction_messages::{
            transaction_info::TransactionInfo, transaction_log::TransactionLog,
            transaction_response::TransactionResponse,
        },
    };

    use std::{
        collections::HashMap,
        sync::{Arc, Condvar, Mutex},
        thread,
    };

    #[test]
    fn process_transaction_should_send_msg_prepare_to_all_services_in_transaction() {
        let id = 0;

        let airline_addr = "127.0.0.1:49156";
        let hotel_addr = "127.0.0.1:49157";
        let bank_addr = "127.0.0.1:49158";
        let services_addrs_str = &HashMap::from([
            (airline_addr, ServiceName::Airline.string_name()),
            (hotel_addr, ServiceName::Hotel.string_name()),
            (bank_addr, ServiceName::Bank.string_name()),
        ]);

        let transaction_id = 0;
        let airline_fee = 100.0;
        let hotel_fee = 200.0;
        let bank_fee = 300.0;
        let transaction = Transaction::new(
            transaction_id,
            &HashMap::from([
                (ServiceName::Airline.string_name(), airline_fee),
                (ServiceName::Hotel.string_name(), hotel_fee),
                (ServiceName::Bank.string_name(), bank_fee),
            ]),
        );
        let curr_transaction = Arc::new((Mutex::new(None), Condvar::new()));

        let waiting_services = transaction.waiting_services();

        let mut mock_sender = MockUdpSocketSender::new();
        //let mut mock_receiver = MockUdpSocketReceiver::new();

        let addresses = [airline_addr, hotel_addr, bank_addr];

        let transaction_messages = [
            TransactionRequest::build(TransactionCode::Prepare, transaction_id, airline_fee),
            TransactionRequest::build(TransactionCode::Prepare, transaction_id, hotel_fee),
            TransactionRequest::build(TransactionCode::Prepare, transaction_id, bank_fee),
        ];

        let addresses_clone = addresses;
        let messages_clone = transaction_messages;
        mock_sender
            .expect_send_to()
            .withf(move |buf, addr| {
                messages_clone.contains(&buf.to_vec()) && addresses_clone.contains(&addr)
            })
            .times(waiting_services.len())
            .returning(|_, _| Ok(()));

        let mut manager = TransactionManager::new(
            id,
            Box::new(mock_sender),
            curr_transaction.clone(),
            Arc::new((Mutex::new(false), Condvar::new())),
            &services_addrs_str,
            &vec![],
            Duration::from_secs(0),
            None,
        );

        manager.update_current(transaction);
        manager.prepare();
    }

    #[test]
    fn process_transaction_should_send_msg_abort_to_all_services_if_any_service_does_not_respond_to_prepare_msg(
    ) {
        let id = 0;

        let airline_addr = "127.0.0.1:49156";
        let hotel_addr = "127.0.0.1:49157";
        let bank_addr = "127.0.0.1:49158";
        let services_addrs_str = &HashMap::from([
            (airline_addr, ServiceName::Airline.string_name()),
            (hotel_addr, ServiceName::Hotel.string_name()),
            (bank_addr, ServiceName::Bank.string_name()),
        ]);

        let transaction_id = 1;
        let airline_fee = 100.0;
        let hotel_fee = 200.0;
        let bank_fee = 300.0;
        let transaction = Transaction::new(
            transaction_id,
            &HashMap::from([
                (ServiceName::Airline.string_name(), airline_fee),
                (ServiceName::Hotel.string_name(), hotel_fee),
                (ServiceName::Bank.string_name(), bank_fee),
            ]),
        );
        let curr_transaction = Arc::new((Mutex::new(None), Condvar::new()));

        let waiting_services = transaction.waiting_services();

        let mut mock_sender = MockUdpSocketSender::new();
        let mock_receiver = MockUdpSocketReceiver::new();

        mock_sender
            .expect_send_to()
            .withf(move |_, _| true)
            .times(waiting_services.len())
            .returning(|_, _| Ok(()));

        let addresses = [airline_addr, hotel_addr, bank_addr];

        let abort_messages = [
            TransactionRequest::build(TransactionCode::Abort, transaction_id, airline_fee),
            TransactionRequest::build(TransactionCode::Abort, transaction_id, hotel_fee),
            TransactionRequest::build(TransactionCode::Abort, transaction_id, bank_fee),
        ];

        let addresses_clone = addresses;
        let messages_clone = abort_messages;
        mock_sender
            .expect_send_to()
            .withf(move |buf, addr| {
                messages_clone.contains(&buf.to_vec()) && addresses_clone.contains(&addr)
            })
            .times(waiting_services.len())
            .returning(|_, _| Ok(()));

        let mut receiver = TransactionReceiver::new(
            Box::new(mock_receiver),
            &services_addrs_str,
            curr_transaction.clone(),
            Arc::new((Mutex::new(false), Condvar::new())),
        );

        thread::spawn(move || loop {
            let _drop = receiver.recv();
        });

        let mut manager = TransactionManager::new(
            id,
            Box::new(mock_sender),
            curr_transaction.clone(),
            Arc::new((Mutex::new(false), Condvar::new())),
            &services_addrs_str,
            &vec![],
            Duration::from_secs(0),
            None,
        );

        manager.update_current(transaction);
        manager.prepare();
        manager.abort();
    }

    #[test]
    fn process_transaction_should_send_msg_commit_to_all_services_if_all_services_responded_with_accept_msg(
    ) {
        let id = 0;

        let airline_addr = "127.0.0.1:49156";
        let hotel_addr = "127.0.0.1:49157";
        let bank_addr = "127.0.0.1:49158";
        let addresses = [airline_addr, hotel_addr, bank_addr];
        let services_addrs_str = &HashMap::from([
            (airline_addr, ServiceName::Airline.string_name()),
            (hotel_addr, ServiceName::Hotel.string_name()),
            (bank_addr, ServiceName::Bank.string_name()),
        ]);

        let transaction_id = 1;
        let airline_fee = 100.0;
        let hotel_fee = 200.0;
        let bank_fee = 300.0;
        let transaction = Transaction::new(
            transaction_id,
            &HashMap::from([
                (ServiceName::Airline.string_name(), airline_fee),
                (ServiceName::Hotel.string_name(), hotel_fee),
                (ServiceName::Bank.string_name(), bank_fee),
            ]),
        );
        let curr_transaction = Arc::new((Mutex::new(None), Condvar::new()));

        let n_services = transaction.waiting_services().len();

        let mut mock_sender = MockUdpSocketSender::new();
        let mut mock_receiver = MockUdpSocketReceiver::new();

        let mut accept_msg = TransactionResponse::build(TransactionCode::Accept, transaction_id);
        TransactionInfo::add_padding(&mut accept_msg);
        let mut accept_msg_clone;

        let commit_messages = [
            TransactionRequest::build(TransactionCode::Commit, transaction_id, airline_fee),
            TransactionRequest::build(TransactionCode::Commit, transaction_id, hotel_fee),
            TransactionRequest::build(TransactionCode::Commit, transaction_id, bank_fee),
        ];

        mock_sender
            .expect_send_to()
            .withf(move |_, _| true)
            .times(n_services)
            .returning(|_, _| Ok(()));

        accept_msg_clone = accept_msg.clone();
        mock_receiver
            .expect_recv()
            .withf(move |_| true)
            .times(1)
            .returning(move |_| Ok((accept_msg_clone.clone(), airline_addr.to_string())));

        accept_msg_clone = accept_msg.clone();
        mock_receiver
            .expect_recv()
            .withf(move |_| true)
            .times(1)
            .returning(move |_| Ok((accept_msg_clone.clone(), hotel_addr.to_string())));

        accept_msg_clone = accept_msg.clone();
        mock_receiver
            .expect_recv()
            .withf(move |_| true)
            .times(1)
            .returning(move |_| Ok((accept_msg_clone.clone(), bank_addr.to_string())));

        mock_sender
            .expect_send_to()
            .withf(move |buf, addr| {
                commit_messages.contains(&buf.to_vec()) && addresses.contains(&addr)
            })
            .times(n_services)
            .returning(|_, _| Ok(()));

        mock_sender
            .expect_send_to()
            .returning(move |_, _| Err(SocketError::ZeroBytes));

        mock_receiver
            .expect_recv()
            .returning(move |_| Err(SocketError::Timeout));

        let mut receiver = TransactionReceiver::new(
            Box::new(mock_receiver),
            &services_addrs_str,
            curr_transaction.clone(),
            Arc::new((Mutex::new(false), Condvar::new())),
        );

        thread::spawn(move || loop {
            let _drop = receiver.recv();
        });

        let mut manager = TransactionManager::new(
            id,
            Box::new(mock_sender),
            curr_transaction.clone(),
            Arc::new((Mutex::new(false), Condvar::new())),
            &services_addrs_str,
            &vec![],
            Duration::from_secs(2),
            None,
        );

        manager.update_current(transaction);
        manager.prepare();
        manager.commit();
    }

    #[test]
    fn it_should_send_log_after_prepare_phase() {
        let id = 0;

        let replicas_addrs = vec![
            "127.0.0.1:49159".to_string(),
            "127.0.0.1:49160".to_string(),
            "127.0.0.1:49161".to_string(),
        ];
        let replicas_addrs_clone;

        let airline_addr = "127.0.0.1:49156";
        let hotel_addr = "127.0.0.1:49157";
        let bank_addr = "127.0.0.1:49158";
        let services_addrs_str = &HashMap::from([
            (airline_addr, ServiceName::Airline.string_name()),
            (hotel_addr, ServiceName::Hotel.string_name()),
            (bank_addr, ServiceName::Bank.string_name()),
        ]);

        let transaction_id = 1;
        let airline_fee = 100.0;
        let hotel_fee = 200.0;
        let bank_fee = 300.0;
        let transaction = Transaction::new(
            transaction_id,
            &HashMap::from([
                (ServiceName::Airline.string_name(), airline_fee),
                (ServiceName::Hotel.string_name(), hotel_fee),
                (ServiceName::Bank.string_name(), bank_fee),
            ]),
        );
        let curr_transaction = Arc::new((Mutex::new(None), Condvar::new()));

        let n_services = transaction.waiting_services().len();

        let mut mock_sender = MockUdpSocketSender::new();
        let mut mock_receiver = MockUdpSocketReceiver::new();

        let mut accept_msg = TransactionResponse::build(TransactionCode::Accept, transaction_id);
        TransactionInfo::add_padding(&mut accept_msg);
        let mut accept_msg_clone;

        let mut log_msg = TransactionLog::build(
            transaction_id,
            (TransactionState::Accepted, airline_fee),
            (TransactionState::Accepted, hotel_fee),
            (TransactionState::Accepted, bank_fee),
        );
        TransactionInfo::add_padding(&mut log_msg);
        let log_msg_clone;

        mock_sender
            .expect_send_to()
            .withf(move |_, _| true)
            .times(n_services)
            .returning(|_, _| Ok(()));

        accept_msg_clone = accept_msg.clone();
        mock_receiver
            .expect_recv()
            .withf(move |_| true)
            .times(1)
            .returning(move |_| Ok((accept_msg_clone.clone(), airline_addr.to_string())));

        accept_msg_clone = accept_msg.clone();
        mock_receiver
            .expect_recv()
            .withf(move |_| true)
            .times(1)
            .returning(move |_| Ok((accept_msg_clone.clone(), hotel_addr.to_string())));

        accept_msg_clone = accept_msg.clone();
        mock_receiver
            .expect_recv()
            .withf(move |_| true)
            .times(1)
            .returning(move |_| Ok((accept_msg_clone.clone(), bank_addr.to_string())));

        log_msg_clone = log_msg.clone();
        replicas_addrs_clone = replicas_addrs.clone();
        mock_sender
            .expect_send_to()
            .withf(move |buf, addr| {
                &buf.to_vec() == &log_msg_clone && replicas_addrs_clone.contains(&addr.to_string())
            })
            .times(n_services)
            .returning(|_, _| Ok(()));

        mock_receiver
            .expect_recv()
            .returning(move |_| Err(SocketError::Timeout));

        let mut receiver = TransactionReceiver::new(
            Box::new(mock_receiver),
            &services_addrs_str,
            curr_transaction.clone(),
            Arc::new((Mutex::new(false), Condvar::new())),
        );

        thread::spawn(move || loop {
            let _drop = receiver.recv();
        });

        let mut manager = TransactionManager::new(
            id,
            Box::new(mock_sender),
            curr_transaction.clone(),
            Arc::new((Mutex::new(false), Condvar::new())),
            &services_addrs_str,
            &replicas_addrs,
            Duration::from_secs(2),
            None,
        );

        manager.update_current(transaction);
        manager.prepare();
    }

    #[test]
    fn it_should_send_log_after_abort_phase() {
        let id = 0;

        let replicas_addrs = vec![
            "127.0.0.1:49159".to_string(),
            "127.0.0.1:49160".to_string(),
            "127.0.0.1:49161".to_string(),
        ];
        let replicas_addrs_clone;

        let airline_addr = "127.0.0.1:49156";
        let hotel_addr = "127.0.0.1:49157";
        let bank_addr = "127.0.0.1:49158";
        let services_addrs_str = &HashMap::from([
            (airline_addr, ServiceName::Airline.string_name()),
            (hotel_addr, ServiceName::Hotel.string_name()),
            (bank_addr, ServiceName::Bank.string_name()),
        ]);

        let transaction_id = 1;
        let airline_fee = 100.0;
        let hotel_fee = 200.0;
        let bank_fee = 300.0;
        let mut transaction = Transaction::new(
            transaction_id,
            &HashMap::from([
                (ServiceName::Airline.string_name(), airline_fee),
                (ServiceName::Hotel.string_name(), hotel_fee),
                (ServiceName::Bank.string_name(), bank_fee),
            ]),
        );
        let n_services = transaction.waiting_services().len();

        transaction.abort(ServiceName::Airline.string_name(), Some(airline_fee));
        let not_abort_services_addrs = [hotel_addr, bank_addr];

        let curr_transaction = Arc::new((Mutex::new(None), Condvar::new()));

        let mut mock_sender = MockUdpSocketSender::new();
        let mut mock_receiver = MockUdpSocketReceiver::new();

        let abort_requests = [
            TransactionRequest::build(TransactionCode::Abort, transaction_id, hotel_fee),
            TransactionRequest::build(TransactionCode::Abort, transaction_id, bank_fee),
        ];

        let mut abort_response = TransactionResponse::build(TransactionCode::Abort, transaction_id);
        TransactionInfo::add_padding(&mut abort_response);
        let mut abort_response_clone;

        let mut log_msg = TransactionLog::build(
            transaction_id,
            (TransactionState::Aborted, airline_fee),
            (TransactionState::Aborted, hotel_fee),
            (TransactionState::Aborted, bank_fee),
        );
        TransactionInfo::add_padding(&mut log_msg);
        let log_msg_clone;

        mock_sender
            .expect_send_to()
            .withf(move |buf, addr| {
                abort_requests.contains(&buf.to_vec()) && not_abort_services_addrs.contains(&addr)
            })
            .times(not_abort_services_addrs.len())
            .returning(|_, _| Ok(()));

        abort_response_clone = abort_response.clone();
        mock_receiver
            .expect_recv()
            .withf(move |_| true)
            .times(1)
            .returning(move |_| Ok((abort_response_clone.clone(), hotel_addr.to_string())));

        abort_response_clone = abort_response.clone();
        mock_receiver
            .expect_recv()
            .withf(move |_| true)
            .times(1)
            .returning(move |_| Ok((abort_response_clone.clone(), bank_addr.to_string())));

        log_msg_clone = log_msg.clone();
        replicas_addrs_clone = replicas_addrs.clone();
        mock_sender
            .expect_send_to()
            .withf(move |buf, addr| {
                &buf.to_vec() == &log_msg_clone && replicas_addrs_clone.contains(&addr.to_string())
            })
            .times(n_services)
            .returning(|_, _| Ok(()));

        mock_receiver
            .expect_recv()
            .returning(move |_| Err(SocketError::Timeout));

        let mut receiver = TransactionReceiver::new(
            Box::new(mock_receiver),
            &services_addrs_str,
            curr_transaction.clone(),
            Arc::new((Mutex::new(false), Condvar::new())),
        );

        thread::spawn(move || loop {
            let _drop = receiver.recv();
        });

        let mut manager = TransactionManager::new(
            id,
            Box::new(mock_sender),
            curr_transaction.clone(),
            Arc::new((Mutex::new(false), Condvar::new())),
            &services_addrs_str,
            &replicas_addrs,
            Duration::from_secs(1),
            None,
        );

        manager.update_current(transaction);
        manager.abort();
    }

    #[test]
    fn it_should_send_log_after_commit_phase() {
        let id = 0;

        let replicas_addrs = vec![
            "127.0.0.1:49159".to_string(),
            "127.0.0.1:49160".to_string(),
            "127.0.0.1:49161".to_string(),
        ];
        let replicas_addrs_clone;

        let airline_addr = "127.0.0.1:49156";
        let hotel_addr = "127.0.0.1:49157";
        let bank_addr = "127.0.0.1:49158";
        let services_addrs_str = &HashMap::from([
            (airline_addr, ServiceName::Airline.string_name()),
            (hotel_addr, ServiceName::Hotel.string_name()),
            (bank_addr, ServiceName::Bank.string_name()),
        ]);

        let transaction_id = 1;
        let airline_fee = 100.0;
        let hotel_fee = 200.0;
        let bank_fee = 300.0;
        let mut transaction = Transaction::new(
            transaction_id,
            &HashMap::from([
                (ServiceName::Airline.string_name(), airline_fee),
                (ServiceName::Hotel.string_name(), hotel_fee),
                (ServiceName::Bank.string_name(), bank_fee),
            ]),
        );
        let n_services = transaction.waiting_services().len();

        transaction.commit(ServiceName::Airline.string_name(), Some(airline_fee));
        transaction.accept(ServiceName::Hotel.string_name(), Some(hotel_fee));
        transaction.accept(ServiceName::Bank.string_name(), Some(bank_fee));
        let accepted_services_addrs = [hotel_addr, bank_addr];

        let curr_transaction = Arc::new((Mutex::new(None), Condvar::new()));

        let mut mock_sender = MockUdpSocketSender::new();
        let mut mock_receiver = MockUdpSocketReceiver::new();

        let commit_requests = [
            TransactionRequest::build(TransactionCode::Commit, transaction_id, hotel_fee),
            TransactionRequest::build(TransactionCode::Commit, transaction_id, bank_fee),
        ];

        let mut commit_response =
            TransactionResponse::build(TransactionCode::Commit, transaction_id);
        TransactionInfo::add_padding(&mut commit_response);
        let mut commit_response_clone;

        let mut log_msg = TransactionLog::build(
            transaction_id,
            (TransactionState::Commited, airline_fee),
            (TransactionState::Commited, hotel_fee),
            (TransactionState::Commited, bank_fee),
        );
        TransactionInfo::add_padding(&mut log_msg);
        let log_msg_clone;

        mock_sender
            .expect_send_to()
            .withf(move |buf, addr| {
                commit_requests.contains(&buf.to_vec()) && accepted_services_addrs.contains(&addr)
            })
            .times(accepted_services_addrs.len())
            .returning(|_, _| Ok(()));

        commit_response_clone = commit_response.clone();
        mock_receiver
            .expect_recv()
            .withf(move |_| true)
            .times(1)
            .returning(move |_| Ok((commit_response_clone.clone(), hotel_addr.to_string())));

        commit_response_clone = commit_response.clone();
        mock_receiver
            .expect_recv()
            .withf(move |_| true)
            .times(1)
            .returning(move |_| Ok((commit_response_clone.clone(), bank_addr.to_string())));

        log_msg_clone = log_msg.clone();
        replicas_addrs_clone = replicas_addrs.clone();
        mock_sender
            .expect_send_to()
            .withf(move |buf, addr| {
                &buf.to_vec() == &log_msg_clone && replicas_addrs_clone.contains(&addr.to_string())
            })
            .times(n_services)
            .returning(|_, _| Ok(()));

        mock_receiver
            .expect_recv()
            .returning(move |_| Err(SocketError::Timeout));

        let mut receiver = TransactionReceiver::new(
            Box::new(mock_receiver),
            &services_addrs_str,
            curr_transaction.clone(),
            Arc::new((Mutex::new(false), Condvar::new())),
        );

        thread::spawn(move || loop {
            let _drop = receiver.recv();
        });

        let mut manager = TransactionManager::new(
            id,
            Box::new(mock_sender),
            curr_transaction.clone(),
            Arc::new((Mutex::new(false), Condvar::new())),
            &services_addrs_str,
            &replicas_addrs,
            Duration::from_secs(1),
            None,
        );

        manager.update_current(transaction);
        manager.commit();
    }
}
