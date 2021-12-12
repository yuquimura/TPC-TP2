// #[allow(dead_code)]
// pub struct ServiceName;

// #[allow(dead_code)]
// impl ServiceName {
//     #[must_use]
//     pub fn airline() -> String {
//         "Airline".to_string()
//     }

//     #[must_use]
//     pub fn bank() -> String {
//         "Bank".to_string()
//     }

//     #[must_use]
//     pub fn hotel() -> String {
//         "Hotel".to_string()
//     }
// }

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum ServiceName {
    Airline,
    Hotel,
    Bank
}

#[allow(dead_code)]
impl ServiceName {
    pub fn byte_code(&self) -> u8 {
        match *self {
            Self::Airline => b'A',
            Self::Hotel => b'H',
            Self::Bank => b'B',
        }
    }

    pub fn string_name(&self) -> String {
        match *self {
            Self::Airline => "Airline".to_string(),
            Self::Hotel => "Hotel".to_string(),
            Self::Bank => "Bank".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::services::service_name::ServiceName;

    #[test]
    fn byte_code_should_return_byte_for_each_name() {
        assert_eq!(ServiceName::Airline.byte_code(), b'A');
        assert_eq!(ServiceName::Hotel.byte_code(), b'H');
        assert_eq!(ServiceName::Bank.byte_code(), b'B');
    }

    #[test]
    fn string_name_should_return_string_for_each_name() {
        assert_eq!(ServiceName::Airline.string_name(), "Airline".to_string());
        assert_eq!(ServiceName::Hotel.string_name(), "Hotel".to_string());
        assert_eq!(ServiceName::Bank.string_name(), "Bank".to_string());
    }
}