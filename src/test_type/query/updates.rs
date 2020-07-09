use crate::database::DatabaseVerifier;
use crate::error::VerifierResult;
use crate::message::Messages;
use crate::request::{get_response_body, get_response_headers, ContentType};
use crate::test_type::query::Query;
use crate::test_type::Verifier;

pub struct Updates {
    pub concurrency_levels: Vec<i32>,
    pub database_verifier: Box<dyn DatabaseVerifier>,
}
impl Query for Updates {}
impl Verifier for Updates {
    fn verify(&self, url: &str) -> VerifierResult<Messages> {
        let mut messages = Messages::new(url);

        let test_cases = ["2", "0", "foo", "501", ""];

        // Initialization for query counting
        let repetitions = 1;
        let concurrency = *self.concurrency_levels.iter().max().unwrap();
        let expected_queries = 20 * repetitions * concurrency;
        let _expected_rows = expected_queries;
        let min = 1;
        let max = 500;

        // todo - before we make a request to the server, we need to capture
        //  the `World` 'table' from the configured database for a "before".

        let response_headers = get_response_headers(&url)?;
        messages.headers(&response_headers);
        self.verify_headers(&response_headers, &url, ContentType::Json, &mut messages);

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
                    url,
                    "world",
                    concurrency,
                    repetitions,
                    expected_queries,
                    true,
                    &mut messages,
                );
                self.verify_updates_count(
                    concurrency,
                    repetitions,
                    expected_queries,
                    &mut messages,
                );
            }
        }

        Ok(messages)
    }
}
impl Updates {
    fn verify_updates_count(
        &self,
        _concurrency: i32,
        _repetitions: i32,
        _expected_queries: i32,
        _messages: &mut Messages,
    ) {
        // todo - capture the `World` 'table' again and compare it to the
        //  "before" to ensure that a believable percent difference
        //  occurred (meaning the verification succeeded because the test
        //  impl actually did writes.
    }
}
