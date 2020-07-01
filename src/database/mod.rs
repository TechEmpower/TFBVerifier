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
use std::str::FromStr;
use strum_macros::EnumString;

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
        concurrency: i32,
        repetitions: i32,
        expected_queries: i32,
        messages: &mut Messages,
    );

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
