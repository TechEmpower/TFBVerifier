//! This module is used for defining valid `TestType`s as well as constructing
//! the corresponding `Executor`.
//!
//! Note: adding a new type of test to the suite requires the following action:
//!
//!  1. Add the new test to the `TestType` enum
//!  2. Implement the new test type `Executor` trait
//!     (see [json](crate::test_type::json::Json) for an example)
//!  3. Implement the branch of the `match` in `get_executor` for the new `TestType`
//!

mod fortune;
mod json;
mod plaintext;
mod query;
mod unknown;

use crate::database::Database;
use crate::error::VerifierResult;
use crate::message::Messages;
use crate::request::{get_response_headers, ContentType};
use crate::test_type::fortune::Fortune;
use crate::test_type::json::Json;
use crate::test_type::plaintext::Plaintext;
use crate::test_type::query::cached_query::CachedQuery;
use crate::test_type::query::multi_query::MultiQuery;
use crate::test_type::query::single_query::SingleQuery;
use crate::test_type::query::updates::Updates;
use crate::test_type::unknown::Unknown;
use std::collections::HashMap;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use strum_macros::EnumString;

/// Enumerates all the test types about which this project is aware. In order
/// to obtain an `Executor` for processing either a verification or a benchmark
/// of a URL, the test type must be one of these enumerates `TestTypes` *and*
/// have a corresponding `Executor` implementation.
#[derive(EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum TestType {
    Json,
    // left as `db` for legacy support
    #[strum(serialize = "db")]
    SingleQuery,
    #[strum(serialize = "cached_query")]
    CachedQuery,
    // left as `query` for legacy support
    #[strum(serialize = "query")]
    MultiQuery,
    Fortune,
    Update,
    Plaintext,
    Unknown(String),
}
impl TestType {
    /// Helper function for getting a `TestType` from `test_type_name`.
    pub fn get(test_type_name: &str) -> VerifierResult<TestType> {
        if let Ok(test_type) = TestType::from_str(&test_type_name.to_lowercase()) {
            Ok(test_type)
        } else {
            Ok(TestType::Unknown(test_type_name.to_string()))
        }
    }

    /// Gets an `Executor` for the given `test_type_name`.
    pub fn get_executor(
        &self,
        database_name: Option<String>,
        concurrency_levels: Vec<i64>,
    ) -> VerifierResult<Box<dyn Executor>> {
        let database = if let Some(name) = database_name {
            Some(Database::get(&name)?)
        } else {
            None
        };
        match self {
            TestType::Json => Ok(Box::new(Json {})),
            TestType::SingleQuery => Ok(Box::new(SingleQuery {
                database_verifier: database.unwrap(),
                concurrency_levels,
            })),
            TestType::MultiQuery => Ok(Box::new(MultiQuery {
                database_verifier: database.unwrap(),
                concurrency_levels,
            })),
            TestType::CachedQuery => Ok(Box::new(CachedQuery {
                database_verifier: database.unwrap(),
                concurrency_levels,
            })),
            TestType::Fortune => Ok(Box::new(Fortune {
                database_verifier: database.unwrap(),
                concurrency_levels,
            })),
            TestType::Update => Ok(Box::new(Updates {
                database_verifier: database.unwrap(),
                concurrency_levels,
            })),
            TestType::Plaintext => Ok(Box::new(Plaintext {})),
            TestType::Unknown(test_type) => Ok(Box::new(Unknown {
                test_type: test_type.clone(),
            })),
        }
    }
}

/// The `Executor` trait is how the entire orchestration of verification and
/// benchmarking works.
///
/// `Executor` implementors are the masters of their own destinies - since only
/// a url is provided, it is expected (though, not strictly required) that the
/// implementation will request said url, capture the response headers and
/// body, and against them perform a verification or benchmark.
pub trait Executor {
    /// Benchmarks the given `url`.
    ///
    /// Note: this method is not expected to produce results of the benchmark
    /// in a consumable way for the purposes of this application; rather, it
    /// should send the output of the benchmark to `stdout` with the
    /// understanding that the caller of this application will consume.
    fn benchmark(&self, url: &str) -> VerifierResult<()>;

    /// Verifies the given `url`.
    fn verify(&self, url: &str) -> VerifierResult<Messages>;

    /// Verifies the headers of a framework response
    /// `should_be` is a switch for the acceptable content types
    fn verify_headers(
        &self,
        headers: &HashMap<String, String>,
        url: &str,
        should_be: ContentType,
        messages: &mut Messages,
    ) {
        verify_headers_internal(headers, url, should_be, true, messages)
    }
}

//
// PRIVATES
//

fn verify_headers_internal(
    headers: &HashMap<String, String>,
    url: &str,
    _should_be: ContentType,
    should_retest: bool,
    messages: &mut Messages,
) {
    if !headers.contains_key("Server") {
        messages.error("Required response header missing: Server", "Missing header");
    }
    if !headers.contains_key("Date") {
        messages.error("Required response header missing: Date", "Missing header");
    }
    if !headers.contains_key("Content-Type") {
        messages.error(
            "Required response header missing: Content-Type",
            "Missing header",
        );
    }
    if !headers.contains_key("Content-Length") && !headers.contains_key("Transfer-Encoding") {
        messages.error("Required response size header missing, please include either \"Content-Length\" or \"Transfer-Encoding\"", "Missing header");
    }
    if let Some(_date) = headers.get("Date") {
        // todo - check format is '%a, %d %b %Y %H:%M:%S %Z'
    }
    if should_retest {
        sleep(Duration::from_secs(3));
        if let Ok(_response_headers) = get_response_headers(url) {
            // todo - Make sure that the date object isn't cached
        }
    }

    if let Some(_content_type) = headers.get("Content-Type") {
        // todo - match a regexp - should probably be enum function
        // 'json':      '^application/json(; ?charset=(UTF|utf)-8)?$',
        // 'html':      '^text/html; ?charset=(UTF|utf)-8$',
        // 'plaintext': '^text/plain(; ?charset=(UTF|utf)-8)?$'
    }
}

//
// TESTS
//

#[cfg(test)]
mod tests {
    use crate::message::Messages;
    use crate::request::ContentType;
    use crate::test_type::{verify_headers_internal, TestType};
    use std::collections::HashMap;

    //
    // verify_headers
    //

    #[test]
    fn it_should_error_on_missing_headers() {
        let map = HashMap::new();
        let mut messages = Messages::default();
        verify_headers_internal(
            &map,
            "http://google.com",
            ContentType::Json,
            false,
            &mut messages,
        );
        let mut server = false;
        let mut date = false;
        let mut content = false;
        let mut transfer = false;
        for error in messages.errors {
            if error
                .message
                .contains("Required response header missing: Server")
            {
                server = true;
            }
            if error
                .message
                .contains("Required response header missing: Date")
            {
                date = true;
            }
            if error
                .message
                .contains("Required response header missing: Content-Type")
            {
                content = true;
            }
            if error
                .message
                .contains("Required response size header missing")
            {
                transfer = true;
            }
        }
        assert!(server);
        assert!(date);
        assert!(content);
        assert!(transfer);
    }

    //
    // verify test types
    //
    #[test]
    fn it_should_get_json() {
        if TestType::get("json").is_err() {
            panic!("json test type broken");
        }
    }
    #[test]
    fn it_should_get_db() {
        if TestType::get("db").is_err() {
            panic!("db test type broken");
        }
    }
    #[test]
    fn it_should_get_query() {
        if TestType::get("query").is_err() {
            panic!("query test type broken");
        }
    }
    #[test]
    fn it_should_get_cached_query() {
        if TestType::get("cached_query").is_err() {
            panic!("cached_query test type broken");
        }
    }
    #[test]
    fn it_should_get_update() {
        if TestType::get("update").is_err() {
            panic!("update test type broken");
        }
    }
    #[test]
    fn it_should_get_fortune() {
        if TestType::get("fortune").is_err() {
            panic!("fortune test type broken");
        }
    }
    #[test]
    fn it_should_get_plaintext() {
        if TestType::get("plaintext").is_err() {
            panic!("plaintext test type broken");
        }
    }
}
