use std::net::UdpSocket;

use super::{client::Client, client_error::ClientError};

const UDP_PACKET_SIZE:usize = 512;

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
    fn send(&mut self, vec_bytes: Vec::<u8>) -> Result<(), ClientError> {
        if vec_bytes.len() == 0 {
            return Ok(());
        }
        let mut total_bytes_sent= 0;
        let buf = &vec_bytes[..];
        while total_bytes_sent < buf.len() {
            let address_port = self.address_port();
            let bytes_sent = self.socket.send_to(&buf[total_bytes_sent..], address_port)
                                                .expect("[UDPclient] Wrong IP address version");
            if bytes_sent == 0 {
                return Err(ClientError::ZeroBytes);
            }
            total_bytes_sent += bytes_sent;
        }
        Ok(())
    }

    fn recv(&mut self, n_bytes: usize) -> Result<Vec::<u8>, ClientError> {
        let mut res:Vec<u8> = vec![];
        let mut buf = [0; UDP_PACKET_SIZE];
        while res.len() < n_bytes {
            let (bytes_recv, _) = self.socket.recv_from(&mut buf)
                                                                .expect("[UDP Client] Recv should not fail");
            if bytes_recv == 0 {
                return Err(ClientError::ZeroBytes);
            }
            res.append(&mut buf[..bytes_recv].to_vec());
        }
        Ok(res)
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
        let mut buf = [0; MSG_LEN];

        let res = client.send(message.as_bytes().to_vec());
        assert!(res.is_ok());

        let (_, _) = socket.recv_from(&mut buf).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), message);
    }

    #[test]
    fn it_should_recv_a_message() {
        let address = "127.0.0.1";
        let port = "49153"; // Test en paralelo => Usar un puerto distinto
        let address_port = format!("{}:{}", address, port);
        let socket = UdpSocket::bind(address_port).unwrap();
        let mut client = UDPClient::new(address, port);

        let message = "a message";
        const MSG_LEN: usize = 9; // Debe resolverse en tiempo de compilación
        assert_eq!(MSG_LEN, message.len());
        let mut buf = [0; MSG_LEN];

        client.send(message.as_bytes().to_vec()).unwrap();
        let (_, client_addr) = socket.recv_from(&mut buf).unwrap();
        socket.send_to(&buf, client_addr).unwrap();

        let res = client.recv(MSG_LEN);

        assert!(res.is_ok());
        assert_eq!(res.unwrap(), message.as_bytes());
    }
}