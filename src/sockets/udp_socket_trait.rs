use super::socket_error::SocketError;

const UDP_PACKET_SIZE:usize = 512;

pub trait UdpSocketTrait {
    fn send_to(&mut self, buf: &[u8], addr: &str) -> Result<(), SocketError>;
    
    fn recv(&mut self, n: usize) -> Result<[u8; UDP_PACKET_SIZE], SocketError>;
}