#[allow(dead_code)]
pub struct UDPClient {
    address: String, 
    port: String
}

impl UDPClient {
    #[must_use]
    pub fn new(address: &str, port: &str) -> Self {
        UDPClient{
            address: address.to_string(),
            port: port.to_string()
        }
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
}