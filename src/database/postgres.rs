use crate::database::DatabaseVerifier;
use crate::logger::{log, LogOptions};
use crate::message::Messages;
use crate::request::request;
use colored::Colorize;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use threadpool::ThreadPool;

pub struct Postgres {}
impl DatabaseVerifier for Postgres {
    fn verify_queries_count(
        &self,
        url: &str,
        concurrency: i32,
        repetitions: i32,
        _expected_queries: i32,
        _messages: &mut Messages,
    ) {
        log(
            format!("VERIFYING QUERY COUNT FOR {}", url).bright_white(),
            LogOptions {
                border: Some('-'),
                border_bottom: None,
                quiet: false,
            },
        );

        // todo - ask postgres for the number of db queries run

        // todo - ask postgres for the number of db rows read

        let transaction_failures: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
        for _ in 0..repetitions - 1 {
            let pool = ThreadPool::new(concurrency as usize);
            for _ in 0..concurrency {
                let url = url.to_string();
                let trans_fails: Arc<AtomicU32> = Arc::clone(&transaction_failures);
                pool.execute(move || {
                    if request(&*url).is_err() {
                        trans_fails.fetch_add(1, Ordering::SeqCst);
                    }
                });
            }
            pool.join();
        }

        // todo - ask postgres for the number of db queries run again; find difference

        // todo - ask postgres for the number of db rows read again; find difference

        // todo - logic for whether the test passed/errored (verify_queries_count)
    }

    fn verify_fortunes_are_dynamically_sized(&self, _messages: &mut Messages) {}
}
