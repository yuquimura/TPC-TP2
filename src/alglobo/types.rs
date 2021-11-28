use std::sync::{Arc, Condvar, Mutex};

use crate::alglobo::transactionable::Transactionable;

use super::transaction::Transaction;

pub type CurrentTransaction = Arc<(Mutex<Option<Box<dyn Transactionable>>>, Condvar)>;
pub type CurrentTransactionDeprecated = Arc<(Mutex<Option<Transaction>>, Condvar)>;
