mod benchmark;
mod database;
mod error;
mod logger;
mod mode;
mod request;
mod test_type;
mod verification;

extern crate html5ever;
extern crate strum;
extern crate threadpool;

use crate::benchmark::send_benchmark_commands;
use crate::error::VerifierResult;
use crate::logger::{log, LogOptions};
use crate::mode::Mode;
use crate::test_type::TestType;
use colored::Colorize;
use std::env;
use std::str::FromStr;

fn main() -> VerifierResult<()> {
    let mode_name = env::var("MODE")?;
    let port = env::var("PORT")?.parse::<u32>()?;
    let endpoint = env::var("ENDPOINT")?;
    let test_type_name = env::var("TEST_TYPE")?;
    let concurrency_levels = env::var("CONCURRENCY_LEVELS")?;
    let pipeline_concurrency_levels = env::var("PIPELINE_CONCURRENCY_LEVELS")?;
    let database = match env::var("DATABASE") {
        Ok(database) => Some(database),
        _ => None,
    };

    let test_type = TestType::get(&test_type_name)?;
    let url = format!("http://{}:{}{}", "tfb-server", port, endpoint);

    let executor = test_type.get_executor(
        database,
        concurrency_levels
            .split(',')
            .map(|item| u32::from_str(item).unwrap())
            .collect(),
        pipeline_concurrency_levels
            .split(',')
            .map(|item| u32::from_str(item).unwrap())
            .collect(),
    )?;

    match Mode::get(&mode_name)? {
        Mode::Benchmark => {
            let benchmark = executor.retrieve_benchmark_commands(&url)?;
            send_benchmark_commands(benchmark);
        }
        Mode::Verify => {
            log(
                format!("VERIFYING {}", test_type_name).bright_white(),
                LogOptions {
                    border: Some('-'),
                    border_bottom: None,
                    quiet: false,
                },
            );

            let messages = executor.verify(&url)?;
            messages.output_verification_results();
        }
        Mode::Unknown(_mode) => {
            // todo - should probably output *something*
        }
    };

    Ok(())
}
