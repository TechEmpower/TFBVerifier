//! The `Database` module deals with the various database interactions that a
//! verifier might need.

mod mongodb;
pub(crate) mod mysql;
mod postgres;

use crate::database::mongodb::Mongodb;
use crate::database::mysql::Mysql;
use crate::database::postgres::Postgres;
use crate::error::VerifierError::InvalidDatabaseType;
use crate::error::VerifierResult;
use crate::message::Messages;
use crate::request::request;
use std::cmp;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::{AtomicI64, AtomicU32, Ordering};
use std::sync::Arc;
use strum_macros::EnumString;
use threadpool::ThreadPool;

#[derive(EnumString, Debug)]
#[strum(serialize_all = "lowercase")]
pub enum Database {
    Mysql,
    Postgres,
    Mongodb,
}
impl Database {
    /// Gets a `Box`ed `DatabaseVerifier` for the given `database_name`.
    pub fn get(database_name: &str) -> VerifierResult<Box<dyn DatabaseVerifier>> {
        if let Ok(database_type) = Database::from_str(&database_name.to_lowercase()) {
            return match database_type {
                Database::Mysql => Ok(Box::new(Mysql {})),
                Database::Postgres => Ok(Box::new(Postgres {})),
                Database::Mongodb => Ok(Box::new(Mongodb {})),
            };
        } else {
            let mut messages = Messages::default();
            messages.error(
                format!("Invalid database type: {}", database_name),
                "Invalid Database",
            );
        }
        Err(InvalidDatabaseType(database_name.to_string()))
    }
}

pub trait DatabaseVerifier {
    fn get_all_from_world_table(&self) -> Option<HashMap<i32, i32>>;

    /// Checks that the number queries issued by the application after
    /// requesting `url` a known number of times (given by
    /// `concurrency` * `repetitions`) are equal.
    fn verify_queries_count(
        &self,
        url: &str,
        table_name: &str,
        concurrency: i64,
        repetitions: i64,
        expected_queries: i64,
        messages: &mut Messages,
    ) {
        let all_queries_before_count = self.get_count_of_all_queries_for_table(table_name);

        self.issue_multi_query_requests(url, concurrency, repetitions, messages);

        let all_queries_after_count = self.get_count_of_all_queries_for_table(table_name);

        let queries = all_queries_after_count - all_queries_before_count;
        match queries.cmp(&expected_queries) {
            cmp::Ordering::Greater => messages.warning(format!("{} Executed queries in the database instead of {} expected. This number is excessively high.", queries, expected_queries), "Extra Queries"),
            cmp::Ordering::Less => messages.error(format!("Only {} executed queries in the database out of roughly {} expected.", queries, expected_queries), "Too Few Queries"),
            _ => {}
        };
    }

    /// Checks that the number of rows that were selected by the application
    /// after requesting `url` a known number of times (given by
    /// `concurrency` * `repetitions`) are equal.
    fn verify_rows_count(
        &self,
        url: &str,
        table_name: &str,
        concurrency: i64,
        repetitions: i64,
        expected_rows: i64,
        messages: &mut Messages,
    ) {
        let all_rows_selected_before_count = self.get_count_of_rows_selected_for_table(table_name);

        self.issue_multi_query_requests(url, concurrency, repetitions, messages);

        let all_rows_selected_after_count = self.get_count_of_rows_selected_for_table(table_name);

        let rows = all_rows_selected_after_count - all_rows_selected_before_count;
        match rows.cmp(&expected_rows) {
            cmp::Ordering::Greater => messages.warning(format!("{} Executed rows read in the database instead of {} expected. This number is excessively high.", rows, expected_rows), "Extra Rows"),
            cmp::Ordering::Less => messages.error(format!("Only {} executed rows read in the database out of roughly {} expected.", rows, expected_rows), "Too Few Rows"),
            _ => {}
        };
    }

    /// Issues `concurrency` requests to `url` exactly `repetition + 1` times
    /// in a concurrent fashion.
    ///
    /// In practice, this means that this function will spawn as many threads
    /// as cores are available, and each thread is going to issue a request to
    /// `url` in a loop over there being more requests to send while decreasing
    /// the number of requests to send on every iteration atomically, and
    /// blocks until all the threads have completed their work.
    ///
    /// For example, on a dual-core machine, this function will spawn 2 threads
    /// each of which will make a request to `url`, increment an atomic counter
    /// of successful or failured requests, decrement the shared remaining
    /// requests atomic counter, and loop until that counter has run out. At
    /// the end of this example, it is expected that each thread will have run
    /// 256 times (on average).
    ///
    /// Returns: `(success_count, failed_count)`
    fn issue_multi_query_requests(
        &self,
        url: &str,
        concurrency: i64,
        repetitions: i64,
        messages: &mut Messages,
    ) {
        let transaction_failures = Arc::new(AtomicU32::new(0));
        let transaction_successes = Arc::new(AtomicU32::new(0));
        for _ in 0..repetitions {
            let requests_to_send = Arc::new(AtomicI64::new(concurrency - 1));
            let pool = ThreadPool::new(num_cpus::get());

            for _ in 0..num_cpus::get() {
                let url = url.to_string();
                let transaction_failures = Arc::clone(&transaction_failures);
                let transaction_successes = Arc::clone(&transaction_successes);
                let requests = Arc::clone(&requests_to_send);
                pool.execute(move || loop {
                    let remaining = requests.load(Ordering::SeqCst);
                    if remaining <= 0 {
                        break;
                    }
                    match request(&*url) {
                        Ok(_) => transaction_successes.fetch_add(1, Ordering::SeqCst),
                        Err(_) => transaction_failures.fetch_add(1, Ordering::SeqCst),
                    };
                    requests.fetch_sub(1, Ordering::SeqCst);
                });
            }
            pool.join();
        }

        let failures = transaction_failures.load(Ordering::SeqCst);
        if failures > 0 {
            messages.error(
                format!("Failed response(s) from {}: {}", url, failures),
                "Failed Response",
            );
        }
        let successes = transaction_successes.load(Ordering::SeqCst);
        if successes as i64 != concurrency * repetitions {
            messages.error(
                format!("Unexpected response count from {}: {}", url, successes),
                "Unexpected Responses",
            );
        }
    }

    fn get_count_of_all_queries_for_table(&self, table_name: &str) -> i64;

    fn get_count_of_rows_selected_for_table(&self, table_name: &str) -> i64;

    fn get_count_of_rows_updated_for_table(&self, table_name: &str) -> i64;
}

//
// TESTS
//

#[cfg(test)]
mod tests {
    use crate::database::Database;

    #[test]
    fn it_should_get_mysql() {
        if Database::get("mysql").is_err() {
            panic!("mysql test type broken");
        }
    }

    #[test]
    fn it_should_get_postgres() {
        if Database::get("postgres").is_err() {
            panic!("postgres test type broken");
        }
    }

    #[test]
    fn it_should_get_mongodb() {
        if Database::get("mongodb").is_err() {
            panic!("mongodb test type broken");
        }
    }
}
