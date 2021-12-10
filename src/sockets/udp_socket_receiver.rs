use super::socket_error::SocketError;
use std::time::Duration;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait UdpSocketReceiver {
    /// # Errors
    ///
    /// `SocketError::ZeroBytes` => Aún quedan bytes por recibir,
    /// pero el socket interno no recibió ningun byte en el último intento
    /// `SocketError::Timeout` => Paso demasiado tiempo sin recibir ningún byte.
    fn recv(&mut self, n: usize) -> Result<(Vec<u8>, String), SocketError>;

    fn set_timeout(&mut self, opt_timeout: Option<Duration>);
}
