use crate::error::VerifierResult;
use std::str::FromStr;
use strum_macros::EnumString;

#[derive(EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Mode {
    Database,
    Verify,
    Benchmark,
    Unknown(String),
}
impl Mode {
    /// Helper function for getting a `TestType` from `test_type_name`.
    pub fn get(test_type_name: &str) -> VerifierResult<Mode> {
        if let Ok(test_type) = Mode::from_str(&test_type_name.to_lowercase()) {
            Ok(test_type)
        } else {
            Ok(Mode::Unknown(test_type_name.to_string()))
        }
    }
}
