use crate::connections::message_receiver::MessageReceiver;
use crate::services::common_client::CommonClient;

pub struct AirlineClient {
    #[allow(dead_code)]
    message_receiver: MessageReceiver,
}

impl CommonClient for AirlineClient {
    fn answer_message() {
        todo!()
    }
    fn rollback_message() {
        todo!()
    }
    fn start_client(&mut self) {}
}
