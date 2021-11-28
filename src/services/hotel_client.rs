use super::common_client::CommonClient;
use crate::sockets::udp_socket_wrap::UdpSocketWrap;
use crate::transactions::transaction_response::TransactionResponse;
use crate::sockets::constants::UDP_PACKET_SIZE;
use crate::sockets::udp_socket_receiver::UdpSocketReceiver;
use crate::sockets::udp_socket_sender::UdpSocketSender;

pub struct HotelClient {
    #[allow(dead_code)]
    communication_socket: UdpSocketWrap,
    fee_sum: i64,
    transaction_response: TransactionResponse
}
impl HotelClient{
    pub fn new(transaction_response:TransactionResponse) -> HotelClient {
        HotelClient{
            communication_socket:UdpSocketWrap::new(None),
            fee_sum:0,
            transaction_response
        }
    }
}

impl CommonClient for HotelClient {
    fn answer_message(&mut self, vector: Vec<u8>) {

    }
    fn rollback_message(&mut self,) {
        todo!()
    }

    fn start_client(&mut self, string:&str)->i64 {

        loop {
            let res = self.communication_socket.recv(UDP_PACKET_SIZE);
            let res_vec = res.unwrap().0;
            if res_vec[0].to_string() == "q" {
                break;
            } else {
                self.answer_message(res_vec);
            }
        }
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use std::net::UdpSocket;
    use crate::services::common_client::CommonClient;
    use crate::services::hotel_client::HotelClient;
    use crate::transactions::transaction_response::TransactionResponse;

    #[test]
    fn it_should_break_the_loop_when_it_receives_a_q(){
        let addr = "192.168.0.106/24";
        let socket = UdpSocket::bind(addr).unwrap();
        let message = "q".as_bytes().to_vec();
        let _ = socket.send_to(&message,addr);
        let mut hotel_client = HotelClient::new(TransactionResponse);
        let return_value = hotel_client.start_client(addr);
        assert_eq!(return_value,0)
    }
}