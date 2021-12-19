use crate::alglobo::transaction_manager::TransactionManager;
use crate::alglobo::transaction_receiver::TransactionReceiver;
use crate::alglobo::types::CurrentTransaction;
use crate::candidates::constants::{
    AIRLINE_ADDR, BANK_ADDR, DEFAULT_IP, EMPTY, 
    HOTEL_ADDR, VEC_PORT_DATA, VEC_PORT_INFO,
    ABORT_FILE
};
use crate::candidates::election_code::ElectionCode;
use crate::candidates::election_message::ElectionMessage;
use crate::candidates::leader::Leader;
use crate::file_reader::file_iterator::FileIterator;
use crate::services::service_name::ServiceName;
use crate::sockets::udp_socket_receiver::UdpSocketReceiver;
use crate::sockets::udp_socket_sender::UdpSocketSender;
use crate::sockets::udp_socket_wrap::UdpSocketWrap;
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

#[allow(dead_code)]
pub struct Candidate {
    udp_receiver: Box<dyn UdpSocketReceiver>,
    udp_sender: Box<dyn UdpSocketSender>,
    my_port: String,
    possible_ports: Vec<String>,
    leader_port: String,
    leader_address: String,
    im_the_leader: bool,
}

impl Candidate {
    #[must_use]
    pub fn new(
        udp_receiver: Box<dyn UdpSocketReceiver>,
        udp_sender: Box<dyn UdpSocketSender>,
        my_port: String,
        possible_ports: Vec<String>,
        leader_port: String,
        leader_address: String,
    ) -> Self {
        let im_the_leader = false;
        Candidate {
            udp_receiver,
            udp_sender,
            my_port,
            possible_ports,
            leader_port,
            leader_address,
            im_the_leader,
        }
    }

    pub fn send_to(&mut self) {
        if self.leader_port == EMPTY {
            self.im_the_leader = self.start_election(&DEFAULT_IP.to_string());
            return;
        }
        let message = ElectionMessage::build(ElectionCode::Alive);
        let _drop = self
            .udp_sender
            .send_to(message.as_slice(), &self.leader_address);
        self.udp_receiver
            .set_timeout(Some(Duration::from_millis(1000)));
        if let Ok(value) = self.udp_receiver.recv(ElectionMessage::size()) {
            match value.0[0] {
                b'v' => {
                    self.udp_receiver
                        .set_timeout(Some(Duration::from_millis(10000)));
                    if let Ok(response) = self.udp_receiver.recv(ElectionMessage::size()) {
                        let his_address = response.1;
                        let _drop = self.udp_sender.send_to(message.as_slice(), &his_address);
                        self.im_the_leader = self.start_election(&his_address);
                        if self.im_the_leader {
                            //soy el lider
                            self.communicate_new_leader(his_address)
                        }
                    }
                }
                //contemplar que pasa cuando llega un mensaje de election y tengo que contestar OK, como se que no soy el lider?
                b'e' => {
                    let his_address = value.1;
                    let _drop = self.udp_sender.send_to(message.as_slice(), &his_address);
                    self.im_the_leader = self.start_election(&his_address);
                    if self.im_the_leader {
                        //soy el lider
                        self.communicate_new_leader(his_address);
                    } else {
                        loop {
                            self.udp_receiver
                                .set_timeout(Some(Duration::from_millis(10000)));
                            if let Ok(response) = self.udp_receiver.recv(ElectionMessage::size()) {
                                if response.0[0] == b'l' {
                                    let his_port_vect: Vec<&str> = response.1.split(':').collect();
                                    self.leader_port = his_port_vect[1].to_string();
                                    self.leader_address = response.1;
                                }
                            }
                        }
                    }
                }
                b'l' => {
                    let his_port_vect: Vec<&str> = value.1.split(':').collect();
                    self.leader_port = his_port_vect[1].to_string();
                    self.leader_address = value.1;
                }
                _ => {
                    println!("No hay mas casos");
                }
            }
        } else {
            self.im_the_leader = self.start_election(&self.leader_address.to_string());
            if self.im_the_leader {
                //soy el lider
                self.communicate_new_leader(self.leader_address.parse().unwrap());
            } else {
                loop {
                    self.udp_receiver
                        .set_timeout(Some(Duration::from_millis(10000)));
                    if let Ok(response) = self.udp_receiver.recv(ElectionMessage::size()) {
                        if response.0[0] == b'l' {
                            let his_port_vect: Vec<&str> = response.1.split(':').collect();
                            self.leader_port = his_port_vect[1].to_string();
                            self.leader_address = response.1;
                        }
                    }
                }
            }
        }
    }

    fn start_election(&mut self, his_address: &str) -> bool {
        let mut im_the_leader = true;
        for port in self.possible_ports.iter() {
            if port.parse::<i32>().unwrap() < self.my_port.parse::<i32>().unwrap() {

                let message = ElectionMessage::build(ElectionCode::Election);
                let his_address_vect: Vec<&str> = his_address.split(':').collect();
                let address_to_send = his_address_vect[0].to_string() + ":" + port;
                let _drop = self
                    .udp_sender
                    .send_to(message.as_slice(), &address_to_send);
                self.udp_receiver
                    .set_timeout(Some(Duration::from_millis(100)));
                if let Ok(_response) = self.udp_receiver.recv(ElectionMessage::size()) {
                    //loggear que me respondieron
                    im_the_leader = false;
                }
            }
        }
        im_the_leader
    }

    fn communicate_new_leader(&mut self, his_address: String) {
        for port in self.possible_ports.iter() {
            let message = ElectionMessage::build(ElectionCode::Leader);
            let his_adr_vect: Vec<&str> = his_address.split(':').collect();
            let adr_to_send = his_adr_vect[0].to_string() + port;
            let _drop = self.udp_sender.send_to(message.as_slice(), &adr_to_send);
        }
        self.leader_port = self.my_port.clone();
    }

    pub fn start_candidate(&mut self) {
        let mut file_iter = FileIterator::new("data/data.csv").unwrap();
        let first_transaction = file_iter.next();
        let true_first_transaction = first_transaction.unwrap();
        let first_trans_cond: CurrentTransaction = Arc::new((
            Mutex::new(Some(Box::new(true_first_transaction))),
            Condvar::new(),
        ));
        let mut port_transaction = 0;
        let mut socket_data_recv = UdpSocketWrap::new(None);
        let mut socket_data_send = UdpSocketWrap::new(None);
        for port in VEC_PORT_DATA.clone() {
            let socket_info_data_new = UdpSocketWrap::new_with_addr(
                Some(Duration::from_millis(1000)),
                DEFAULT_IP.to_string() + port.to_string().as_str(),
            );
            if let Ok(socket_new_aux) = socket_info_data_new {
                socket_data_recv = socket_new_aux;
                if let Ok(socket_aux) = socket_data_recv.try_clone() {
                    socket_data_send = socket_aux;
                    port_transaction = port;
                    break;
                }
            }
        }
        let true_first_trans_cond = first_trans_cond.clone();
        let ended_cvar = Arc::new((Mutex::new(false), Condvar::new()));
        let ended_cvar_clone = ended_cvar.clone();
        thread::spawn(move || {
            let services_addrs_str_recv = &HashMap::from([
                (AIRLINE_ADDR, ServiceName::Airline.string_name()),
                (HOTEL_ADDR, ServiceName::Hotel.string_name()),
                (BANK_ADDR, ServiceName::Bank.string_name()),
            ]);
            let mut transaction_receiver = TransactionReceiver::new(
                Box::new(socket_data_recv),
                services_addrs_str_recv,
                true_first_trans_cond,
                ended_cvar_clone
            );

            loop {
                let _drop = transaction_receiver.recv();
            }
        });
        loop {
            self.send_to();

            if self.im_the_leader {
                break;
            }
        }

        let mut leader = Leader::new(
            VEC_PORT_INFO.clone(),
        );
        let services_addrs_str = &HashMap::from([
            (AIRLINE_ADDR, ServiceName::Airline.string_name()),
            (HOTEL_ADDR, ServiceName::Hotel.string_name()),
            (BANK_ADDR, ServiceName::Bank.string_name()),
        ]);
        let mut vec_addr: Vec<String> = vec![DEFAULT_IP.to_string() + "49353"];
        for port in VEC_PORT_DATA.clone() {
            vec_addr.push(DEFAULT_IP.to_string() + port.to_string().as_str());
        }
        let vec = &vec_addr;
        let transaction_manager = TransactionManager::new(
            port_transaction as u64,
            Box::new(socket_data_send),
            first_trans_cond.clone(),
            ended_cvar,
            services_addrs_str,
            vec,
            Duration::from_millis(10000),
            Some(ABORT_FILE.to_string())
        );
        leader.start_leader(transaction_manager, &mut self.udp_receiver, &mut self.udp_sender);
    }
}
