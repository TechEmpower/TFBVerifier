use crate::database::DatabaseVerifier;
use crate::message::Messages;

pub struct Postgres {}
impl DatabaseVerifier for Postgres {
    fn verify_queries_count(
        &self,
        _concurrency: i32,
        _repetitions: i32,
        _expected_queries: i32,
        _messages: &mut Messages,
    ) {
    }

    fn verify_fortunes_are_dynamically_sized(&self, messages: &mut Messages) {}
}
