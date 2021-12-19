use std::{ops::Range, time::Duration};

pub static EMPTY: &str = "EMPTY";
pub static SLEEP_MANAGER: Duration = Duration::from_secs(1);
pub static DEFAULT_IP: &str = "127.0.0.1:";
// addr de los servicios
pub static VEC_PORT_DATA: Range<i32> = 49152..49155;
// addr de las replicas
pub static VEC_PORT_INFO: Range<i32> = 49354..49356;
pub static AIRLINE_ADDR: &str = "127.0.0.1:59353";
pub static HOTEL_ADDR: &str = "127.0.0.1:59354";
pub static BANK_ADDR: &str = "127.0.0.1:59355";
pub static TRANSACTION_FILE: &str = "data/data.csv";
pub static ABORT_FILE: &str = "data/abortadas.csv";
pub static END_TIMEOUT: Duration = Duration::from_secs(10);