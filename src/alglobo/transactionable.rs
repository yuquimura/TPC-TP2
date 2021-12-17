use std::collections::HashMap;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait Transactionable {
    fn get_id(&self) -> u64;

    fn set_id(&mut self, id: u64) -> bool;

    fn wait(&mut self, name: String, opt_fee: Option<f64>) -> bool;

    fn accept(&mut self, name: String, opt_fee: Option<f64>) -> bool;

    fn abort(&mut self, name: String, opt_fee: Option<f64>) -> bool;

    fn commit(&mut self, name: String, opt_fee: Option<f64>) -> bool;

    fn waiting_services(&self) -> HashMap<String, f64>;

    fn not_aborted_services(&self) -> HashMap<String, f64>;

    fn accepted_services(&self) -> HashMap<String, f64>;

    fn all_services(&self) -> HashMap<String, f64>;

    fn is_any_waiting(&self) -> bool;

    fn is_accepted(&self) -> bool;

    fn is_aborted(&self) -> bool;

    fn is_commited(&self) -> bool;

    fn log(&self) -> Vec<u8>;

    fn representation(&self) -> String;
}
