pub trait CommonService {
    fn answer_message(&mut self, _vector: Vec<u8>);
    fn rollback_message(&mut self);
    fn start_client(&mut self, ip:&str) -> i64;
}
