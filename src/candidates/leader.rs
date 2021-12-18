use crate::alglobo::transaction_manager::TransactionManager;
// use crate::alglobo::transactionable::Transactionable;
// use crate::candidates::constants::SLEEP;
use crate::candidates::election_code::ElectionCode;
use crate::candidates::election_message::ElectionMessage;
// use crate::file_reader::file_iterator::FileIterator;
use crate::sockets::udp_socket_receiver::UdpSocketReceiver;
use crate::sockets::udp_socket_sender::UdpSocketSender;
use std::ops::Range;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
// use std::thread::sleep;
use std::time::Duration;

use super::constants::TRANSACTION_FILE;

// use super::constants::END_TIMEOUT;

#[allow(dead_code)]
pub struct Leader {
    /*udp_receiver: Box<dyn UdpSocketReceiver + Send>,
    udp_sender: Box<dyn UdpSocketSender + Send>,*/
    possible_ports: Range<i32>,
}

impl Leader {
    #[must_use]
    pub fn new(
       /* udp_receiver: Box<dyn UdpSocketReceiver + Send>,
        udp_sender: Box<dyn UdpSocketSender + Send>,*/
        possible_ports: Range<i32>,
    ) -> Self {
        Leader {
         /*   udp_receiver,
            udp_sender,*/
            possible_ports,
        }
    }

    pub fn recv(&mut self, recv: &mut Box<dyn UdpSocketReceiver>, send: &mut Box<dyn UdpSocketSender>) {
        recv.set_timeout(Some(Duration::from_millis(10000)));
        let result = recv.recv(ElectionMessage::size());
        if let Ok(response) = result.as_ref() {
            match response.0[0] {
                b'v' => {
                    let message = ElectionMessage::build(ElectionCode::Alive);
                    let his_address = response.1.clone();
                    let _drop = send.send_to(message.as_slice(), &his_address);
                }
                b'e' => {
                    let his_address = response.1.clone();
                    for port in self.possible_ports.clone() {
                        let message = ElectionMessage::build(ElectionCode::Leader);
                        let his_address_vect: Vec<&str> = his_address.split(':').collect();
                        let address_to_send = his_address_vect[0].to_string() +":"+ &port.to_string();
                        let _drop = send.send_to(message.as_slice(), &address_to_send);
                    }
                }
                _ => {
                }
            }
        }
    }

    pub fn start_leader(&mut self, mut transaction_manager: TransactionManager, _start_line: u64, recv: &mut Box<dyn UdpSocketReceiver>,send: &mut Box<dyn UdpSocketSender>) {
        let boolean = false;
        let finish_lock = Arc::new(RwLock::new(boolean));
        let finish_lock_clone = finish_lock.clone();
        let join_handle = thread::spawn(move || {
            transaction_manager.run(TRANSACTION_FILE, finish_lock_clone);
            // if let Ok(mut reader) = FileIterator::new("data/data.csv") {
            //     while !reader.ended() {
            //         if let Some(transaction) = reader.next() {
            //             if transaction.get_id() > start_line {
            //                 sleep(Duration::from_secs(SLEEP));
            //                 transaction_manager.process(Some(transaction));
            //             }
            //         }
            //     }
            // }

            // while let Ok(_) = transaction_manager.wait_end_while(END_TIMEOUT.clone()) {
            //     transaction_manager.process(None);
            // }

            // // Esperar Reintentos por un tiempo X antes de terminar la ejecucion.
            // let mut result = lock_clone.write().expect("El lock esta envenenado");
            // *result = true;
        }); 
        loop {
            self.recv(recv,send);
            let result_read = finish_lock.read().expect("El lock esta envenenado");
            if *result_read {
                break;
            }
        };
        let _ = join_handle.join();
    }
}
/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidates::candidate::Candidate;
    use crate::{
        candidates::election_code::ElectionCode,
        sockets::udp_socket_receiver::MockUdpSocketReceiver,
        sockets::udp_socket_sender::MockUdpSocketSender,
    };

    #[test]
    fn it_should_receive_alive_message(){
        let address = "127.0.0.1:49156";
        let mut mock_receiver = MockUdpSocketReceiver::new();
        let mut mock_sender = MockUdpSocketSender::new();
        let mut mock_receiver_candidate = MockUdpSocketReceiver::new();
        let mut mock_sender_leader = MockUdpSocketSender::new();
        let message = ElectionMessage::build(ElectionCode::Alive);
        let messages = [message.clone()];
        mock_receiver
            .expect_recv()
            .withf(|n_bytes| n_bytes == &ElectionMessage::size())
            .times(1)
            .returning(move |_| Ok((message.clone(),address.to_string())));
        mock_sender
            .expect_send_to()
            .withf(move |buf, addr| {
                messages.contains(&buf.to_vec()) && addr == address
            })
            .times(1)
            .returning(|_, _| Ok(()));
        let leader = Leader::new(Box::new(mock_receiver), Box::new(mock_sender_leader),vec!["".to_string()]);
        let mut candidate = Candidate::new(Box::new(mock_receiver_candidate),
                                           Box::new(mock_sender),
                                           "49156".to_string(),
                                           vec!["".to_string()],
                                           "49156".to_string(), address.to_string());
        candidate.send_to();
    }
}
 */
