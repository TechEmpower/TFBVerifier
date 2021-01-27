use crate::benchmark::BenchmarkCommands;
use crate::database::DatabaseInterface;
use crate::error::VerifierResult;
use crate::request::{get_response_body, get_response_headers, ContentType};
use crate::test_type::query::Query;
use crate::test_type::Executor;
use crate::verification::Messages;
use std::cmp::min;

pub struct MultiQuery {
    pub concurrency_levels: Vec<u32>,
    pub database_verifier: Box<dyn DatabaseInterface>,
}
impl Query for MultiQuery {}
impl Executor for MultiQuery {
    fn wait_for_database_to_be_available(&self) {
        self.database_verifier.wait_for_database_to_be_available();
    }

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

    /// Validates the response is a JSON array of the proper length, each JSON
    /// Object in the array has keys 'id' and 'randomNumber', and these keys
    /// map to integer-ish types.
    /// The `MultiQuery` tests accept a `queries` parameter that is expected to
    /// be between 1-500.
    ///
    /// The reason for using 'warn' is generally for a case that will be
    /// allowed in the current run but that may/will be a failing case in
    /// future rounds. The cases above suggest that not sanitizing the
    /// `queries` parameter against non-int input, or failing to ensure the
    /// parameter is between 1-500 will just be a warn, and not prevent the
    /// framework from being benchmarked.
    fn verify(&self, url: &str) -> VerifierResult<Messages> {
        let mut messages = Messages::new(url);

        // Initialization for query counting
        let repetitions = 2;
        let concurrency = *self.concurrency_levels.iter().max().unwrap();
        let expected_queries = 20 * repetitions * concurrency;
        let expected_rows = expected_queries;

        // Because this test type is going to make a LOT of requests with a reasonably long timeout,
        // we use `get_response_headers` as a sentinel. If a `CurlError` is thrown, then we do not
        // perform any of the follow-up requests to conserve time.
        if let Ok(response_headers) = get_response_headers(&url, &mut messages) {
            messages.headers(&response_headers);
            self.verify_headers(&response_headers, &url, ContentType::Json, &mut messages);

            let test_cases = ["2", "0", "foo", "501", ""];
            let min = 1;
            let max = 500;

            for test_case in test_cases.iter() {
                let expected_length = self.translate_query_count(*test_case, min, max);
                let count_url = format!("{}{}", url, test_case);

                if let Some(response_body) = get_response_body(&count_url, &mut messages) {
                    messages.body(&response_body);
                    self.verify_with_length(&response_body, expected_length, &mut messages);

                    // Only check update changes if we're testing the highest number of
                    // queries, to ensure that we don't accidentally FAIL for a query
                    // that only updates 1 item and happens to set its randomNumber to
                    // the same value it previously held
                    if expected_length == max {
                        self.database_verifier.verify_queries_count(
                            &format!("{}20", url),
                            "world",
                            concurrency,
                            repetitions,
                            expected_queries,
                            &mut messages,
                        );
                        self.database_verifier.verify_rows_count(
                            &format!("{}20", url),
                            "world",
                            concurrency,
                            repetitions,
                            expected_rows,
                            1,
                            &mut messages,
                        );
                    }
                }
            }
        }

        Ok(messages)
    }
}
impl MultiQuery {
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
