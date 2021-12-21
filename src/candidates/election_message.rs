use super::election_code::ElectionCode;

const ALIVE_BYTE: u8 = b'v';
const ELECTION_BYTE: u8 = b'e';
const LEADER_BYTE: u8 = b'l';
const FIRST_BYTE: u8 = b'f';

pub struct ElectionMessage;

impl ElectionMessage {
    #[must_use]
    pub fn size() -> usize {
        ElectionMessage::build(ElectionCode::Alive).len()
    }

    #[must_use]
    pub fn build(code: ElectionCode) -> Vec<u8> {
        let code = ElectionMessage::map_code(code);
        let message = vec![code];
        message
    }

    /// # Panics
    ///
    /// Esta funcion paniquea si:
    /// - se recibio un codigo desconocido
    #[must_use]
    pub fn code(code: u8) -> ElectionCode {
        let err_msg = format!(
            "[Election message] Codigo de eleccion desconocido: {}",
            code
        );
        match code {
            ALIVE_BYTE => ElectionCode::Alive,
            ELECTION_BYTE => ElectionCode::Election,
            LEADER_BYTE => ElectionCode::Leader,
            _ => panic!("{}", err_msg),
        }
    }

    fn map_code(code: ElectionCode) -> u8 {
        match code {
            ElectionCode::Alive => ALIVE_BYTE,
            ElectionCode::Election => ELECTION_BYTE,
            ElectionCode::Leader => LEADER_BYTE,
            ElectionCode::First => FIRST_BYTE,
        }
    }
}
