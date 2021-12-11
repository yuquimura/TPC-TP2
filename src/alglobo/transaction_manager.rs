use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex},
    time::Duration, thread,
};

use crate::{
    sockets::{
        udp_socket_sender::UdpSocketSender,
        udp_socket_receiver::UdpSocketReceiver
    },
    transactions::{transaction_code::TransactionCode, transaction_request::TransactionRequest},
};

use super::{
    transaction_receiver::TransactionReceiver,
    transaction::Transaction, transaction_error::TransactionError,
    transactionable::Transactionable,
    types::CurrentTransaction, 
};

#[allow(dead_code)]
struct TransactionManager {
    pub id: u64,
    udp_sender: Box<dyn UdpSocketSender>,
    services_addrs: HashMap<String, String>,
    curr_transaction: CurrentTransaction,
    timeout: Duration,
}

#[allow(dead_code)]
impl TransactionManager {
    pub fn new(
        id: u64,
        udp_sender: Box<dyn UdpSocketSender>,
        udp_receiver: Box<dyn UdpSocketReceiver + Send>,
        services_addrs_str: &HashMap<&str, String>, // Voltear => (addr, name)
        timeout: Duration,
    ) -> Self {
        let services_addrs = services_addrs_str
            .clone()
            .iter()
            .map(|(addr, name)| (name.clone(), (*addr).to_string()))
            .collect();

        let curr_transaction = Arc::new((Mutex::new(None), Condvar::new()));

        let mut receiver = TransactionReceiver::new(
            id,
            udp_receiver,
            services_addrs_str,
            curr_transaction.clone()
        );

        thread::spawn(move || {
            // let mut rounds = 3;
            // while rounds > 0 {
            //     receiver.recv()
            //         .expect("[Transaction Receiver] Algo salio mal");
            //     rounds -= 1;
            // }
            loop {
                receiver.recv()
                    .expect("[Transaction Receiver] Algo salio mal");
            }
        });

        TransactionManager {
            id,
            udp_sender,
            services_addrs,
            curr_transaction,
            timeout,
        }
    }

    pub fn process(&mut self, transaction: Transaction) {
        self.update_current(transaction);
        if !self.prepare() {
            self.abort();
        } else {
            self.commit();
        }
    }

    fn update_current(&mut self, transaction: Transaction) {
        let mut opt_transaction = self
            .curr_transaction
            .0
            .lock()
            .expect("[Transaction Manager] Lock de transaccion envenenado");
        *opt_transaction = Some(Box::new(transaction));
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

    fn commit(&mut self) -> bool {
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
        self.send_messages(TransactionCode::Commit, transaction_id, all_services);
        let res = self.wait_update(|opt_transaction| {
            !opt_transaction
                .as_ref()
                .expect("[Transaction Manager] La transacci\u{f3}n actual deberia existir")
                .is_commited()
        });
        res.is_ok()
    }

    fn wait_update(
        &self,
        condition: fn(&mut Option<Box<dyn Transactionable + Send>>) -> bool,
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
            self.udp_sender
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
        services::service_name::ServiceName, 
        sockets::{
            udp_socket_sender::MockUdpSocketSender,
            udp_socket_receiver::MockUdpSocketReceiver, socket_error::SocketError
        },
        transactions::transaction_response::TransactionResponse
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

        let mut mock_sender = MockUdpSocketSender::new();
        let mock_receiver = MockUdpSocketReceiver::new();

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

        // Necesario porque no esta implementado TransactionReceiver
        mock_sender
            .expect_send_to()
            .withf(move |_, _| true)
            .times(waiting_services.len())
            .returning(|_, _| Ok(()));

        // mock_receiver
        //     .expect_recv()
        //     .returning(move |_| 
        //         Err(SocketError::Timeout)
        //     );

        let mut manager = TransactionManager::new(
            0,
            Box::new(mock_sender),
            Box::new(mock_receiver),
            &HashMap::from([
                (airline_addr, ServiceName::airline()),
                (hotel_addr, ServiceName::hotel()),
                (bank_addr, ServiceName::bank()),
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

        // mock_receiver
        //     .expect_recv()
        //     .returning(move |_| 
        //         Err(SocketError::Timeout)
        //     );

        let mut manager = TransactionManager::new(
            0,
            Box::new(mock_sender),
            Box::new(mock_receiver),
            &HashMap::from([
                (airline_addr, ServiceName::airline()),
                (hotel_addr, ServiceName::hotel()),
                (bank_addr, ServiceName::bank()),
            ]),
            Duration::from_secs(0),
        );

        manager.process(transaction);
    }

    #[test]
    fn process_transaction_should_send_msg_commit_to_all_services_if_all_services_responded_with_accept_msg(
    ) {
        let airline_addr = "127.0.0.1:49156";
        let hotel_addr = "127.0.0.1:49157";
        let bank_addr = "127.0.0.1:49158";
        let addresses = [airline_addr, hotel_addr, bank_addr];

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

        let n_services = transaction.waiting_services().len();

        let mut mock_sender = MockUdpSocketSender::new();
        let mut mock_receiver = MockUdpSocketReceiver::new();
        
        let accept_msg = TransactionResponse::build(TransactionCode::Accept, transaction_id);
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
            .returning(move |_| 
                Ok((accept_msg_clone.clone(), airline_addr.to_string())
            ));
        
        accept_msg_clone = accept_msg.clone();
        mock_receiver
            .expect_recv()
            .withf(move |_| true)
            .times(1)
            .returning(move |_| 
                Ok((accept_msg_clone.clone(), hotel_addr.to_string())
            ));
        
        accept_msg_clone = accept_msg.clone();
        mock_receiver
            .expect_recv()
            .withf(move |_| true)
            .times(1)
            .returning(move |_| 
                Ok((accept_msg_clone.clone(), bank_addr.to_string())
            ));   

        mock_sender
            .expect_send_to()
            .withf(move |buf, addr| 
                commit_messages.contains(&buf.to_vec()) &&
                addresses.contains(&addr)
            )
            .times(n_services)
            .returning(|_, _| Ok(()));
        
        mock_receiver
            .expect_recv()
            .returning(move |_| 
                Err(SocketError::Timeout)
            );

        let mut manager = TransactionManager::new(
            0,
            Box::new(mock_sender),
            Box::new(mock_receiver),
            &HashMap::from([
                (airline_addr, ServiceName::airline()),
                (hotel_addr, ServiceName::hotel()),
                (bank_addr, ServiceName::bank()),
            ]),
            Duration::from_secs(1),
        );

        manager.process(transaction);
    }
}
