use crate::clients::common_client::CommonClient;
use crate::connections::message_receiver::MessageReceiver;

pub struct BankClient{
    #[allow(dead_code)]
    message_receiver: MessageReceiver,
}

impl CommonClient for BankClient{
    fn answer_message() {
        todo!()
    }
    fn rollback_message() {
        todo!()
    }
    fn start_client(&mut self) {
    }

}
