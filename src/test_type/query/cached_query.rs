use crate::database::DatabaseInterface;
use crate::error::VerifierResult;
use crate::message::Messages;
use crate::request::{get_response_body, get_response_headers, ContentType};
use crate::test_type::query::Query;
use crate::test_type::Verifier;

pub struct CachedQuery {
    pub concurrency_levels: Vec<i64>,
    pub database_verifier: Box<dyn DatabaseInterface>,
}
impl Query for CachedQuery {}
impl Verifier for CachedQuery {
    fn verify(&self, url: &str) -> VerifierResult<Messages> {
        let mut messages = Messages::new(url);

        // Initialization for query counting
        let repetitions = 2;
        let concurrency = *self.concurrency_levels.iter().max().unwrap();
        let expected_queries = 20 * repetitions * concurrency;
        let _expected_rows = expected_queries;

        let response_headers = get_response_headers(&url)?;
        messages.headers(&response_headers);
        self.verify_headers(&response_headers, &url, ContentType::Json, &mut messages);

        let test_cases = ["2", "0", "foo", "501", ""];
        let min = 1;
        let max = 500;

        for test_case in test_cases.iter() {
            let expected_length = self.translate_query_count(*test_case, min, max);
            let url = format!("{}{}", url, test_case);

            let response_body = get_response_body(&url, &mut messages);
            messages.body(&response_body);
            self.verify_with_length(&response_body, expected_length, &mut messages);
        }

        Ok(messages)
    }
}
impl CachedQuery {}
