use crate::database::DatabaseVerifier;
use crate::message::Messages;

pub struct Mysql {}
impl DatabaseVerifier for Mysql {
    fn verify_queries_count(
        &self,
        _concurrency: i32,
        _repetitions: i32,
        _expected_queries: i32,
        _messages: &mut Messages,
    ) {
    }
}
