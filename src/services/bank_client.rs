use crate::sockets::udp_socket_receiver::UdpSocketReceiver;
use crate::sockets::udp_socket_sender::UdpSocketSender;
use crate::transaction_messages::transaction_code::TransactionCode;
use crate::transaction_messages::transaction_info::TransactionInfo;
use crate::transaction_messages::transaction_request::TransactionRequest;
use crate::transaction_messages::transaction_response::TransactionResponse;
use std::convert::TryInto;
use std::mem::size_of;

use super::common_client::CommonClient;

#[allow(dead_code)]
pub struct Bank {
    socket_receiver: Box<dyn UdpSocketReceiver+ Send>,
    socket_sender: Box<dyn UdpSocketSender + Send>,
    fee_sum: f64,
    addr: String,
}
impl Bank {
    pub fn new(
        socket_receiver: Box<dyn UdpSocketReceiver+ Send>,
        socket_sender: Box<dyn UdpSocketSender+ Send>,
        addr: String,
    ) -> Bank {
        Bank {
            socket_sender,
            socket_receiver,
            fee_sum: 0.0,
            addr,
        }
    }
}

impl CommonClient for Bank {
    fn answer_message(&mut self, vector: Vec<u8>, addr_to_answer:String) {
        /*let mut rng = rand::thread_rng();
        if rng > 0.2{
            let mut response= TransactionResponse::build(TransactionCode::Abort,
                                                      transaction_id);
            let addr = self.addr.clone();
            let _ =self.socket_sender.send_to(&*response, &addr);
        }*/
        let code = vector[0];
        if code == TransactionRequest::map_transaction_code(TransactionCode::Prepare) {
            println!("Soy buena aerolinea y respondo el Prepare");
            let id_bytes: [u8; size_of::<u64>()] = vector[1..size_of::<u64>() + 1]
                .try_into()
                .expect("[Client] Los ids deberian ocupar 8 bytes");
            let transaction_id = u64::from_be_bytes(id_bytes);
            let mut response = TransactionResponse::build(TransactionCode::Accept, transaction_id);
            TransactionInfo::add_padding(&mut response);
            let _ = self.socket_sender.send_to(&*response, &addr_to_answer);
        } else if code == TransactionRequest::map_transaction_code(TransactionCode::Abort) {
            let id_bytes: [u8; size_of::<u64>()] = vector[1..size_of::<u64>() + 1]
                .try_into()
                .expect("[Client] Los ids deberian ocupar 8 bytes");
            let transaction_id = u64::from_be_bytes(id_bytes);
            let mut response = TransactionResponse::build(TransactionCode::Abort, transaction_id);
            TransactionInfo::add_padding(&mut response);
            let fee: [u8; size_of::<f64>()] = vector[size_of::<u64>() + 1..]
                .try_into()
                .expect("[Client] Los fee deberian ocupar size_of::<f64> bytes");
            let fee_value = f64::from_be_bytes(fee);
            self.fee_sum -= fee_value;
            let _ = self.socket_sender.send_to(&*response, &addr_to_answer);
        } else {
            let id_bytes: [u8; size_of::<u64>()] = vector[1..size_of::<u64>() + 1]
                .try_into()
                .expect("[Client] Los ids deberian ocupar 8 bytes");
            let transaction_id = u64::from_be_bytes(id_bytes);
            let mut response = TransactionResponse::build(TransactionCode::Commit, transaction_id);
            TransactionInfo::add_padding(&mut response);
            let fee: [u8; size_of::<f64>()] = vector[size_of::<u64>() + 1..]
                .try_into()
                .expect("[Client] Los fee deberian ocupar size_of::<f64> bytes");
            let fee_value = f64::from_be_bytes(fee);
            self.fee_sum += fee_value;
            let _ = self.socket_sender.send_to(&*response, &addr_to_answer);
        }
    }

    fn start_client(&mut self){
        loop {
            let _ =self.process_one_transaction();
        }

    }

    fn process_one_transaction(&mut self) -> Result<i64, String> {
        let res = self.socket_receiver.recv(TransactionRequest::size());
        let res_vec = res.unwrap();
        let res_vector = res_vec.0;
        let addr_to_answer = res_vec.1;
        self.answer_message(res_vector,addr_to_answer);
        Ok(0)
    }

    fn get_fee_sum(&mut self) -> f64 {
        self.fee_sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::services::bank_client::Bank;
    use crate::services::common_client::CommonClient;
    use crate::transaction_messages::transaction_code::TransactionCode;
    use crate::transaction_messages::transaction_request::TransactionRequest;
    use ntest::timeout;

    use crate::sockets::udp_socket_receiver::MockUdpSocketReceiver;
    use crate::sockets::udp_socket_sender::MockUdpSocketSender;
    use crate::transaction_messages::transaction_response::TransactionResponse;

    #[test]
    #[timeout(3000)]
    fn it_should_return_accept_when_receives_prepare() {
        let bank_addr = "127.0.0.1:49157";
        let transaction_id = 0;
        let bank_fee = 100.0;
        let first_msg =
            TransactionRequest::build(TransactionCode::Prepare, transaction_id, bank_fee);
        let mut response = TransactionResponse::build(TransactionCode::Accept, transaction_id);
        TransactionInfo::add_padding(&mut response);
        let first_msg_len = first_msg.len();

        let mut mock_socket_sender = MockUdpSocketSender::new();
        mock_socket_sender
            .expect_send_to()
            .withf(move |buff, addr| buff.to_vec() == response && addr == bank_addr)
            .times(1)
            .returning(|_, _| Ok(()));

        let mut mock_socket_receiver = MockUdpSocketReceiver::new();
        mock_socket_receiver
            .expect_recv()
            .withf(move |n| n == &first_msg_len)
            .times(1)
            .returning(move |_| Ok((first_msg.clone(), bank_addr.to_string())));

        let mut bank = Bank::new(
            Box::new(mock_socket_receiver),
            Box::new(mock_socket_sender),
            bank_addr.to_string(),
        );

        let _ = bank.process_one_transaction();
    }

    #[test]
    #[timeout(3000)]
    fn it_should_return_accept_when_receives_abort() {
        let bank_addr = "127.0.0.1:49157";
        let transaction_id = 0;
        let bank_fee = 100.0;
        let first_msg = TransactionRequest::build(TransactionCode::Abort, transaction_id, bank_fee);
        let mut response = TransactionResponse::build(TransactionCode::Accept, transaction_id);
        TransactionInfo::add_padding(&mut response);
        let first_msg_len = first_msg.len();

        let mut mock_socket_sender = MockUdpSocketSender::new();
        mock_socket_sender
            .expect_send_to()
            .withf(move |buff, addr| buff.to_vec() == response && addr == bank_addr)
            .times(1)
            .returning(|_, _| Ok(()));

        let mut mock_socket_receiver = MockUdpSocketReceiver::new();
        mock_socket_receiver
            .expect_recv()
            .withf(move |n| n == &first_msg_len)
            .times(1)
            .returning(move |_| Ok((first_msg.clone(), bank_addr.to_string())));

        let mut bank = Bank::new(
            Box::new(mock_socket_receiver),
            Box::new(mock_socket_sender),
            bank_addr.to_string(),
        );

        let _ = bank.process_one_transaction();
    }

    #[test]
    #[timeout(3000)]
    fn it_should_return_accept_when_receives_commit() {
        let bank_addr = "127.0.0.1:49157";
        let transaction_id = 0;
        let bank_fee = 100.0;
        let first_msg =
            TransactionRequest::build(TransactionCode::Commit, transaction_id, bank_fee);
        let mut response = TransactionResponse::build(TransactionCode::Accept, transaction_id);
        TransactionInfo::add_padding(&mut response);
        let first_msg_len = first_msg.len();

        let mut mock_socket_sender = MockUdpSocketSender::new();
        mock_socket_sender
            .expect_send_to()
            .withf(move |buff, addr| buff.to_vec() == response && addr == bank_addr)
            .times(1)
            .returning(|_, _| Ok(()));

        let mut mock_socket_receiver = MockUdpSocketReceiver::new();
        mock_socket_receiver
            .expect_recv()
            .withf(move |n| n == &first_msg_len)
            .times(1)
            .returning(move |_| Ok((first_msg.clone(), bank_addr.to_string())));

        let mut bank = Bank::new(
            Box::new(mock_socket_receiver),
            Box::new(mock_socket_sender),
            bank_addr.to_string(),
        );

        let _ = bank.process_one_transaction();
    }

    #[test]
    #[timeout(3000)]
    fn it_should_change_fee_when_receives_commit() {
        let bank_addr = "127.0.0.1:49157";
        let transaction_id = 0;
        let bank_fee = 100.0;
        let first_msg =
            TransactionRequest::build(TransactionCode::Commit, transaction_id, bank_fee);
        let mut response = TransactionResponse::build(TransactionCode::Accept, transaction_id);
        TransactionInfo::add_padding(&mut response);
        let first_msg_len = first_msg.len();

        let mut mock_socket_sender = MockUdpSocketSender::new();
        mock_socket_sender
            .expect_send_to()
            .withf(move |buff, addr| buff.to_vec() == response && addr == bank_addr)
            .times(1)
            .returning(|_, _| Ok(()));

        let mut mock_socket_receiver = MockUdpSocketReceiver::new();
        mock_socket_receiver
            .expect_recv()
            .withf(move |n| n == &first_msg_len)
            .times(1)
            .returning(move |_| Ok((first_msg.clone(), bank_addr.to_string())));

        let mut bank = Bank::new(
            Box::new(mock_socket_receiver),
            Box::new(mock_socket_sender),
            bank_addr.to_string(),
        );

        let _ = bank.process_one_transaction();
        assert_eq!(bank.fee_sum, 100.0)
    }
}
