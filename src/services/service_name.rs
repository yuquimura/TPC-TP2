#[allow(dead_code)]
pub struct ServiceName;

#[allow(dead_code)]
impl ServiceName {
    #[must_use]
    pub fn airline() -> String {
        "Airline".to_string()
    }

    #[must_use]
    pub fn bank() -> String {
        "Bank".to_string()
    }
    
    #[must_use]
    pub fn hotel() -> String {
        "Hotel".to_string()
    }
}
