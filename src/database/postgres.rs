use crate::database::DatabaseVerifier;
use crate::logger::{log, LogOptions};
use crate::message::Messages;
use crate::request::request;
use colored::Colorize;
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};
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
                border: None,
                border_bottom: None,
                quiet: false,
            },
        );

        // todo - ask postgres for the number of db queries run

        // todo - ask postgres for the number of db rows read

        let transaction_failures = Arc::new(AtomicU32::new(0));
        let transaction_successes = Arc::new(AtomicU32::new(0));
        for _ in 0..repetitions {
            let requests_to_send = Arc::new(AtomicI32::new(concurrency));
            let pool = ThreadPool::new(num_cpus::get());

            for _ in 0..num_cpus::get() {
                let url = format!("{}20", url);
                let trans_fails = Arc::clone(&transaction_failures);
                let trans_succs = Arc::clone(&transaction_successes);
                let requests = Arc::clone(&requests_to_send);
                pool.execute(move || {
                    // This loop attempts to keep a guarded count of the number
                    // of requests required (concurrency) and spawns threads
                    // which "consume" a concurrency until there are no
                    // requests left to be consumed. On a dual-core machine, we
                    // expect 2 threads (this `execute` closure) with 256
                    // requests (loops).
                    loop {
                        let remaining = requests.load(Ordering::SeqCst);
                        if remaining <= 0 {
                            break;
                        }
                        match request(&*url) {
                            Ok(_) => trans_succs.fetch_add(1, Ordering::SeqCst),
                            Err(_) => trans_fails.fetch_add(1, Ordering::SeqCst),
                        };
                        requests.fetch_sub(1, Ordering::SeqCst);
                    }
                });
            }
            pool.join();
        }
        eprintln!(
            "Successful requests: {}, failed requests: {}",
            transaction_successes.load(Ordering::SeqCst),
            transaction_failures.load(Ordering::SeqCst)
        );

        // todo - ask postgres for the number of db queries run again; find difference

        // todo - ask postgres for the number of db rows read again; find difference

        // todo - logic for whether the test passed/errored (verify_queries_count)
    }

    fn verify_fortunes_are_dynamically_sized(&self, _messages: &mut Messages) {}
}
