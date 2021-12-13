use std::ops::Range;

pub static EMPTY: &str = "EMPTY";
pub static DEFAULT_IP: &str = "127.0.0.1:10101";
pub static VEC_PORT_DATA: Range<i32> = (49152..49352);// addr de los servicios
pub static VEC_PORT_INFO: Range<i32> = (49353..49552); // addr de las replicas
pub static AIRLINE_ADDR: &str= "127.0.0.1:49353";
pub static HOTEL_ADDR: &str= "127.0.0.1:49354";
pub static BANK_ADDR: &str= "127.0.0.1:49355";
