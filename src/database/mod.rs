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
use std::str::FromStr;
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};
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
    /// Checks that the number of executed queries, at the given concurrency
    /// level, corresponds to: the total number of http requests made * the
    /// number of queries per request.
    ///
    /// No margin is accepted on the number of queries, which seems reliable.
    ///
    /// On the number of rows read or updated, the margin related to the
    /// database applies (1% by default see cls.margin)
    ///
    /// On updates, if the use of bulk updates is detected (number of requests
    /// close to that expected), a margin (5% see bulk_margin) is allowed on
    /// the number of updated rows.
    fn verify_queries_count(
        &self,
        url: &str,
        table_name: &str,
        concurrency: i32,
        repetitions: i32,
        expected_queries: i32,
        check_updates: bool,
        messages: &mut Messages,
    );

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
        concurrency: i32,
        repetitions: i32,
    ) -> (u32, u32) {
        let transaction_failures = Arc::new(AtomicU32::new(0));
        let transaction_successes = Arc::new(AtomicU32::new(0));
        for _ in 0..repetitions {
            let requests_to_send = Arc::new(AtomicI32::new(concurrency - 1));
            let pool = ThreadPool::new(num_cpus::get());

            for _ in 0..num_cpus::get() {
                let url = format!("{}20", url);
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
        (
            transaction_successes.load(Ordering::SeqCst),
            transaction_failures.load(Ordering::SeqCst),
        )
    }

    /// Checks that test implementations are using dynamically sized data
    /// structures when gathering fortunes from the database.
    ///
    /// In practice, this function will connect to the database and add several
    /// thousand fortunes, request the test implementation for its fortune test
    /// again, and compare to expected output.
    fn verify_fortunes_are_dynamically_sized(&self, messages: &mut Messages);
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
