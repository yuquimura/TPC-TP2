use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ElectionCode {
    Alive,
    Election,
    Leader,
    First,
}

impl fmt::Display for ElectionCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ElectionCode::Alive => write!(f, "VIVO"),
            ElectionCode::Election => write!(f, "ELECCION"),
            ElectionCode::Leader => write!(f, "LIDER"),
            ElectionCode::First => write!(f, "PRIMERA"),
        }
    }
}
