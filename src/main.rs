use tp::sockets::udp_socket_wrap::UdpSocketWrap;
use tp::candidates::constants::{VEC_PORT_INFO,EMPTY};
use tp::candidates::candidate::Candidate;
use input_reader::get_input;

fn main() {
    let input = get_input();
    if input.is_err() {
        print!("Invalid input");
        return;
    }
    let mut socket_data_recv = UdpSocketWrap::new(None);
    let mut socket_data_send = UdpSocketWrap::new(None);
    let mut port_candidate :i32 = 0;
    let mut vec_addr: Vec<String> = vec!["49353".to_string()];
    for port in VEC_PORT_INFO.clone() {
        vec_addr.push(port.to_string().clone());
    }
    for port in VEC_PORT_INFO.clone() {
        let socket_info_data_new = UdpSocketWrap::new_with_addr(None, port.to_string());
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
    println!("Voy a inicializar al candidato");
    candidate.start_candidate();
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