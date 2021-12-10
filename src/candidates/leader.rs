
use crate::sockets::udp_socket_receiver::UdpSocketReceiver;
use crate::candidates::election_message::ElectionMessage;
use crate::sockets::udp_socket_sender::UdpSocketSender;

#[allow(dead_code)]
pub struct Leader{
    udp_receiver: Box<dyn UdpSocketReceiver>,
    udp_sender: Box<dyn UdpSocketSender>,
}

impl Leader {
    #[must_use]
    pub fn new(udp_receiver: Box<dyn UdpSocketReceiver>,udp_sender: Box<dyn UdpSocketSender>)->Self{
        Leader{
            udp_receiver,
            udp_sender,
        }
    }

    pub fn recv(& mut self){
        let result = self.udp_receiver.recv(ElectionMessage::size());
        let (_message,_address) = match result{
            Ok(value) => value,
            Err(_err) => return 
        };
        let result_send = self.udp_sender.send_to(_message.as_slice(),&_address);
        match result_send{
            Ok(var) => var,
            Err(_err) => return 
        };

    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        sockets::udp_socket_receiver::MockUdpSocketReceiver,
        sockets::udp_socket_sender::MockUdpSocketSender,
        candidates::election_code::ElectionCode,
    };

    #[test]
    fn it_should_receive_alive_message(){
        let address = "127.0.0.1:49156";
        let mut mock_receiver = MockUdpSocketReceiver::new();
        let mut mock_sender = MockUdpSocketSender::new();
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
        let mut leader = Leader::new(Box::new(mock_receiver),Box::new(mock_sender));
        leader.recv();

    }
}