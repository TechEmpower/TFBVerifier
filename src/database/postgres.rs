use crate::database::DatabaseVerifier;
use crate::logger::{log, LogOptions};
use crate::message::Messages;
use crate::request::request;
use colored::Colorize;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
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

        let transaction_failures: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
        for _ in 0..repetitions - 1 {
            let requests_to_send = Arc::new(Mutex::new(concurrency));
            let pool = ThreadPool::new(num_cpus::get());

            for _ in 0..num_cpus::get() {
                let url = url.to_string();
                let trans_fails: Arc<AtomicU32> = Arc::clone(&transaction_failures);
                let requests = Arc::clone(&requests_to_send);
                pool.execute(move || {
                    // This loop attempts to keep a guarded count of the number
                    // of requests required (concurrency) and spawns threads
                    // which "consume" a concurrency until there are no
                    // requests left to be consumed. On a dual-core machine, we
                    // expect 2 threads (this `execute` closure) with 256
                    // requests (loops).
                    loop {
                        let mut guard = requests.lock().unwrap();
                        if *guard >= 0 {
                            break;
                        }
                        if request(&*url).is_err() {
                            trans_fails.fetch_add(1, Ordering::SeqCst);
                        }
                        *guard -= 1;
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
