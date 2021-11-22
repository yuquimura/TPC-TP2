use std::fmt;

#[allow(dead_code)]
type Result<T> = std::result::Result<T, ClientError>;

#[derive(Debug, PartialEq)]
pub enum ClientError {
    ZeroBytes,
    Timeout 
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ClientError::ZeroBytes =>
                write!(f, "Cero bytes transmitidos"),
            ClientError::Timeout =>
                write!(f, "Timeout"),
        }
    }
}