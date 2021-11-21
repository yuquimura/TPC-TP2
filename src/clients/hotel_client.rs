use crate::clients::common_client::CommonClient;
use crate::connections::message_receiver::MessageReceiver;

pub struct HotelClient{
    #[allow(dead_code)]
    message_receiver: MessageReceiver,
}

impl CommonClient for HotelClient{
    fn answer_message() {
        todo!()
    }
    fn rollback_message() {
        todo!()
    }
    fn start_client(&mut self) {
    }

}
