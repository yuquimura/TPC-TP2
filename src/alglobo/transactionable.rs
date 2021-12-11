use std::collections::HashMap;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Transactionable {
    fn get_id(&self) -> u64;

    fn accept(&mut self, name: String) -> bool;

    fn abort(&mut self, name: String) -> bool;

    fn commit(&mut self, name: String) -> bool;

    fn waiting_services(&self) -> HashMap<String, f64>;

    fn all_services(&self) -> HashMap<String, f64>;

    fn is_waiting(&self) -> bool;

    fn is_accepted(&self) -> bool;

    fn is_aborted(&self) -> bool;

    fn is_commited(&self) -> bool;
}
