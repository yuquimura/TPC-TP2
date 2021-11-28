use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

use crate::sockets::udp_socket_sender::UdpSocketSender;

use super::{transaction::Transaction, transaction_message::TransactionMessage};

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
        self.update_curr_transaction(transaction);
        self.prepare_transaction();
        //self.wait_transaction_update();
    }

    fn update_curr_transaction(&mut self, transaction: Transaction) {
        let mut opt_transaction = self
            .curr_transaction
            .0
            .lock()
            .expect("[Transaction Manager] Lock de transaccion envenenado");
        *opt_transaction = Some(transaction);
    }

    fn prepare_transaction(&mut self) {
        let opt_transaction = self
            .curr_transaction
            .0
            .lock()
            .expect("[Transaction Manager] Lock de transaccion envenenado");
        let transaction_ref = opt_transaction
            .as_ref()
            .expect("[Transaction Manager] La transaccion actual deberia exitir");
        let id = transaction_ref.get_id();
        let waiting_services = transaction_ref.waiting_services();
        for (name, fee) in waiting_services {
            let addr = self
                .services_addrs
                .get(&name)
                .expect("[Transaction Manager] Direccion del servicio no existe");
            self.udp_socket_wrap
                .send_to(&TransactionMessage::prepare(id, fee), addr)
                .expect("[Transaction Manager] Enviar el mensaje PREPARE no deberia fallar");
        }
    }

    // fn wait_transaction_update(&self) {
    //     let res = self.curr_transaction
    //         .1
    //         .wait_timeout_while(
    //             self.curr_transaction
    //                     .0
    //                     .lock()
    //                     .unwrap(),
    //             self.timeout,
    //             |opt_transaction|
    //                         opt_transaction
    //                             .as_ref()
    //                             .expect("[Transaction Manager] La transacción actual deberia existir")
    //                             .is_accepted()
    //         );

    // }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        alglobo::{transaction::Transaction, transaction_message::TransactionMessage},
        services::service_name::ServiceName,
        sockets::udp_socket_sender::MockUdpSocketSender,
    };

    use super::*;

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
            TransactionMessage::prepare(transaction_id, airline_fee),
            TransactionMessage::prepare(transaction_id, hotel_fee),
            TransactionMessage::prepare(transaction_id, bank_fee),
        ];

        let addresses_clone = addresses.clone();
        let messages_clone = transaction_messages.clone();
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

    // #[test]
    // fn process_transaction_should_send_msg_abort_to_all_services_if_any_service_does_not_respond_to_prepare_msg() {
    //     let airline_addr = "127.0.0.1:49156";
    //     let hotel_addr = "127.0.0.1:49157";
    //     let bank_addr = "127.0.0.1:49158";

    //     let transaction_id = 1;
    //     let airline_fee = 100.0;
    //     let hotel_fee = 200.0;
    //     let bank_fee = 300.0;
    //     let transaction = Transaction::new(
    //         transaction_id,
    //         HashMap::from([
    //             (ServiceName::airline(), airline_fee),
    //             (ServiceName::hotel(), hotel_fee),
    //             (ServiceName::bank(), bank_fee),
    //         ]),
    //     );

    //     let waiting_services = transaction.waiting_services();

    //     let mut mock_socket = MockUdpSocketSender::new();

    //     mock_socket
    //         .expect_send_to()
    //         .withf(move |_, _| true)
    //         .times(waiting_services.len())
    //         .returning(|_, _| Ok(()));

    //     let addresses = [
    //         airline_addr,
    //         hotel_addr,
    //         bank_addr
    //     ];

    //     let abort_messages = [
    //         TransactionMessage::abort(transaction_id, airline_fee),
    //         TransactionMessage::abort(transaction_id, hotel_fee),
    //         TransactionMessage::abort(transaction_id, bank_fee),
    //     ];

    //     let addresses_clone = addresses.clone();
    //     let messages_clone = abort_messages.clone();
    //     mock_socket
    //         .expect_send_to()
    //         .withf(move |buf, addr|
    //             messages_clone.contains(&buf.to_vec())
    //             && addresses_clone.contains(&addr)
    //         )
    //         .times(waiting_services.len())
    //         .returning(|_, _| Ok(()));

    //     let mut manager = TransactionManager::new(
    //         0,
    //         Box::new(mock_socket),
    //         &HashMap::from([
    //             (ServiceName::airline(), airline_addr),
    //             (ServiceName::hotel(), hotel_addr),
    //             (ServiceName::bank(), bank_addr),
    //         ]),
    //     );

    //     manager.process(&transaction);
    // }
}
