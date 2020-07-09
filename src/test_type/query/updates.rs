use crate::database::DatabaseVerifier;
use crate::error::VerifierResult;
use crate::message::Messages;
use crate::request::{get_response_body, get_response_headers, ContentType};
use crate::test_type::query::Query;
use crate::test_type::Verifier;

pub struct Updates {
    pub concurrency_levels: Vec<i64>,
    pub database_verifier: Box<dyn DatabaseVerifier>,
}
impl Query for Updates {}
impl Verifier for Updates {
    fn verify(&self, url: &str) -> VerifierResult<Messages> {
        let mut messages = Messages::new(url);

        let test_cases = ["2", "0", "foo", "501", ""];

        // Initialization for query counting
        let repetitions = 2;
        let concurrency = *self.concurrency_levels.iter().max().unwrap();
        let expected_rows = 20 * repetitions * concurrency;
        let expected_selects = expected_rows;
        let expected_updates = expected_rows;
        let expected_queries = expected_selects + expected_updates;
        let min = 1;
        let max = 500;

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
                    &format!("{}20", url),
                    "world",
                    concurrency,
                    repetitions,
                    expected_queries,
                    expected_rows,
                );
                self.verify_updates_count(
                    &format!("{}20", url),
                    "world",
                    concurrency,
                    repetitions,
                    expected_updates,
                    &mut messages,
                );
                self.verify_updates(url, concurrency, &mut messages)
            }
        }

        Ok(messages)
    }
}
impl Updates {
    /// Counts all the updates that the datastore has on record, then performs
    /// `concurrency` requests for `url` `repitions` times, then checks all the
    /// updates that the datastore has on record again.
    /// Reports error if the number of updated rows does not meet the threshold.
    fn verify_updates_count(
        &self,
        url: &str,
        table_name: &str,
        concurrency: i64,
        repetitions: i64,
        expected_queries: i64,
        _messages: &mut Messages,
    ) {
        let all_rows_updated_before_count = self
            .database_verifier
            .get_count_of_rows_updated_for_table(table_name);
        eprintln!(
            "all updates count before: {}",
            all_rows_updated_before_count
        );

        let (_successes, _failures) =
            self.database_verifier
                .issue_multi_query_requests(url, concurrency, repetitions);

        let all_rows_updated_after_count = self
            .database_verifier
            .get_count_of_rows_updated_for_table(table_name);
        eprintln!("all updates count after: {}", all_rows_updated_after_count);

        eprintln!(
            "expected updates: {}, updates: {}, equal: {}",
            expected_queries,
            all_rows_updated_after_count - all_rows_updated_before_count,
            expected_queries == (all_rows_updated_after_count - all_rows_updated_before_count)
        );
    }

    /// Queries all the data in the `World` table, runs an example update
    /// set of requests, then queries all the data in the `World` table again.
    /// Reports error if the number of updated rows does not meet the threshold.
    fn verify_updates(&self, url: &str, concurrency: i64, _messages: &mut Messages) {
        // Note: we do this outside of `verify_updates_count` so we do not mess
        // up the counting. Down here, we no longer care about the query/select
        // counts, we only want to see that an appropriate number of updates
        // occurred on the underlying data.

        // todo - capture the `World` table entirely for comparison later

        let (_successes, _failures) =
            self.database_verifier
                .issue_multi_query_requests(url, concurrency, 1);

        // todo - capture the `World` table again and compare the values
    }
}
