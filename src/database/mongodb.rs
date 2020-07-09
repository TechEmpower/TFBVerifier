use crate::database::DatabaseVerifier;
use crate::message::Messages;

pub struct Mongodb {}
impl DatabaseVerifier for Mongodb {
    fn verify_queries_count(
        &self,
        _url: &str,
        _table_name: &str,
        _concurrency: i32,
        _repetitions: i32,
        _expected_queries: i32,
        _check_updates: bool,
        _messages: &mut Messages,
    ) {
    }

    fn verify_fortunes_are_dynamically_sized(&self, _messages: &mut Messages) {}
}
