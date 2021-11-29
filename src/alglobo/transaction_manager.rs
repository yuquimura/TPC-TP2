use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

use crate::{
    sockets::udp_socket_sender::UdpSocketSender,
    transactions::{transaction_code::TransactionCode, transaction_request::TransactionRequest},
};

use super::{
    transaction::Transaction, transaction_error::TransactionError, transactionable::Transactionable,
};

#[allow(dead_code)]
struct TransactionManager {
    pub id: usize,
    udp_socket_wrap: Box<dyn UdpSocketSender>,
    services_addrs: HashMap<String, String>,
    curr_transaction: Arc<(Mutex<Option<Transaction>>, Condvar)>,
    timeout: Duration,
}

#[allow(dead_code)]
impl TransactionManager {
    pub fn new(
        id: usize,
        udp_socket_wrap: Box<dyn UdpSocketSender>,
        services_addrs_str: &HashMap<String, &str>,
        timeout: Duration,
    ) -> Self {
        let services_addrs = services_addrs_str
            .iter()
            .map(|(name, addr)| (name.clone(), (*addr).to_string()))
            .collect();
        // thread::spawn(|| {
        //     let receiver = TransactionReceiver::new();
        //     receiver.recv()
        // });

        TransactionManager {
            id,
            udp_socket_wrap,
            services_addrs,
            curr_transaction: Arc::new((Mutex::new(None), Condvar::new())),
            timeout,
        }
    }

    pub fn process(&mut self, transaction: Transaction) {
        self.update_current(transaction);
        if !self.prepare() {
            self.abort();
        }
    }

    fn update_current(&mut self, transaction: Transaction) {
        let mut opt_transaction = self
            .curr_transaction
            .0
            .lock()
            .expect("[Transaction Manager] Lock de transaccion envenenado");
        *opt_transaction = Some(transaction);
    }

    fn prepare(&mut self) -> bool {
        let transaction_id;
        let waiting_services;
        {
            let opt_transaction = self
                .curr_transaction
                .0
                .lock()
                .expect("[Transaction Manager] Lock de transaccion envenenado");
            let transaction = opt_transaction
                .as_ref()
                .expect("[Transaction Manager] La transaccion actual deberia exitir");
            transaction_id = transaction.get_id();
            waiting_services = transaction.waiting_services();
        }
        self.send_messages(TransactionCode::Prepare, transaction_id, waiting_services);
        let res = self.wait_update(|opt_transaction| {
            !opt_transaction
                .as_ref()
                .expect("[Transaction Manager] La transacci\u{f3}n actual deberia existir")
                .is_accepted()
        });
        res.is_ok()
    }

    fn abort(&mut self) -> bool {
        let transaction_id;
        let all_services;
        {
            let opt_transaction = self
                .curr_transaction
                .0
                .lock()
                .expect("[Transaction Manager] Lock de transaccion envenenado");
            let transaction = opt_transaction
                .as_ref()
                .expect("[Transaction Manager] La transaccion actual deberia exitir");
            transaction_id = transaction.get_id();
            all_services = transaction.all_services();
        }
        self.send_messages(TransactionCode::Abort, transaction_id, all_services);
        let res = self.wait_update(|opt_transaction| {
            !opt_transaction
                .as_ref()
                .expect("[Transaction Manager] La transacci\u{f3}n actual deberia existir")
                .is_aborted()
        });
        res.is_ok()
    }

    fn wait_update(
        &self,
        condition: fn(&mut Option<Transaction>) -> bool,
    ) -> Result<(), TransactionError> {
        let res = self
            .curr_transaction
            .1
            .wait_timeout_while(
                self.curr_transaction.0.lock().unwrap(),
                self.timeout,
                condition,
            )
            .expect("[Transaction Manager] Lock de transacci\u{f3}n envenenado");
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
            self.udp_socket_wrap
                .send_to(&TransactionRequest::build(code, id, fee), addr)
                .expect(
                    "[Transaction Manager] Enviar mensaje de transacci\u{f3}n no deberia fallar",
                );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        services2::service_name::ServiceName, 
        sockets::udp_socket_sender::MockUdpSocketSender,
    };

    use std::collections::HashMap;

    #[test]
    fn process_transaction_should_send_msg_prepare_to_all_services_in_transaction() {
        let airline_addr = "127.0.0.1:49156";
        let hotel_addr = "127.0.0.1:49157";
        let bank_addr = "127.0.0.1:49158";

        let transaction_id = 0;
        let airline_fee = 100.0;
        let hotel_fee = 200.0;
        let bank_fee = 300.0;
        let transaction = Transaction::new(
            transaction_id,
            HashMap::from([
                (ServiceName::airline(), airline_fee),
                (ServiceName::hotel(), hotel_fee),
                (ServiceName::bank(), bank_fee),
            ]),
        );

        let waiting_services = transaction.waiting_services();

        let mut mock_socket = MockUdpSocketSender::new();

        let addresses = [airline_addr, hotel_addr, bank_addr];

        let transaction_messages = [
            TransactionRequest::build(TransactionCode::Prepare, transaction_id, airline_fee),
            TransactionRequest::build(TransactionCode::Prepare, transaction_id, hotel_fee),
            TransactionRequest::build(TransactionCode::Prepare, transaction_id, bank_fee),
        ];

        let addresses_clone = addresses;
        let messages_clone = transaction_messages;
        mock_socket
            .expect_send_to()
            .withf(move |buf, addr| {
                messages_clone.contains(&buf.to_vec()) && addresses_clone.contains(&addr)
            })
            .times(waiting_services.len())
            .returning(|_, _| Ok(()));

        // Necesario porque no esta implementado TransactionReceiver
        mock_socket
            .expect_send_to()
            .withf(move |_, _| true)
            .times(waiting_services.len())
            .returning(|_, _| Ok(()));

        let mut manager = TransactionManager::new(
            0,
            Box::new(mock_socket),
            &HashMap::from([
                (ServiceName::airline(), airline_addr),
                (ServiceName::hotel(), hotel_addr),
                (ServiceName::bank(), bank_addr),
            ]),
            Duration::from_secs(0),
        );

        manager.process(transaction);
    }

    #[test]
    fn process_transaction_should_send_msg_abort_to_all_services_if_any_service_does_not_respond_to_prepare_msg(
    ) {
        let airline_addr = "127.0.0.1:49156";
        let hotel_addr = "127.0.0.1:49157";
        let bank_addr = "127.0.0.1:49158";

        let transaction_id = 1;
        let airline_fee = 100.0;
        let hotel_fee = 200.0;
        let bank_fee = 300.0;
        let transaction = Transaction::new(
            transaction_id,
            HashMap::from([
                (ServiceName::airline(), airline_fee),
                (ServiceName::hotel(), hotel_fee),
                (ServiceName::bank(), bank_fee),
            ]),
        );

        let waiting_services = transaction.waiting_services();

        let mut mock_socket = MockUdpSocketSender::new();

        mock_socket
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
        mock_socket
            .expect_send_to()
            .withf(move |buf, addr| {
                messages_clone.contains(&buf.to_vec()) && addresses_clone.contains(&addr)
            })
            .times(waiting_services.len())
            .returning(|_, _| Ok(()));

        let mut manager = TransactionManager::new(
            0,
            Box::new(mock_socket),
            &HashMap::from([
                (ServiceName::airline(), airline_addr),
                (ServiceName::hotel(), hotel_addr),
                (ServiceName::bank(), bank_addr),
            ]),
            Duration::from_secs(0),
        );

        manager.process(transaction);
    }
}
