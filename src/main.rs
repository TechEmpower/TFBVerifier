mod database;
mod error;
mod logger;
mod message;
mod request;
mod test_type;

extern crate html5ever;
extern crate strum;

use crate::error::VerifierResult;
use crate::logger::{log, LogOptions};
use crate::test_type::TestType;
use colored::Colorize;
use std::env;
use std::str::FromStr;

fn main() -> VerifierResult<()> {
    let port = env::var("PORT")?.parse::<u32>()?;
    let endpoint = env::var("ENDPOINT")?;
    let test_type_name = &env::var("TEST_TYPE")?;
    let concurrency_levels = env::var("CONCURRENCY_LEVELS")?;
    let database = match env::var("DATABASE") {
        Ok(database) => Some(database),
        _ => None,
    };

    let test_type = TestType::get(&test_type_name)?;
    let url = format!("http://{}:{}{}", "tfb-server", port, endpoint);

    let verifier = test_type.get_verifier(
        database,
        concurrency_levels
            .split(',')
            .map(|item| i32::from_str(item).unwrap())
            .collect(),
    )?;

    log(
        format!("VERIFYING {}", test_type_name).bright_white(),
        LogOptions {
            border: Some('-'),
            border_bottom: None,
            quiet: false,
        },
    );

    let messages = verifier.verify(&url)?;
    messages.output_results();

    Ok(())
}
