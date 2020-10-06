use crate::benchmark::BenchmarkCommands;
use crate::database::DatabaseInterface;
use crate::error::VerifierResult;
use crate::request::{get_response_body, get_response_headers, ContentType};
use crate::test_type::query::Query;
use crate::test_type::Executor;
use crate::verification::Messages;
use serde_json::Value;
use std::cmp::min;

pub struct SingleQuery {
    pub concurrency_levels: Vec<u32>,
    pub database_verifier: Box<dyn DatabaseInterface>,
}
impl Query for SingleQuery {}
impl Executor for SingleQuery {
    fn retrieve_benchmark_commands(&self, url: &str) -> VerifierResult<BenchmarkCommands> {
        let primer_command = self.get_wrk_command(url, 5, 8);
        let warmup_command =
            self.get_wrk_command(url, 15, *self.concurrency_levels.iter().max().unwrap());
        let mut benchmark_commands = Vec::default();
        for concurrency in &self.concurrency_levels {
            benchmark_commands.push(self.get_wrk_command(url, 15, *concurrency));
        }

        Ok(BenchmarkCommands {
            primer_command,
            warmup_command,
            benchmark_commands,
        })
    }

    fn verify(&self, url: &str) -> VerifierResult<Messages> {
        let mut messages = Messages::new(url);

        let response_headers = get_response_headers(&url)?;
        messages.headers(&response_headers);
        self.verify_headers(&response_headers, &url, ContentType::Json, &mut messages);
        let response_body = get_response_body(&url, &mut messages);
        messages.body(&response_body);

        // Initialization for query counting
        let repetitions = 2;
        let concurrency = *self.concurrency_levels.iter().max().unwrap();
        let expected_queries = repetitions * concurrency;
        let expected_rows = expected_queries;

        self.verify_single_query(&response_body, &mut messages);
        self.database_verifier.verify_queries_count(
            url,
            "world",
            concurrency,
            repetitions,
            expected_queries,
            &mut messages,
        );
        self.database_verifier.verify_rows_count(
            url,
            "world",
            concurrency,
            repetitions,
            expected_rows,
            1,
            &mut messages,
        );

        Ok(messages)
    }
}
impl SingleQuery {
    fn verify_single_query(&self, response_body: &str, messages: &mut Messages) {
        match serde_json::from_str::<Value>(&response_body.to_lowercase()) {
            Err(e) => {
                messages.error(format!("Invalid JSON: {:?}", e), "Invalid JSON");
            }
            Ok(mut json) => {
                if let Some(arr) = json.as_array() {
                    messages.warning(
                        "Response is a JSON array. Expected JSON object (e.g. [] vs {})",
                        "Expected JSON object",
                    );
                    if let Some(first) = arr.get(0) {
                        json = first.clone();
                    }
                }
                if let Some(json) = json.as_object() {
                    self.verify_random_number_object(json, messages);
                } else {
                    messages.error(
                        "Response is not a JSON object or an array of JSON objects",
                        "Invalid JSON",
                    );
                }
            }
        }
    }

    fn get_wrk_command(&self, url: &str, duration: u32, concurrency: u32) -> Vec<String> {
        vec![
            "wrk",
            "-H",
            "Host: tfb-server",
            "-H",
            "Accept: application/json,text/html;q=0.9,application/xhtml+xml;q=0.9,application/xml;q=0.8,*/*;q=0.7",
            "-H",
            "Connection: keep-alive",
            "--latency",
            "-d",
            &format!("{}", duration),
            "-c",
            &format!("{}", concurrency),
            "--timeout",
            "8",
            "-t",
            &format!("{}", min(concurrency, num_cpus::get() as u32)),
            url,
        ].iter().map(|item| item.to_string()).collect()
    }
}

//
// TESTS
//

#[cfg(test)]
mod tests {
    use crate::database::mysql::Mysql;
    use crate::test_type::query::single_query::SingleQuery;
    use crate::verification::Messages;

    #[test]
    fn it_should_pass_simply() {
        let query = SingleQuery {
            concurrency_levels: vec![16, 32, 64, 128, 256, 512],
            database_verifier: Box::new(Mysql {}),
        };
        let mut messages = Messages::default();
        query.verify_single_query("{\"id\": 2354,\"randomNumber\":8952}", &mut messages);
        assert!(messages.errors.is_empty());
        assert!(messages.warnings.is_empty());
    }
}
