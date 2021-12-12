pub mod TransactionInfo {
    pub fn size() -> usize {
        39
    }
}

#[cfg(test)]
mod tests {
    use crate::transactions::{transaction_log::TransactionLog, transaction_response::TransactionResponse};

    use super::*;
    use std::cmp::max;
    #[test]
    fn size_should_return_the_max_size_of_all_transaction_info_messages() {
        let size = max(TransactionLog::size(), TransactionResponse::size());
        assert_eq!(TransactionInfo::size(), size);
    }
}