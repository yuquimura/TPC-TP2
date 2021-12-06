use std::mem::size_of;
use crate::sockets::udp_socket_receiver::UdpSocketReceiver;
use crate::sockets::udp_socket_sender::UdpSocketSender;
use crate::{
    transactions::transaction_response::TransactionResponse,
};
use std::convert::TryInto;
use crate::transactions::transaction_code::TransactionCode;
use crate::transactions::transaction_request::TransactionRequest;

use super::common_client::CommonClient;

#[allow(dead_code)]
pub struct Hotel {
    socket_receiver: Box<dyn UdpSocketReceiver>,
    socket_sender: Box<dyn UdpSocketSender>,
    fee_sum: f64,
    transaction_response: TransactionResponse,
    addr:String
}
impl Hotel {
    pub fn new(
        socket_receiver: Box<dyn UdpSocketReceiver>,
        socket_sender: Box<dyn UdpSocketSender>,
        transaction_response: TransactionResponse,
        addr:String
    ) -> Hotel {
        Hotel {
            socket_sender,
            socket_receiver,
            fee_sum: 0.0,
            transaction_response,
            addr
        }
    }
}

impl CommonClient for Hotel {
    fn answer_message(&mut self, vector: Vec<u8>) {
        let code= vector[0];
        if code == TransactionRequest::map_transaction_code(TransactionCode::Prepare){
            let id_bytes: [u8; size_of::<u64>()] = vector[1..]
                .try_into()
                .expect("[Client] Los ids deberian ocupar 8 bytes");
            let transaction_id = u64::from_be_bytes(id_bytes);
            let response = TransactionResponse::build(TransactionCode::Accept, transaction_id);
            let addr = self.addr.clone();
            let _ =self.socket_sender.send_to(&*response, &addr);
        }
        else if  code == TransactionRequest::map_transaction_code(TransactionCode::Abort) {
            let id_bytes: [u8; size_of::<u64>()] = vector[1..]
                .try_into()
                .expect("[Client] Los ids deberian ocupar 8 bytes");
            let transaction_id = u64::from_be_bytes(id_bytes);
            let response =TransactionResponse::build(TransactionCode::Accept, transaction_id);
            let fee: [u8; size_of::<f64>()] = vector[size_of::<u64>()+1..]
                .try_into()
                .expect("[Client] Los fee deberian ocupar size_of::<f64> bytes");
            let fee_value = f64::from_be_bytes(fee);
            self.fee_sum += fee_value;
            let addr = self.addr.clone();
            let _ =self.socket_sender.send_to(&*response, &addr);
        }
        else {
            let id_bytes: [u8; size_of::<u64>()] = vector[1..]
                .try_into()
                .expect("[Client] Los ids deberian ocupar 8 bytes");
            let transaction_id = u64::from_be_bytes(id_bytes);
            let response =TransactionResponse::build(TransactionCode::Accept, transaction_id);
            let fee: [u8; size_of::<f64>()] = vector[size_of::<u64>()+1..]
                .try_into()
                .expect("[Client] Los fee deberian ocupar size_of::<f64> bytes");
            let fee_value = f64::from_be_bytes(fee);
            self.fee_sum += fee_value;
            let addr = self.addr.clone();
            let _ =self.socket_sender.send_to(&*response, &addr);
        }
    }


    fn start_client(&mut self) -> i64 {
        loop {
            let res = self
                .socket_receiver
                .recv(TransactionResponse::size());
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

/*#[cfg(test)]
mod tests {

    use ntest::timeout;
    use crate::transactions::transaction_code::TransactionCode;
    use crate::transactions::transaction_request::TransactionRequest;

    use crate::sockets::udp_socket_receiver::MockUdpSocketReceiver;
    use crate::sockets::udp_socket_sender::MockUdpSocketSender;
    use crate::transactions::transaction_response::TransactionResponse;

    #[test]
    #[timeout(3000)]
    fn it_should_return_accept_when_receives_prepare() {
        let hotel_addr = "127.0.0.1:49157";
        let transaction_id = 0;
        let hotel_fee = 100.0;
        let first_msg = TransactionRequest::build(TransactionCode::Prepare,transaction_id,hotel_fee);
        let first_msg_len = first_msg.len();

        let mut mock_socket_sender = MockUdpSocketSender::new();
        mock_socket_sender.expect_send_to()
            .withf(move |n_bytes| n_bytes == &first_msg_len).times(2).returning(|_, _| Ok(()));

    }
}*/
