use super::socket_error::SocketError;

pub const UDP_PACKET_SIZE: usize = 512;

pub trait UdpSocketTrait {
    /// # Errors
    ///
    /// `SocketError::ZeroBytes` => Aún quedan bytes por enviar,
    /// pero el socket interno no envió ningun byte en el último intento
    fn send_to(&mut self, buf: &Vec<u8>, addr: &str) -> Result<(), SocketError>;

    /// # Errors
    ///
    /// `SocketError::ZeroBytes` => Aún quedan bytes por recibir,
    /// pero el socket interno no recibió ningun byte en el último intento
    /// `SocketError::Timeout` => Paso demasiado tiempo sin recibir ningún byte.
    fn recv(&mut self, n: usize) -> Result<Vec<u8>, SocketError>;
}
