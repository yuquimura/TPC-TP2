use std::cmp::max;

pub struct TransactionInfo;

impl TransactionInfo {
    pub fn size() -> usize {
        39
    }

    pub fn add_padding(message: &mut Vec<u8>) {
        let padding_len = max(0, TransactionInfo::size() - message.len());
        for _ in 0..padding_len {
            message.push(0);
        }
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

    #[test]
    fn add_padding_should_expand_vec_to_reach_transaction_info_size() {
        let mut message: Vec<u8> = Vec::new();
        assert_eq!(message.len(), 0);

        TransactionInfo::add_padding(&mut message);
        assert_eq!(message.len(), TransactionInfo::size());
    }
}