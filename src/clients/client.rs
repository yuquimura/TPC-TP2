pub trait Client {
    fn send(&mut self, message: &str) -> Result<(), String>;
}