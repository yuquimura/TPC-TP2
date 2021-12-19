use std::{collections::HashMap, env};

use tp::{
    alglobo::{transaction::Transaction, transaction_error::TransactionError},
    candidates::constants::{DEFAULT_IP, VEC_PORT_DATA},
    retry::transaction_retrier::TransactionRetrier,
    services::service_name::ServiceName,
    sockets::udp_socket_wrap::UdpSocketWrap,
};

static ERR_MSG: &str = "
    Uso:
        cargo run --bin retry -- <id:u64> <pago_aerolinea:f64> <pago_hotel:f64> <pago_banco:f64>
    ";

fn parse_args() -> Result<Transaction, TransactionError> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        println!("{}", ERR_MSG);
        return Err(TransactionError::None);
    }
    let id = if let Ok(value) = args[1].parse::<u64>() {
        value
    } else {
        println!("{}", ERR_MSG);
        return Err(TransactionError::None);
    };
    let services_names = [
        (ServiceName::Airline.string_name()),
        (ServiceName::Hotel.string_name()),
        (ServiceName::Bank.string_name()),
    ];
    let mut services_info = HashMap::new();
    for (idx, name) in services_names.iter().enumerate() {
        let fee = if let Ok(value) = args[2 + idx].parse::<f64>() {
            value
        } else {
            println!("{}", ERR_MSG);
            return Err(TransactionError::None);
        };
        services_info.insert(name.clone(), fee);
    }
    Ok(Transaction::new(id, &services_info))
}

fn main() {
    let transaction = match parse_args() {
        Ok(value) => value,
        Err(_) => return,
    };

    let udp_sender = UdpSocketWrap::new(None);
    let mut resplicas_addrs: Vec<String> = Vec::new();
    for port in VEC_PORT_DATA.clone() {
        resplicas_addrs.push(DEFAULT_IP.to_string() + port.to_string().as_str());
    }
    let mut retrier = TransactionRetrier::new(resplicas_addrs, Box::new(udp_sender));

    retrier.process(&transaction);
}
