use std::net::UdpSocket;
use crate::clients::client::Client;

#[allow(dead_code)]
pub struct UDPClient {
    address: String, 
    port: String,
    socket: UdpSocket
}

impl UDPClient {
    #[must_use]
    pub fn new(address: &str, port: &str) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0")
                                .expect("[UDPClient] Unable to bind socket");
        UDPClient{
            address: address.to_string(),
            port: port.to_string(),
            socket
        }
    }

    fn address_port(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }
}

impl Client for UDPClient {
    fn send(&mut self, message: &str) -> Result<(), String> {
        if message.len() == 0 {
            return Ok(());
        }
        let buf = message.as_bytes();
        let address_port = self.address_port();

        let mut bytes_sent= 0;
        let msg_bytes_len = message.as_bytes().len();
        
        while bytes_sent < msg_bytes_len {
            let last_bytes_sent = self.socket.send_to(&buf[bytes_sent..], address_port.clone())
                                                .expect("[UDPclient] Wrong IP address version");
            if last_bytes_sent == 0 {
                return Err("[UDPClient] Zero bytes sent".to_string());
            }
            bytes_sent += last_bytes_sent;
        }
        println!("bytes sent: {}", bytes_sent);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_have_an_address_and_a_port() {
        let address = "127.0.0.1";
        let port = "49152";
        let client = UDPClient::new(address, port);

        assert_eq!(client.address, address);
        assert_eq!(client.port, port);
    }

    #[test]
    fn it_should_send_a_message() {
        let address = "127.0.0.1";
        let port = "49152";
        let address_port = format!("{}:{}", address, port);
        let socket = UdpSocket::bind(address_port).unwrap();
        let mut client = UDPClient::new(address, port);

        let message = "a message";
        const MSG_LEN: usize = 9; // Necesito que se resuelva en tiempo de compilación
        assert_eq!(MSG_LEN, message.len());

        let res = client.send(message);

        assert!(res.is_ok());

        let mut buf = [0; MSG_LEN];
        let (_, _) = socket.recv_from(&mut buf).unwrap();

        assert_eq!(std::str::from_utf8(&buf).unwrap(), message);
    }
}