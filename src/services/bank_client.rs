use crate::services::common_client::CommonClient;

pub struct BankClient {
}

impl CommonClient for BankClient {
    fn start_client(&mut self, ip: &str) -> i64 {
        todo!()
    }
}
