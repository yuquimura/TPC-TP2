use crate::{
    alglobo::transactionable::Transactionable, sockets::udp_socket_sender::UdpSocketSender,
};

pub struct TransactionRetrier {
    replicas_addrs: Vec<String>,
    udp_sender: Box<dyn UdpSocketSender>,
}

impl TransactionRetrier {
    #[must_use]
    pub fn new(replicas_addrs: Vec<String>, udp_sender: Box<dyn UdpSocketSender>) -> Self {
        TransactionRetrier {
            replicas_addrs,
            udp_sender,
        }
    }

    pub fn process(&mut self, transaction: &dyn Transactionable) {
        let msg = transaction.retry();
        for addr in &self.replicas_addrs {
            println!(
                "[Transaction Retrier] Enviando reintento de transaccion a {}",
                addr
            );
            self.udp_sender
                .send_to(&msg, addr)
                .expect("[Transaction Retrier] Enviar reintento no deberia fallar");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        alglobo::transactionable::MockTransactionable,
        sockets::udp_socket_sender::MockUdpSocketSender,
        transaction_messages::{
            transaction_info::TransactionInfo, transaction_retry::TransactionRetry,
        },
    };

    #[test]
    fn it_should_send_message_transaction_retry_to_all_alglobo_replicas() {
        let replicas_addrs = vec![
            "127.0.0.1:49152".to_string(),
            "127.0.0.1:49353".to_string(),
            "127.0.0.1:49354".to_string(),
        ];

        let mut message = TransactionRetry::build(0, 100.0, 200.0, 300.0);
        TransactionInfo::add_padding(&mut message);

        let mut mock_sender = MockUdpSocketSender::new();
        let replicas_addrs_clone = replicas_addrs.clone();
        let message_clone = message.clone();
        mock_sender
            .expect_send_to()
            .withf(move |buf, addr| {
                buf.to_vec() == message_clone && replicas_addrs_clone.contains(&addr.to_string())
            })
            .times(3)
            .returning(|_, _| Ok(()));

        let mut retrier = TransactionRetrier::new(replicas_addrs, Box::new(mock_sender));

        let mut mock_transaction = MockTransactionable::new();

        mock_transaction
            .expect_retry()
            .times(1)
            .returning(move || message.clone());

        retrier.process(&mock_transaction);
    }
}
