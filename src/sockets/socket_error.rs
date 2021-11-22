use std::fmt;

#[allow(dead_code)]
type Result<T> = std::result::Result<T, SocketError>;

#[derive(Debug, PartialEq)]
pub enum SocketError {
    ZeroBytes,
    Timeout,
}

impl fmt::Display for SocketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SocketError::ZeroBytes => write!(f, "Cero bytes transmitidos"),
            SocketError::Timeout => write!(f, "Timeout"),
        }
    }
}
