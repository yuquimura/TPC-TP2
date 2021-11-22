#[allow(dead_code)]
pub struct Client {
    addr: String
}

impl Client {
    pub fn new(addr: &str) -> Self {
        Client { 
            addr: addr.to_string()
        }
    }

    pub fn get_addr(&self) -> String {
        self.addr.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_have_an_address() {
        let addr = "127.0.0.1:49153";
        let client = Client::new(addr);

        assert_eq!(client.get_addr(), addr);
    }
}