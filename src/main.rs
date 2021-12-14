use std::time::Duration;
use tp::sockets::udp_socket_wrap::UdpSocketWrap;
use tp::candidates::constants::{VEC_PORT_INFO, EMPTY, HOTEL_ADDR, BANK_ADDR, AIRLINE_ADDR, DEFAULT_IP};
use tp::candidates::candidate::Candidate;
use input_reader::get_input;
use tp::services::airline_client::Airline;
use tp::services::bank_client::Bank;
use tp::services::common_client::CommonClient;
use tp::services::hotel_client::Hotel;

fn main() {
    let input = get_input();
    if input.is_err() {
        print!("Invalid input");
        return;
    }
    if input.clone().unwrap() == "c"{
        let mut socket_data_recv = UdpSocketWrap::new(None);
        let mut socket_data_send = UdpSocketWrap::new(None);
        let mut port_candidate :i32 = 0;
        let mut vec_addr: Vec<String> = vec!["49353".to_string()];
        for port in VEC_PORT_INFO.clone() {
            vec_addr.push(port.to_string().clone());

        }
        for port in VEC_PORT_INFO.clone() {
            let socket_info_data_new = UdpSocketWrap::new_with_addr(None, DEFAULT_IP.to_string()+ port.to_string().as_str());
            if let Ok(socket_new_aux) = socket_info_data_new {
                socket_data_recv = socket_new_aux;
                if let Ok(socket_aux) = socket_data_recv.try_clone() {
                    socket_data_send = socket_aux;
                    port_candidate = port;
                    break;
                }
            }
        }

        let mut candidate = Candidate::new(Box::new(socket_data_recv), Box::new(socket_data_send), port_candidate.to_string(), vec_addr, EMPTY.to_string(), "".to_string());
        candidate.start_candidate();
    }
    else if  input.as_ref().clone().unwrap() == "a" {

        let socket_send_airline = UdpSocketWrap::new_with_addr(Some(Duration::from_millis(100000)), AIRLINE_ADDR.to_string()).expect("No pude crear el socket del servicio de la aerolinea");
        let socket_recv_airline = socket_send_airline.try_clone().expect("No pude copiar el socket del servicio de la aerolinea");
        let mut airline_service = Airline::new(Box::new(socket_send_airline), Box::new(socket_recv_airline), AIRLINE_ADDR.to_string());
        airline_service.start_client();
    }
    else if  input.as_ref().clone().unwrap() == "b" {
        let socket_send_bank = UdpSocketWrap::new_with_addr(Some(Duration::from_millis(100000)), BANK_ADDR.to_string()).expect("No pude crear el socket del servicio de la aerolinea");
        let socket_recv_bank = socket_send_bank.try_clone().expect("No pude copiar el socket del servicio de la aerolinea");
        let mut bank_service = Bank::new(Box::new(socket_send_bank), Box::new(socket_recv_bank), BANK_ADDR.to_string());
        bank_service.start_client();

    }
    else if input.as_ref().clone().unwrap() == "h"  {
        let socket_send_hotel = UdpSocketWrap::new_with_addr(Some(Duration::from_millis(100000)), HOTEL_ADDR.to_string()).expect("No pude crear el socket del servicio del hotel");
        let socket_recv_hotel = socket_send_hotel.try_clone().expect("No pude copiar el socket del servicio del hotel");
        let mut hotel_service = Hotel::new(Box::new(socket_recv_hotel), Box::new(socket_send_hotel), HOTEL_ADDR.to_string());
        hotel_service.start_client();
    }
    else {
        println!("Error. Utilizar cargo run + c: crear candidato. a: servicio aerolinea, b: servicio banco o h: servicio hotel");
    }

}



mod input_reader {
    use std::env;

    pub fn get_input() -> Result<String, i64> {
        let args: Vec<String> = env::args().collect();
        if args.is_empty() {
            return Err(1);
        }
        let filename = &args[1];
        Ok(filename.to_string())
    }
}