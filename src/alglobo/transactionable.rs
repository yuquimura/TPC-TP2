#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Transactionable {
    fn get_id(&self) -> u64;

    fn accept(&mut self, name: String) -> bool;

    fn abort(&mut self, name: String) -> bool;
}