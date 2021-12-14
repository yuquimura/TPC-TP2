pub trait CommonClient {
    fn answer_message(&mut self, _vector: Vec<u8>, _string: String) {}
    fn start_client(&mut self) {}
    fn process_one_transaction(&mut self) -> Result<i64, String>;
    fn get_fee_sum(&mut self) -> f64;
}
