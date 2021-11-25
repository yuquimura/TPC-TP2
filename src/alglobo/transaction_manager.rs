use std::collections::HashMap;

use crate::sockets::udp_socket_trait::UdpSocketTrait;

use super::{transaction::Transaction, transaction_message::TransactionMessage};

#[allow(dead_code)]
struct TransactionManager {
    pub id: usize,
    udp_socket_wrap: Box<dyn UdpSocketTrait>,
    services_addrs: HashMap<String, String>
}

#[allow(dead_code)]
impl TransactionManager {
    pub fn new(
        id: usize,
        udp_socket_wrap: Box<dyn UdpSocketTrait>,
        services_addrs_str: HashMap<String, &str>
    ) -> Self {
        let services_addrs = services_addrs_str
            .iter()
            .map(|(name, addr)| (name.clone(), addr.to_string()))
            .collect();
        TransactionManager{
            id,
            udp_socket_wrap,
            services_addrs
        }
    }

    pub fn process(&mut self, transaction: Transaction) -> Result<(), String> {
        let waiting_services = transaction.waiting_services();
        for (name, _fee) in waiting_services {
            let addr = self.services_addrs
                .get(&name)
                .expect("[Transaction Manager] Dirección del servicio no existe");
            self.udp_socket_wrap.send_to(&TransactionMessage::prepare(), addr)
                .expect("[Transaction Manager] Enviar el mensaje PREPARE no deberia fallar");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        alglobo::{transaction::Transaction, transaction_message::TransactionMessage}, 
        services::service_name::ServiceName,
        sockets::udp_socket_trait::MockUdpSocketTrait
    };

    use super::*;

    #[test]
    fn process_transaction_should_send_msg_prepare_to_all_services_in_transaction() {
        let airline_addr = "127.0.0.1:49156";
        let hotel_addr = "127.0.0.1:49157";
        let bank_addr = "127.0.0.1:49158";

        let transaction = Transaction::new(
            HashMap::from([
                (ServiceName::airline(), 100.0),
                (ServiceName::hotel(), 200.0),
                (ServiceName::bank(), 300.0),
            ])
        );

        let mut mock_socket = MockUdpSocketTrait::new();

        mock_socket.expect_send_to()
            .withf(move |bytes_vec, addr| 
                bytes_vec == &TransactionMessage::prepare() && 
                [airline_addr, hotel_addr, bank_addr].contains(&addr))
            .returning(|_, _| Ok(()));
        
        mock_socket.expect_send_to()
            .withf(move |bytes_vec, addr| 
                bytes_vec == &TransactionMessage::prepare() && 
                [airline_addr, hotel_addr, bank_addr].contains(&addr))
            .returning(|_, _| Ok(()));
        
        mock_socket.expect_send_to()
            .withf(move |bytes_vec, addr| 
                bytes_vec == &TransactionMessage::prepare() && 
                [airline_addr, hotel_addr, bank_addr].contains(&addr))
            .returning(|_, _| Ok(()));
        
        let mut manager = TransactionManager::new(
            0, 
            Box::new(mock_socket),
            HashMap::from([
                (ServiceName::airline(), airline_addr),
                (ServiceName::hotel(), hotel_addr),
                (ServiceName::bank(), bank_addr),
            ])
        );

        let res = manager.process(transaction);
        assert!(res.is_ok());

    }
}