use crate::clients::common_client::CommonCLient;
use crate::connections::message_receiver::MessageReceiver;

pub struct HotelClient{
    message_receiver: MessageReceiver,
}

impl CommonCLient for HotelClient{
    fn answer_message() {
        todo!()
    }
    fn rollback_message() {
        todo!()
    }
    fn start_client(&mut self) {
    }

}