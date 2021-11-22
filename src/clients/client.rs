pub trait Client {
    fn send(&mut self, vec_bytes: Vec::<u8>) -> Result<(), String>;
    
    fn recv(&mut self, n: usize) -> Result<Vec::<u8>, String>;
}