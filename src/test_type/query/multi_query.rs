use crate::benchmark::BenchmarkCommands;
use crate::database::DatabaseInterface;
use crate::error::VerifierResult;
use crate::request::{get_response_body, get_response_headers, ContentType};
use crate::test_type::query::Query;
use crate::test_type::Executor;
use crate::verification::Messages;

pub struct MultiQuery {
    pub concurrency_levels: Vec<i64>,
    pub pipeline_concurrency_levels: Vec<i64>,
    pub database_verifier: Box<dyn DatabaseInterface>,
}
impl Query for MultiQuery {}
impl Executor for MultiQuery {
    fn retrieve_benchmark_commands(&self, _url: &str) -> VerifierResult<BenchmarkCommands> {
        // todo

        Ok(BenchmarkCommands {
            primer_command: "".to_string(),
            warmup_command: "".to_string(),
            benchmark_commands: vec![],
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

        let response_headers = get_response_headers(&url)?;
        messages.headers(&response_headers);
        self.verify_headers(&response_headers, &url, ContentType::Json, &mut messages);

        let test_cases = ["2", "0", "foo", "501", ""];
        let min = 1;
        let max = 500;

        for test_case in test_cases.iter() {
            let expected_length = self.translate_query_count(*test_case, min, max);
            let count_url = format!("{}{}", url, test_case);

            let response_body = get_response_body(&count_url, &mut messages);
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
                    &mut messages,
                );
            }
        }

        Ok(messages)
    }
}
