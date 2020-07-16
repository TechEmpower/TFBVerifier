use crate::database::DatabaseInterface;
use crate::error::VerifierResult;
use crate::message::Messages;
use crate::request::{get_response_body, get_response_headers, ContentType};
use crate::test_type::query::Query;
use crate::test_type::Verifier;
use std::cmp;

pub struct Updates {
    pub concurrency_levels: Vec<i64>,
    pub database_verifier: Box<dyn DatabaseInterface>,
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
                self.verify_updates_count(
                    &format!("{}20", url),
                    "world",
                    concurrency,
                    repetitions,
                    expected_updates,
                    &mut messages,
                );
                self.verify_updates(
                    &format!("{}20", url),
                    concurrency,
                    repetitions,
                    &mut messages,
                )
            }
        }

        Ok(messages)
    }
}
impl Updates {
    /// Counts all the updates that the datastore has on record, then performs
    /// `concurrency` requests for `url` `repetitions` times, then checks all
    /// the updates that the datastore has on record again.
    /// Reports error if the number of updated rows does not meet the threshold.
    fn verify_updates_count(
        &self,
        url: &str,
        table_name: &str,
        concurrency: i64,
        repetitions: i64,
        expected_updates: i64,
        messages: &mut Messages,
    ) {
        let all_rows_updated_before_count = self
            .database_verifier
            .get_count_of_rows_updated_for_table(table_name);

        self.database_verifier
            .issue_multi_query_requests(url, concurrency, repetitions, messages);

        let all_rows_updated_after_count = self
            .database_verifier
            .get_count_of_rows_updated_for_table(table_name);

        let updated = all_rows_updated_after_count - all_rows_updated_before_count;
        // Note: Some database implementations are less accurate (though still
        // precise) than others, and sometimes over-report rows updated. We do
        // not warn because it would just be noisy over something out of the
        // implementer's control.
        if let cmp::Ordering::Less = updated.cmp(&expected_updates) {
            messages.error(
                format!(
                    "Only {} executed rows updated in the database out of roughly {} expected.",
                    updated, expected_updates
                ),
                "Too Few Rows",
            )
        };
    }

    /// Queries all the data in the `World` table, runs an example update
    /// set of requests, then queries all the data in the `World` table again.
    /// Reports error if the number of updated rows does not meet the threshold.
    fn verify_updates(
        &self,
        url: &str,
        concurrency: i64,
        repetitions: i64,
        messages: &mut Messages,
    ) {
        let expected_updates = concurrency * repetitions;
        // Note: we do this outside of `verify_updates_count` so we do not mess
        // up the counting. Down here, we no longer care about the query/select
        // counts, we only want to see that an appropriate number of updates
        // occurred on the underlying data.

        let worlds_before = self.database_verifier.get_all_from_world_table();

        self.database_verifier
            .issue_multi_query_requests(url, concurrency, 1, messages);

        let worlds_after = self.database_verifier.get_all_from_world_table();

        let mut updates = 0;
        for index in 0..worlds_before.len() {
            if worlds_before.get(&(index as i32)).is_some()
                && worlds_after.get(&(index as i32)).is_some()
                && worlds_before.get(&(index as i32)).unwrap()
                    != worlds_after.get(&(index as i32)).unwrap()
            {
                updates += 1;
            }
        }

        if updates == 0 {
            messages.error("No items were updated in the database.", "No Updates");
        } else if updates <= (expected_updates as f32 * 0.90) as i32 {
            messages.error(
                format!(
                    "Only {} items were updated in the database out of roughly {} expected.",
                    updates, expected_updates
                ),
                "Too Few Updates",
            );
        } else if updates <= (expected_updates as f32 * 0.95) as i32 {
            messages.warning(format!("There may have been an error updating the database. Only {} items were updated in the database out of the roughly {} expected.", updates, expected_updates), "Too Few Updates");
        }
    }
}
