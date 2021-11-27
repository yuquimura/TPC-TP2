use std::fmt;

#[allow(dead_code)]
type Result<T> = std::result::Result<T, SocketError>;

#[derive(Debug, PartialEq)]
pub enum SocketError {
    CloneFailed,
    Timeout,
    ZeroBytes,
}

impl fmt::Display for SocketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SocketError::CloneFailed => write!(f, "Clonar un socket no deberia fallar"),
            SocketError::Timeout => write!(f, "Timeout"),
            SocketError::ZeroBytes => write!(f, "Cero bytes transmitidos"),
        }
    }
}
