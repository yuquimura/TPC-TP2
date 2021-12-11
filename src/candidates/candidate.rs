
use crate::sockets::udp_socket_receiver::UdpSocketReceiver;
use crate::candidates::election_message::ElectionMessage;
use crate::sockets::udp_socket_sender::UdpSocketSender;
use crate::candidates::election_code::ElectionCode;
use std::time::Duration;


#[allow(dead_code)]
pub struct Candidate{
    udp_receiver: Box<dyn UdpSocketReceiver>,
    udp_sender: Box<dyn UdpSocketSender>,
    my_port: String,
    possible_ports: Vec<String>,
    leader_port: String
}

impl Candidate {
    #[must_use]
    pub fn new(udp_receiver: Box<dyn UdpSocketReceiver>,udp_sender: Box<dyn UdpSocketSender>,
        my_port: String, possible_ports: Vec<String>,leader_port: String)->Self{
        Candidate{
            udp_receiver,
            udp_sender,
            my_port,
            possible_ports,
            leader_port,
        }
    }

    pub fn send_to(& mut self,addr: &str){
        let message = ElectionMessage::build(ElectionCode::Alive);        
        let _ = self.udp_sender.send_to(message.as_slice(),addr);
        self.udp_receiver.set_timeout(Some(Duration::from_millis(1000)));        
        if let Ok(value) = self.udp_receiver.recv(ElectionMessage::size()) {
            match value.0[0] {                 
                b'v'=>{
                    self.udp_receiver.set_timeout(Some(Duration::from_millis(10000)));
                    if let Ok(response) = self.udp_receiver.recv(ElectionMessage::size()){                      
                        let his_address = response.1;                             
                        let _ = self.udp_sender.send_to(message.as_slice(),&his_address);              
                        let im_the_leader =  self.start_election(&his_address);
                        if im_the_leader{
                            //soy el lider
                            self.communicate_new_leader(his_address)
                        }
                    }
                }                
                //contemplar que pasa cuando llega un mensaje de election y tengo que contestar OK, como se que no soy el lider?
                b'e' => {  
                    let his_address = value.1;                             
                    let _ = self.udp_sender.send_to(message.as_slice(),&his_address);
                    let im_the_leader =  self.start_election(&his_address);
                    self.start_election(&his_address);
                    if im_the_leader{
                        //soy el lider
                        self.communicate_new_leader(his_address);
                    }
                    else{
                        self.udp_receiver.set_timeout(Some(Duration::from_millis(10000)));
                        if let Ok(_response) = self.udp_receiver.recv(ElectionMessage::size()){
                            let his_port_vect: Vec<&str> = his_address.split(':').collect();
                            self.leader_port = his_port_vect[1].to_string();
                        }
                    }
                    
                }
                b'l' => {
                    let his_port_vect: Vec<&str> = value.1.split(':').collect();
                    self.leader_port = his_port_vect[1].to_string();
                }
                _ => {
                    println!("No hay mas casos");
                }
            }  
            
        } else {
            let im_the_leader =  self.start_election(&addr.to_string());
            if im_the_leader{
                //soy el lider
                self.communicate_new_leader(addr.parse().unwrap());

            }
            else{
                self.udp_receiver.set_timeout(Some(Duration::from_millis(10000)));
                if let Ok(_response) = self.udp_receiver.recv(ElectionMessage::size()){
                    let his_port_vect: Vec<&str> = addr.split(':').collect();
                    self.leader_port = his_port_vect[1].to_string();
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