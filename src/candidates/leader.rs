use crate::alglobo::transaction_manager::TransactionManager;
use crate::alglobo::transactionable::Transactionable;
use crate::candidates::election_code::ElectionCode;
use crate::candidates::election_message::ElectionMessage;
use crate::file_reader::file_iterator::FileIterator;
use crate::sockets::udp_socket_receiver::UdpSocketReceiver;
use crate::sockets::udp_socket_sender::UdpSocketSender;
use std::ops::Range;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use crate::candidates::constants::SLEEP;

#[allow(dead_code)]
pub struct Leader {
    udp_receiver: Box<dyn UdpSocketReceiver + Send>,
    udp_sender: Box<dyn UdpSocketSender + Send>,
    possible_ports: Range<i32>,
}

impl Leader {
    #[must_use]
    pub fn new(
        udp_receiver: Box<dyn UdpSocketReceiver + Send>,
        udp_sender: Box<dyn UdpSocketSender + Send>,
        possible_ports: Range<i32>,
    ) -> Self {
        Leader {
            udp_receiver,
            udp_sender,
            possible_ports,
        }
    }

    pub fn recv(&mut self) {
        self.udp_receiver
            .set_timeout(Some(Duration::from_millis(1000)));
        if let Ok(response) = self.udp_receiver.recv(ElectionMessage::size()) {
            match response.0[0] {
                b'v' => {
                    let message = ElectionMessage::build(ElectionCode::Alive);
                    let his_address = response.1.clone();
                    let _ = self.udp_sender.send_to(message.as_slice(), &his_address);
                }
                b'e' => {
                    let his_address = response.1.clone();
                    for port in self.possible_ports.clone() {
                        let message = ElectionMessage::build(ElectionCode::Leader);
                        let his_address_vect: Vec<&str> = his_address.split(':').collect();
                        let address_to_send = his_address_vect[0].to_string() + &port.to_string();
                        let _ = self
                            .udp_sender
                            .send_to(message.as_slice(), &address_to_send);
                    }
                }
                _ => {
                    println!("No hay mas casos");
                }
            }
        }
    }

    pub fn start_leader(&mut self, mut transaction_manager: TransactionManager, start_line: u64) {
        let boolean = false;
        let lock = Arc::new(RwLock::new(boolean));
        let lock_clone = lock.clone();
        let join_handle = thread::spawn(move || {
            if let Ok(mut reader) = FileIterator::new("data/data.csv") {
                while !reader.ended() {
                    if let Some(transaction) = reader.next() {
                        if transaction.get_id() > start_line {
                            sleep(Duration::from_secs(SLEEP));
                            transaction_manager.process(Some(transaction));
                        }
                    }
                }
            }
            let mut result = lock_clone.write().expect("El lock esta envenenado");
            *result = true;
        });
        loop {
            self.recv();
            let result_read = lock.read().expect("El lock esta envenenado");
            if *result_read {
                break;
            }
        }
        let _ = join_handle.join();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidates::candidate::Candidate;
    use crate::{
        candidates::election_code::ElectionCode,
        sockets::udp_socket_receiver::MockUdpSocketReceiver,
        sockets::udp_socket_sender::MockUdpSocketSender,
    };

    /*#[test]
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
    }*/
}
