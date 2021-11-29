use crate::sockets::udp_socket_receiver::UdpSocketReceiver;
use crate::sockets::udp_socket_sender::UdpSocketSender;
use crate::{
    transactions::transaction_response::TransactionResponse,
};

use super::common_service::CommonService;

#[allow(dead_code)]
pub struct Hotel {
    // Para los tests usar de template transaction_manager.
    socket_receiver: Box<dyn UdpSocketReceiver>,
    socket_sender: Box<dyn UdpSocketSender>,
    fee_sum: f64,
    transaction_response: TransactionResponse,
}
impl Hotel {
    pub fn new(
        socket_receiver: Box<dyn UdpSocketReceiver>,
        socket_sender: Box<dyn UdpSocketSender>,
        transaction_response: TransactionResponse
    ) -> Hotel {
        Hotel {
            socket_sender,
            socket_receiver,
            fee_sum: 0.0,
            transaction_response,
        }
    }
}

impl CommonService for Hotel {
    fn answer_message(&mut self, _vector: Vec<u8>) {}
    fn rollback_message(&mut self) {}

    fn start_client(&mut self, _string: &str) -> i64 {
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

#[cfg(test)]
mod tests {

    use ntest::timeout;
    use crate::transactions::transaction_code::TransactionCode;
    use crate::transactions::transaction_request::TransactionRequest;

    use crate::sockets::udp_socket_receiver::MockUdpSocketReceiver;
    use crate::sockets::udp_socket_sender::MockUdpSocketSender;

    #[test]
    #[timeout(3000)]
    fn it_should_break_the_loop_when_it_receives_a_q() {
        let hotel_addr = "127.0.0.1:49157";
        let transaction_id = 0;
        let hotel_fee = 200.0;
        let mut mock_socket_sender = MockUdpSocketSender::new();
        let mut mock_socket_receiver = MockUdpSocketReceiver::new();
        let addresses = [ hotel_addr];
        let transaction_message =[
            TransactionRequest::build(TransactionCode::Quit, transaction_id, hotel_fee)];
        let addresses_clone = addresses;
        let messages_clone = transaction_message;
        mock_socket_sender
            .expect_send_to()
            .withf(move |buf, addr| {
                messages_clone.contains(&buf.to_vec()) && addresses_clone.contains(&addr)
            })
            .times(1)
            .returning(|_, _| Ok(()));
        mock_socket.expect_recv().with(...).returning(...)

        assert_eq!(0, 0)
    }
}
