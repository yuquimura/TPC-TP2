use super::client_error::ClientError;

pub trait Client {
    fn send(&mut self, vec_bytes: Vec::<u8>) -> Result<(), ClientError>;
    
    fn recv(&mut self, n: usize) -> Result<Vec::<u8>, ClientError>;
}