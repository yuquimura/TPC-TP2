use crate::alglobo::transaction_manager::TransactionManager;
use crate::candidates::election_code::ElectionCode;
use crate::candidates::election_message::ElectionMessage;
use crate::sockets::udp_socket_receiver::UdpSocketReceiver;
use crate::sockets::udp_socket_sender::UdpSocketSender;
use std::ops::Range;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

use super::constants::TRANSACTION_FILE;

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

    pub fn recv(
        &mut self,
        recv: &mut Box<dyn UdpSocketReceiver>,
        send: &mut Box<dyn UdpSocketSender>,
    ) {
        recv.set_timeout(Some(Duration::from_millis(10000)));
        let result = recv.recv(ElectionMessage::size());
        if let Ok(response) = result.as_ref() {
            match response.0[0] {
                b'f' => {
                    println!("[Lider] Informo que soy el lider");
                    let his_address = response.1.clone();
                    let message = ElectionMessage::build(ElectionCode::Leader);
                    let _drop = send.send_to(message.as_slice(), &his_address);
                }

                b'v' => {
                    let message = ElectionMessage::build(ElectionCode::Alive);
                    let his_address = response.1.clone();
                    let _drop = send.send_to(message.as_slice(), &his_address);
                }
                b'e' => {
                    println!("[Lider] Informo que soy el lider");
                    let his_address = response.1.clone();
                    for port in self.possible_ports.clone() {
                        let message = ElectionMessage::build(ElectionCode::Leader);
                        let his_address_vect: Vec<&str> = his_address.split(':').collect();
                        let address_to_send =
                            his_address_vect[0].to_string() + ":" + &port.to_string();
                        let _drop = send.send_to(message.as_slice(), &address_to_send);
                    }
                }
                _ => {
                    println!("[Lider] Mensaje inesperado");
                }
            }
        }
    }

    pub fn start_leader(
        &mut self,
        mut transaction_manager: TransactionManager,
        recv: &mut Box<dyn UdpSocketReceiver>,
        send: &mut Box<dyn UdpSocketSender>,
    ) {
        let boolean = false;
        let finish_lock = Arc::new(RwLock::new(boolean));
        let finish_lock_clone = finish_lock.clone();
        let join_handle = thread::spawn(move || {
            transaction_manager.run(TRANSACTION_FILE, &finish_lock_clone);
        });
        loop {
            self.recv(recv, send);
            let result_read = finish_lock.read().expect("El lock esta envenenado");
            if *result_read {
                break;
            }
        }
        let _ = join_handle.join();
    }
}
