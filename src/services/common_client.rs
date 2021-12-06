pub trait CommonClient {
    fn answer_message(&mut self, _vector: Vec<u8>) {}
    fn start_client(&mut self)->i64;
}
