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
pub struct Bank {
    socket_receiver: Box<dyn UdpSocketReceiver>,
    socket_sender: Box<dyn UdpSocketSender>,
    fee_sum: f64,
    addr:String
}
impl Bank {
    pub fn new(
        socket_receiver: Box<dyn UdpSocketReceiver>,
        socket_sender: Box<dyn UdpSocketSender>,
        addr: String
    ) -> Bank {
        Bank {
            socket_sender,
            socket_receiver,
            fee_sum: 0.0,
            addr
        }
    }
}

impl CommonClient for Bank {
    fn answer_message(&mut self, vector: Vec<u8>) {
        let code= vector[0];
        if code == TransactionRequest::map_transaction_code(TransactionCode::Prepare){
            let id_bytes: [u8; size_of::<u64>()] = vector[1..size_of::<u64>()+1]
                .try_into()
                .expect("[Client] Los ids deberian ocupar 8 bytes");
            let transaction_id = u64::from_be_bytes(id_bytes);
            let response = TransactionResponse::build(TransactionCode::Accept,
                                                      transaction_id);
            let addr = self.addr.clone();
            let _ =self.socket_sender.send_to(&*response, &addr);
        }
        else if  code == TransactionRequest::map_transaction_code(TransactionCode::Abort) {
            let id_bytes: [u8; size_of::<u64>()] = vector[1..size_of::<u64>()+1]
                .try_into()
                .expect("[Client] Los ids deberian ocupar 8 bytes");
            let transaction_id = u64::from_be_bytes(id_bytes);
            let response =TransactionResponse::build(TransactionCode::Accept,
                                                     transaction_id);
            let fee: [u8; size_of::<f64>()] = vector[size_of::<u64>()+1..]
                .try_into()
                .expect("[Client] Los fee deberian ocupar size_of::<f64> bytes");
            let fee_value = f64::from_be_bytes(fee);
            self.fee_sum -= fee_value;
            let addr = self.addr.clone();
            let _ =self.socket_sender.send_to(&*response, &addr);
        }
        else {
            let id_bytes: [u8; size_of::<u64>()] = vector[1.. size_of::<u64>()+1]
                .try_into()
                .expect("[Client] Los ids deberian ocupar 8 bytes");
            let transaction_id = u64::from_be_bytes(id_bytes);
            let response =TransactionResponse::build(TransactionCode::Accept,
                                                     transaction_id);
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
                .recv(TransactionRequest::size());
            let res_vec = res.unwrap().0;
            if res_vec[0].to_string() == "q" {
                break;
            } else {
                self.answer_message(res_vec);
            }
        }
        0
    }


    fn process_one_transaction(&mut self)->Result<i64, String>{
        let res = self
            .socket_receiver
            .recv(TransactionRequest::size());
        let res_vec = res.unwrap().0;
        self.answer_message(res_vec);
        Ok(0)

    }

    fn get_fee_sum(&mut self) -> f64 {
        self.fee_sum
    }
}

#[cfg(test)]
mod tests {

    use ntest::timeout;
    use crate::services::bank_client::Bank;
    use crate::services::common_client::CommonClient;
    use crate::transactions::transaction_code::TransactionCode;
    use crate::transactions::transaction_request::TransactionRequest;

    use crate::sockets::udp_socket_receiver::MockUdpSocketReceiver;
    use crate::sockets::udp_socket_sender::MockUdpSocketSender;
    use crate::transactions::transaction_response::TransactionResponse;

    #[test]
    #[timeout(3000)]
    fn it_should_return_accept_when_receives_prepare() {
        let bank_addr = "127.0.0.1:49157";
        let transaction_id = 0;
        let bank_fee = 100.0;
        let first_msg = TransactionRequest::build(TransactionCode::Prepare,
                                                  transaction_id,
                                                  bank_fee);
        let response = TransactionResponse::build(TransactionCode::Accept,
                                                  transaction_id);
        let first_msg_len = first_msg.len();

        let mut mock_socket_sender = MockUdpSocketSender::new();
        mock_socket_sender.expect_send_to()
            .withf(move |buff, addr| buff.to_vec() == response && addr==bank_addr).times(1).returning(|_, _| Ok(()));

        let mut mock_socket_receiver = MockUdpSocketReceiver::new();
        mock_socket_receiver.expect_recv()
            .withf(move |n| n==&first_msg_len).times(1).returning(move|_|Ok((first_msg.clone(),bank_addr.to_string())));

        let mut bank = Bank::new(Box::new(mock_socket_receiver),
                                       Box::new(mock_socket_sender),
                                    bank_addr.to_string());

        let _ =bank.process_one_transaction();
    }

    #[test]
    #[timeout(3000)]
    fn it_should_return_accept_when_receives_abort() {
        let bank_addr = "127.0.0.1:49157";
        let transaction_id = 0;
        let bank_fee = 100.0;
        let first_msg = TransactionRequest::build(TransactionCode::Abort,
                                                  transaction_id,
                                                  bank_fee);
        let response = TransactionResponse::build(TransactionCode::Accept,
                                                  transaction_id);
        let first_msg_len = first_msg.len();

        let mut mock_socket_sender = MockUdpSocketSender::new();
        mock_socket_sender.expect_send_to()
            .withf(move |buff, addr| buff.to_vec() == response && addr==bank_addr).times(1).returning(|_, _| Ok(()));

        let mut mock_socket_receiver = MockUdpSocketReceiver::new();
        mock_socket_receiver.expect_recv()
            .withf(move |n| n==&first_msg_len).times(1).returning(move|_|Ok((first_msg.clone(),bank_addr.to_string())));

        let mut bank = Bank::new(Box::new(mock_socket_receiver),
                                       Box::new(mock_socket_sender),
                                    bank_addr.to_string());

        let _ =bank.process_one_transaction();
    }

    #[test]
    #[timeout(3000)]
    fn it_should_return_accept_when_receives_commit() {
        let bank_addr = "127.0.0.1:49157";
        let transaction_id = 0;
        let bank_fee = 100.0;
        let first_msg = TransactionRequest::build(TransactionCode::Commit,
                                                  transaction_id,
                                                  bank_fee);
        let response = TransactionResponse::build(TransactionCode::Accept,
                                                  transaction_id);
        let first_msg_len = first_msg.len();

        let mut mock_socket_sender = MockUdpSocketSender::new();
        mock_socket_sender.expect_send_to()
            .withf(move |buff, addr| buff.to_vec() == response && addr==bank_addr).times(1).returning(|_, _| Ok(()));

        let mut mock_socket_receiver = MockUdpSocketReceiver::new();
        mock_socket_receiver.expect_recv()
            .withf(move |n| n==&first_msg_len).times(1).returning(move|_|Ok((first_msg.clone(),bank_addr.to_string())));

        let mut bank = Bank::new(Box::new(mock_socket_receiver),
                                       Box::new(mock_socket_sender),
                                    bank_addr.to_string());

        let _ =bank.process_one_transaction();
    }

    #[test]
    #[timeout(3000)]
    fn it_should_change_fee_when_receives_commit() {
        let bank_addr = "127.0.0.1:49157";
        let transaction_id = 0;
        let bank_fee = 100.0;
        let first_msg = TransactionRequest::build(TransactionCode::Commit,
                                                  transaction_id,
                                                  bank_fee);
        let response = TransactionResponse::build(TransactionCode::Accept,
                                                  transaction_id);
        let first_msg_len = first_msg.len();

        let mut mock_socket_sender = MockUdpSocketSender::new();
        mock_socket_sender.expect_send_to()
            .withf(move |buff, addr| buff.to_vec() == response && addr==bank_addr).times(1).returning(|_, _| Ok(()));

        let mut mock_socket_receiver = MockUdpSocketReceiver::new();
        mock_socket_receiver.expect_recv()
            .withf(move |n| n==&first_msg_len).times(1).returning(move|_|Ok((first_msg.clone(),bank_addr.to_string())));

        let mut bank = Bank::new(Box::new(mock_socket_receiver),
                                       Box::new(mock_socket_sender),
                                    bank_addr.to_string());

        let _ =bank.process_one_transaction();
        assert_eq!(bank.fee_sum,100.0)
    }
}
