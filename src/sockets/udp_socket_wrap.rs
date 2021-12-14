use crate::sockets::constants::UDP_PACKET_SIZE;
use std::{net::UdpSocket, time::Duration};

use super::{
    socket_error::SocketError, udp_socket_receiver::UdpSocketReceiver,
    udp_socket_sender::UdpSocketSender,
};

#[allow(dead_code)]
pub struct UdpSocketWrap {
    socket: UdpSocket,
}

impl UdpSocketWrap {
    #[must_use]
    pub fn new(opt_timeout: Option<Duration>) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("[UdpSocketWrap] Bind ha fallado");
        socket
            .set_read_timeout(opt_timeout)
            .expect("[UdpSocketWrap] Set timeout ha fallado");
        UdpSocketWrap { socket }
    }

    /// # Errors
    ///
    /// Devuelve error si no fue posible inicializar un socket con esa addr
    pub fn new_with_addr(
        opt_timeout: Option<Duration>,
        addr: String,
    ) -> Result<UdpSocketWrap, String> {
        let socket_result = UdpSocket::bind(addr);
        if let Ok(socket_result) = socket_result {
            let socket = socket_result;
            socket
                .set_read_timeout(opt_timeout)
                .expect("[UdpSocketWrap] Set timeout ha fallado");
            let udp_socket_wrap = UdpSocketWrap { socket };
            Ok(udp_socket_wrap)
        } else {
            Err("Todo mal".to_string())
        }
    }
    /// # Errors
    ///
    /// `SocketError::CloneFailed` => Fallo en el intento de clonación
    pub fn try_clone(&self) -> Result<Self, SocketError> {
        let socket_clone = match self.socket.try_clone() {
            Ok(clone) => clone,
            Err(_) => return Err(SocketError::CloneFailed),
        };
        Ok(UdpSocketWrap {
            socket: socket_clone,
        })
    }
}

impl UdpSocketSender for UdpSocketWrap {
    fn send_to(&mut self, bytes_vec: &[u8], addr: &str) -> Result<(), SocketError> {
        let mut buf = [0; UDP_PACKET_SIZE];
        for (i, x) in bytes_vec.iter().enumerate() {
            if i >= buf.len() {
                break;
            }
            buf[i] = *x;
        }

        let mut total_bytes_sent = 0;
        while total_bytes_sent < bytes_vec.len() {
            let bytes_sent = self
                .socket
                .send_to(&buf[total_bytes_sent..], addr)
                .expect("[UdpSocketWrap] Version de direccion IP incorrecta");
            if bytes_sent == 0 {
                return Err(SocketError::ZeroBytes);
            }
            total_bytes_sent += bytes_sent;
        }
        Ok(())
    }
}
impl UdpSocketReceiver for UdpSocketWrap {
    fn recv(&mut self, n_bytes: usize) -> Result<(Vec<u8>, String), SocketError> {
        let mut buf = [0; UDP_PACKET_SIZE];
        let mut from_addr = "".to_string();
        let mut total_bytes_recv = 0;
        while total_bytes_recv < n_bytes {
            let res = self.socket.recv_from(&mut buf[total_bytes_recv..]);
            let (bytes_recv, addr) = match res {
                Ok(value) => value,
                Err(_) => return Err(SocketError::Timeout),
            };

            if bytes_recv == 0 {
                return Err(SocketError::ZeroBytes);
            }
            total_bytes_recv += bytes_recv;
            from_addr = addr.to_string();
        }
        let res = buf[..n_bytes].to_vec();
        Ok((res, from_addr))
    }

    fn set_timeout(&mut self, opt_timeout: Option<Duration>) {
        self.socket
            .set_read_timeout(opt_timeout)
            .expect("[UdpSocketWrap] Set timeout ha fallado ");
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use ntest::timeout;

    #[test]
    fn it_should_send_a_message() {
        let addr = "127.0.0.1:59153";
        let socket = UdpSocket::bind(addr).unwrap();
        let mut client = UdpSocketWrap::new(None);

        let message = "a message".as_bytes().to_vec();
        const MSG_LEN: usize = 9; // Necesito que se resuelva en tiempo de compilación
        assert_eq!(MSG_LEN, message.len());
        let mut buf = [0; MSG_LEN];

        let res = client.send_to(&message, addr);
        assert!(res.is_ok());

        let (_, _) = socket.recv_from(&mut buf).unwrap();
        assert_eq!(buf[..message.len()].to_vec(), message);
    }

    #[test]
    fn it_should_recv_a_message() {
        let addr = "127.0.0.1:59154"; // Test en paralelo => Usar un puerto distinto
        let socket = UdpSocket::bind(addr).unwrap();
        let mut client = UdpSocketWrap::new(None);

        let message = "a message".as_bytes().to_vec();
        let mut buf = [0; UDP_PACKET_SIZE];

        client.send_to(&message, addr).unwrap();
        let (_, client_addr) = socket.recv_from(&mut buf).unwrap();
        socket.send_to(&buf[..message.len()], client_addr).unwrap();

        let res = client.recv(message.len());

        assert!(res.is_ok());
        let res_vec = res.unwrap().0;
        assert_eq!(res_vec, message);
    }

    #[test]
    #[timeout(5000)]
    fn it_should_return_timeout_error_on_recv_timeout() {
        let some_timeout = Some(Duration::from_millis(1));
        let mut client = UdpSocketWrap::new(some_timeout);
        let res = client.recv(1);
        match res {
            Ok(_) => assert!(false),
            Err(err) => assert_eq!(err, SocketError::Timeout),
        };
    }

    #[test]
    #[timeout(5000)]
    fn it_should_be_send_from_the_original_and_receive_from_the_cloned() {
        let addr = "127.0.0.1:59155"; // Test en paralelo => Usar un puerto distinto
        let socket = UdpSocket::bind(addr).unwrap();
        let original = UdpSocketWrap::new(None);
        let clone = original.try_clone().unwrap();

        let mut original_box: Box<dyn UdpSocketSender> = Box::new(original);
        let mut clone_box: Box<dyn UdpSocketReceiver> = Box::new(clone);

        let message = "a message".as_bytes().to_vec();
        let mut buf = [0; UDP_PACKET_SIZE];

        let send_res = original_box.send_to(&message, addr);
        assert!(send_res.is_ok());

        let (_, sender_addr) = socket.recv_from(&mut buf).unwrap();
        socket.send_to(&buf[..message.len()], sender_addr).unwrap();

        let recv_res = clone_box.recv(message.len());
        assert!(recv_res.is_ok());

        let result_message = recv_res.unwrap().0;
        assert_eq!(result_message, message);
    }
}
