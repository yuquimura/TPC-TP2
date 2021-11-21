#[allow(dead_code)]
struct TransactionManager {
    pub id: usize
}

#[allow(dead_code)]
impl TransactionManager {
    pub fn new(id: usize) -> Self {
        TransactionManager{
            id
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_have_an_id() {
        let id = 0;
        let manager = TransactionManager::new(id);

        assert_eq!(manager.id, id);
    }
}