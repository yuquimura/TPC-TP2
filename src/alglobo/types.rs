use std::sync::{Arc, Condvar, Mutex};

use crate::alglobo::transactionable::Transactionable;

pub type CurrentTransaction = Arc<(Mutex<Option<Box<dyn Transactionable + Send>>>, Condvar)>;
