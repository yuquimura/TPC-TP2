use std::ops::Range;

pub static EMPTY: &str = "EMPTY";
pub static SLEEP: u64 = 1;
pub static DEFAULT_IP: &str = "127.0.0.1:";
// addr de los servicios
pub static VEC_PORT_DATA: Range<i32> = 49152..49155;
// addr de las replicas
pub static VEC_PORT_INFO: Range<i32> = 49354..49356;
pub static AIRLINE_ADDR: &str = "127.0.0.1:59353";
pub static HOTEL_ADDR: &str = "127.0.0.1:59354";
pub static BANK_ADDR: &str = "127.0.0.1:59355";
