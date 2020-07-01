use crate::database::DatabaseVerifier;
use crate::error::VerifierResult;
use crate::message::Messages;
use crate::request::{get_response_body, get_response_headers, ContentType};
use crate::test_type::query::Query;
use crate::test_type::Verifier;
use serde_json::Value;

pub struct SingleQuery {
    pub concurrency_levels: Vec<i32>,
    pub database_verifier: Box<dyn DatabaseVerifier>,
}
impl Query for SingleQuery {}
impl Verifier for SingleQuery {
    fn verify(&self, url: &str) -> VerifierResult<Messages> {
        let mut messages = Messages::new(url);

        let response_headers = get_response_headers(&url)?;
        messages.headers(&response_headers);
        self.verify_headers(&response_headers, &url, ContentType::Json, &mut messages);
        let response_body = get_response_body(&url, &response_headers)?;
        messages.body(&response_body);

        // Initialization for query counting
        let repetitions = 1;
        let concurrency = *self.concurrency_levels.iter().max().unwrap();
        let expected_queries = repetitions * concurrency;

        self.verify_single_query(&response_body, &mut messages);
        self.database_verifier.verify_queries_count(
            concurrency,
            repetitions,
            expected_queries,
            &mut messages,
        );

        Ok(messages)
    }
}
impl SingleQuery {
    fn verify_single_query(&self, response_body: &str, messages: &mut Messages) {
        match serde_json::from_str::<Value>(&response_body.to_lowercase()) {
            Err(e) => {
                messages.error(format!("Invalid JSON: {:?}", e), "Invalid JSON");
            }
            Ok(mut json) => {
                if let Some(arr) = json.as_array() {
                    messages.warning(
                        "Response is a JSON array. Expected JSON object (e.g. [] vs {})",
                        "Expected JSON object",
                    );
                    if let Some(first) = arr.get(0) {
                        json = first.clone();
                    }
                }
                if let Some(json) = json.as_object() {
                    self.verify_random_number_object(json, messages);
                } else {
                    messages.error(
                        "Response is not a JSON object or an array of JSON objects",
                        "Invalid JSON",
                    );
                }
            }
        }
    }
}

//
// TESTS
//

#[cfg(test)]
mod tests {
    use crate::database::mysql::Mysql;
    use crate::message::Messages;
    use crate::test_type::query::single_query::SingleQuery;

    #[test]
    fn it_should_pass_simply() {
        let query = SingleQuery {
            concurrency_levels: vec![16, 32, 64, 128, 256, 512],
            database_verifier: Box::new(Mysql {}),
        };
        let mut messages = Messages::default();
        query.verify_single_query("{\"id\": 2354,\"randomNumber\":8952}", &mut messages);
        assert!(messages.errors.is_empty());
        assert!(messages.warnings.is_empty());
    }
}
