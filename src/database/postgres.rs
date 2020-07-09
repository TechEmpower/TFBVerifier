use crate::database::DatabaseVerifier;
use crate::logger::{log, LogOptions};
use crate::message::Messages;
use colored::Colorize;
use postgres::{Client, NoTls};

pub struct Postgres {}
impl Postgres {
    /// Queries the PostgreSQL database for the number of queries at this
    /// moment and returns that count.
    fn get_queries(&self, table_name: &str) -> i32 {
        let query = format!(
            "SELECT CAST(SELECT SUM(calls) FROM pg_stat_statements WHERE query ~* '[[:<:]]{}[[:>:]]') AS INTEGER",
            table_name
        );
        match Client::connect(
            "postgresql://benchmarkdbuser:benchmarkdbpass@tfb-database/hello_world",
            NoTls,
        ) {
            Ok(mut client) => match client.query(&*query, &[]) {
                Ok(rows) => {
                    for row in rows {
                        dbg!(&row);
                        let sum: i32 = row.get("sum");
                        eprintln!("sum: {}", sum);
                    }
                }
                Err(e) => {
                    dbg!(e);
                }
            },
            Err(e) => {
                dbg!(e);
            }
        };
        0
    }
}
impl DatabaseVerifier for Postgres {
    fn verify_queries_count(
        &self,
        url: &str,
        table_name: &str,
        concurrency: i32,
        repetitions: i32,
        _expected_queries: i32,
        _check_updates: bool,
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

        self.get_queries(table_name);

        // todo - ask postgres for the number of db rows read

        let (successes, failures) = self.issue_multi_query_requests(url, concurrency, repetitions);

        log(
            format!(
                "Successful requests: {}, failed requests: {}",
                successes, failures
            )
            .normal(),
            LogOptions {
                border: None,
                border_bottom: None,
                quiet: false,
            },
        );

        // todo - ask postgres for the number of db queries run again; find difference

        // todo - ask postgres for the number of db rows read again; find difference

        // todo - logic for whether the test passed/errored (verify_queries_count)
    }

    fn verify_fortunes_are_dynamically_sized(&self, _messages: &mut Messages) {}
}
