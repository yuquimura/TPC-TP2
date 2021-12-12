use std::thread;
use crate::sockets::udp_socket_receiver::UdpSocketReceiver;
use crate::candidates::election_message::ElectionMessage;
use crate::sockets::udp_socket_sender::UdpSocketSender;
use crate::candidates::election_code::ElectionCode;
use std::time::Duration;
use rand::Rng;
use crate::alglobo::transaction_manager::TransactionManager;
use crate::candidates::constants::{DEFAULT_IP, EMPTY, VEC_PORT_DATA, VEC_PORT_INFO};
use crate::candidates::leader::Leader;
use crate::sockets::udp_socket_wrap::UdpSocketWrap;


#[allow(dead_code)]
pub struct Candidate{
    udp_receiver: Box<dyn UdpSocketReceiver>,
    udp_sender: Box<dyn UdpSocketSender>,
    my_port: String,
    possible_ports: Vec<String>,
    leader_port: String,
    leader_address: String,
    im_the_leader: bool,
    udp_receiver_info: Box<dyn UdpSocketReceiver>,
    udp_sender_info: Box<dyn UdpSocketSender>,
}

impl Candidate {
    #[must_use]
    pub fn new(udp_receiver: Box<dyn UdpSocketReceiver>,udp_sender: Box<dyn UdpSocketSender>,
               udp_receiver_info: Box<dyn UdpSocketReceiver>,udp_sender_info: Box<dyn UdpSocketSender>,
        my_port: String, possible_ports: Vec<String>,leader_port: String, 
        leader_address: String)->Self{
        let im_the_leader = false;
        Candidate{
            udp_receiver,
            udp_sender,
            my_port,
            possible_ports,
            leader_port,
            leader_address,
            im_the_leader,
            udp_sender_info,
            udp_receiver_info
        }
    }

    pub fn send_to(& mut self){
        if self.leader_port == EMPTY {
            self.start_election(&DEFAULT_IP.to_string());
            return;
        }
        let message = ElectionMessage::build(ElectionCode::Alive);        
        let _ = self.udp_sender.send_to(message.as_slice(),&self.leader_address);
        self.udp_receiver.set_timeout(Some(Duration::from_millis(1000)));        
        if let Ok(value) = self.udp_receiver.recv(ElectionMessage::size()) {
            match value.0[0] {                 
                b'v'=>{
                    self.udp_receiver.set_timeout(Some(Duration::from_millis(10000)));
                    if let Ok(response) = self.udp_receiver.recv(ElectionMessage::size()){                      
                        let his_address = response.1;                             
                        let _ = self.udp_sender.send_to(message.as_slice(),&his_address);              
                        self.im_the_leader =  self.start_election(&his_address);
                        if self.im_the_leader{
                            //soy el lider
                            self.communicate_new_leader(his_address)
                        }
                    }
                }                
                //contemplar que pasa cuando llega un mensaje de election y tengo que contestar OK, como se que no soy el lider?
                b'e' => {  
                    let his_address = value.1;                             
                    let _ = self.udp_sender.send_to(message.as_slice(),&his_address);
                    self.im_the_leader =  self.start_election(&his_address);
                    self.start_election(&his_address);
                    if self.im_the_leader{
                        //soy el lider
                        self.communicate_new_leader(his_address);
                    }
                    else{
                        loop{
                            self.udp_receiver.set_timeout(Some(Duration::from_millis(10000)));
                            if let Ok(response) = self.udp_receiver.recv(ElectionMessage::size()){
                                if response.0[0] == b'l'{
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
            self.im_the_leader =  self.start_election(&self.leader_address.to_string());
            if self.im_the_leader{
                //soy el lider
                self.communicate_new_leader(self.leader_address.parse().unwrap());

            }
            else{
                loop{
                    self.udp_receiver.set_timeout(Some(Duration::from_millis(10000)));
                    if let Ok(response) = self.udp_receiver.recv(ElectionMessage::size()){
                        if response.0[0] == b'l'{
                            let his_port_vect: Vec<&str> = response.1.split(':').collect();
                            self.leader_port = his_port_vect[1].to_string();
                            self.leader_address = response.1;

                        } 
                    }
                }
                
            }
        }
    }

    fn start_election(&mut self, his_address: &String) ->bool {
        let mut im_the_leader = true;
        for port in self.possible_ports.iter() {
            if port.parse::<i32>().unwrap() > self.my_port.parse::<i32>().unwrap() {
                let message = ElectionMessage::build(ElectionCode::Election);
                let his_address_vect: Vec<&str> = his_address.split(':').collect();
                let address_to_send = his_address_vect[0].to_string() + port;
                let _ = self.udp_sender.send_to(message.as_slice(), &address_to_send);
                self.udp_receiver.set_timeout(Some(Duration::from_millis(1000)));
                if let Ok(_response) = self.udp_receiver.recv(ElectionMessage::size()) {
                    //loggear que me respondieron
                    im_the_leader = false;
                }
            }
        }
        return im_the_leader;
    }

    fn communicate_new_leader(&mut self, his_address: String) {
        for port in self.possible_ports.iter() {
            let message = ElectionMessage::build(ElectionCode::Leader);
            let his_adr_vect: Vec<&str> = his_address.split(':').collect();
            let adr_to_send = his_adr_vect[0].to_string() + port;
            let _ = self.udp_sender.send_to(message.as_slice(), &adr_to_send);
        }
    }

    fn start_candidate(&mut self,){
        loop {
            self.send_to();
            if self.im_the_leader {
                break;
            }
        }
        for port in VEC_PORT_DATA {
            let socket_data_recv = UdpSocketWrap::new_with_addr(None, port);
            if let Ok(result_socket_recv) = socket_data_recv {
                let socket_cloned_result = result_socket_recv.try_clone();
                if let Ok(socket_data_clone) = socket_cloned_result {
                    break;
                }
            }
        }
        for port in VEC_PORT_INFO {
            let socket_info_recv = UdpSocketWrap::new_with_addr(None, port);
            if let Ok(result_socket_recv) = socket_info_recv {
                let socket_info_cloned_result = result_socket_recv.try_clone();
                if let Ok(socket_info_clone) = socket_info_cloned_result {
                    break;
                }
            }
        }
        /*
        let mut leader = Leader::new(Box::new(socket_wrap_recv), Box::new(socket_wrap_send), vect_port);

        let transaction_manager = TransactionManager::new(0,)
        leader.start_leader();
*/


        thread::spawn(move || {
            loop{
                //transaction receiver.recv

            }
        });
    }
}



// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         sockets::udp_socket_receiver::MockUdpSocketReceiver,
//         sockets::udp_socket_sender::MockUdpSocketSender,
//         candidates::election_code::ElectionCode,
//     };

//     #[test]
//     fn it_should_receive_alive_message(){
//         let address = "127.0.0.1:49156";
//         let mut mock_receiver = MockUdpSocketReceiver::new();
//         let mut mock_sender = MockUdpSocketSender::new();
//         let message = ElectionMessage::build(ElectionCode::Alive);
//         let messages = [message.clone()];
//         mock_receiver
//             .expect_recv()
//             .withf(|n_bytes| n_bytes == &ElectionMessage::size())
//             .times(1)
//             .returning(move |_| Ok((message.clone(),address.to_string())));    
//         mock_sender
//             .expect_send_to()
//             .withf(move |buf, addr| {
//                 messages.contains(&buf.to_vec()) && addr == address
//             })
//             .times(1)
//             .returning(|_, _| Ok(()));
//         let duration = Duration::from_millis(0);
//         let mut candidate = Candidate::new(Box::new(mock_receiver),Box::new(mock_sender),duration);
//         candidate.send_to(&address);

//     }
// }