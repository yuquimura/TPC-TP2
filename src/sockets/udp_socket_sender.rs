use super::socket_error::SocketError;

#[cfg(test)]
use mockall::automock;

pub const UDP_PACKET_SIZE: usize = 512;

#[cfg_attr(test, automock)]
pub trait UdpSocketSender {
    /// # Errors
    ///
    /// `SocketError::ZeroBytes` => Aún quedan bytes por enviar,
    /// pero el socket interno no envió ningun byte en el último intento
    fn send_to(&mut self, buf: &[u8], addr: &str) -> Result<(), SocketError>;
}
