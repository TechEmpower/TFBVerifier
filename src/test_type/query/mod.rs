pub(crate) mod cached_query;
pub(crate) mod multi_query;
pub(crate) mod single_query;
pub(crate) mod updates;

use crate::verification::Messages;
use serde_json::{Map, Value};
use std::str::FromStr;

pub trait Query {
    /// Ensures that `json` is a JSON object with keys 'id' and 'randomNumber'
    /// that both map to ints.
    ///
    /// Should closely resemble:
    ///
    /// `{"id": 2354,"randomNumber":8952}`
    fn verify_random_number_object(&self, json: &Map<String, Value>, messages: &mut Messages) {
        let mut id_found = false;
        let mut id_key = "id";
        let mut random_number_found = false;
        let mut random_number_key = "randomnumber";
        let mut keys = 0;
        let mut unknown_keys = String::new();
        for key in json.keys() {
            keys += 1;
            if key.to_lowercase() == "id" {
                id_found = true;
                id_key = key;
            } else if key.to_lowercase() == "randomnumber" {
                random_number_found = true;
                random_number_key = key;
            } else {
                unknown_keys.push_str(&format!("{}, ", key.to_lowercase()));
            }
        }
        if !id_found {
            messages.error(
                "Response object was missing required key: id",
                "Missing Key",
            );
        } else if !random_number_found {
            messages.error(
                "Response object was missing required key: randomnumber",
                "Missing Key",
            );
        } else {
            if keys > 2 {
                // Always ends with ", "
                unknown_keys.pop();
                unknown_keys.pop();
                let single = format!(
                    "An extra key is being included with the db object: {}",
                    unknown_keys
                );
                let plural = format!(
                    "Extra keys are being included with the db object: {}",
                    unknown_keys
                );
                let (warning, short) = match keys {
                    3 => (single, "Extra Key"),
                    _ => (plural, "Extra Keys"),
                };
                messages.warning(warning, short);
            }
            let id = {
                let mut tmp_id = json[id_key].as_i64();
                if let Some(id_str) = json[id_key].as_str() {
                    if let Ok(parsed_id) = i64::from_str(id_str) {
                        messages.warning(
                            format!("Response key 'id' is int-string; should be int: {}. This may negatively affect performance by sending extra bytes.", id_str),
                            "Extra Bytes"
                        );
                        tmp_id = Some(parsed_id);
                    }
                }
                if tmp_id.is_none() {
                    messages.error(
                        format!(
                            "Response key 'id' does not map to an integer: {}",
                            json[id_key]
                        ),
                        "Invalid Value",
                    );
                }
                tmp_id.unwrap_or(0)
            };

            if id > 10_000 {
                messages.warning(
                    format!("Response key 'id' should be between 1 and 10,000: {}", id),
                    "Value Out of Range",
                );
            }

            if let Some(random_number) = json[random_number_key].as_i64() {
                if random_number < 1 {
                    messages.error(
                        format!(
                            "Response key 'randomnumber' must be greater than zero: {}",
                            random_number
                        ),
                        "Invalid Value",
                    );
                } else if random_number > 10_000 {
                    messages.warning(
                        "Response key `randomNumber` is over 10,000. This may negatively affect performance by sending extra bytes.",
                        "Value Out of Range"
                    );
                }
            } else {
                messages.error(
                    format!(
                        "Response key 'randomnumber' does not map to an integer: {}",
                        json[random_number_key]
                    ),
                    "Invalid Value",
                );
            }
        }
    }

    /// Verifies the given `response_body` and `expected_count`.
    fn verify_with_length(
        &self,
        response_body: &str,
        expected_count: i32,
        messages: &mut Messages,
    ) {
        match serde_json::from_str::<Value>(&response_body.to_lowercase()) {
            Err(e) => {
                messages.error(format!("Invalid JSON: {:?}", e), "Invalid JSON");
            }
            Ok(json) => {
                if let Some(list) = json.as_array() {
                    for obj in list {
                        if let Some(json) = obj.as_object() {
                            self.verify_random_number_object(json, messages);
                            // There isn't much sense having 500 errors/warnings for the same
                            // random number object validation issue. Walk each item and verify
                            // it is a valid json, break out on the first error/warning.
                            if !messages.warnings.is_empty() || !messages.errors.is_empty() {
                                break;
                            }
                        }
                    }
                    if list.len() != expected_count as usize {
                        messages.error(
                            format!(
                                "JSON array length of {} != expected length of {}",
                                list.len(),
                                expected_count
                            ),
                            "Incorrect Length",
                        );
                    }
                } else if let Some(object) = json.as_object() {
                    messages.warning("Top-level JSON is an object, not an array", "Invalid JSON");
                    self.verify_random_number_object(object, messages);
                }
            }
        }
    }

    /// Helper function for returning the translated query string.
    fn translate_query_count(&self, query_string: &str, min: i32, max: i32) -> i32 {
        if let Ok(queries) = i32::from_str(query_string) {
            if queries > max {
                max
            } else if queries < min {
                min
            } else {
                queries
            }
        } else {
            min
        }
    }
}

//
// PRIVATES
//
struct _QueryTest {}
impl Query for _QueryTest {}

//
// TESTS
//

#[cfg(test)]
mod tests {

    #[test]
    fn it_should_translate_correctly() {
        let query_test = _QueryTest {};

        assert_eq!(query_test.translate_query_count("2", 1, 500), 2);
        assert_eq!(query_test.translate_query_count("0", 1, 500), 1);
        assert_eq!(query_test.translate_query_count("foo", 1, 500), 1);
        assert_eq!(query_test.translate_query_count("501", 1, 500), 500);
        assert_eq!(query_test.translate_query_count("", 1, 500), 1);
    }

    //
    // verify_random_number_object
    //

    use crate::verification::Messages;
    use crate::test_type::query::{Query, _QueryTest};
    use serde_json::Value;

    #[test]
    fn it_should_succeed_on_valid_db_object() {
        let json = serde_json::from_str::<Value>("{\"id\":1234,\"randomnumber\":4321}").unwrap();
        let query_test = _QueryTest {};

        let mut messages = Messages::default();
        query_test.verify_random_number_object(json.as_object().unwrap(), &mut messages);

        assert!(messages.errors.is_empty());
        assert!(messages.warnings.is_empty());
    }

    #[test]
    fn it_should_error_on_missing_id_key() {
        let json = serde_json::from_str::<Value>("{\"randomnumber\":4321}").unwrap();
        let query_test = _QueryTest {};

        let mut messages = Messages::default();
        query_test.verify_random_number_object(json.as_object().unwrap(), &mut messages);

        assert!(messages.warnings.is_empty());
        assert!(!messages.errors.is_empty());
        assert!(messages
            .errors
            .get(0)
            .unwrap()
            .message
            .contains("missing required key: id"));
    }

    #[test]
    fn it_should_error_on_missing_random_number_key() {
        let json = serde_json::from_str::<Value>("{\"id\":1234}").unwrap();
        let query_test = _QueryTest {};

        let mut messages = Messages::default();
        query_test.verify_random_number_object(json.as_object().unwrap(), &mut messages);

        assert!(messages.warnings.is_empty());
        assert!(!messages.errors.is_empty());
        assert!(messages
            .errors
            .get(0)
            .unwrap()
            .message
            .contains("missing required key: randomnumber"));
    }

    #[test]
    fn it_should_error_on_random_number_less_than_one() {
        let json = serde_json::from_str::<Value>("{\"id\":1234,\"randomnumber\":0}").unwrap();
        let query_test = _QueryTest {};

        let mut messages = Messages::default();
        query_test.verify_random_number_object(json.as_object().unwrap(), &mut messages);

        assert!(messages.warnings.is_empty());
        assert!(!messages.errors.is_empty());
        assert!(messages
            .errors
            .get(0)
            .unwrap()
            .message
            .contains("must be greater than zero"));
    }

    #[test]
    fn it_should_error_on_id_being_non_integer() {
        let json = serde_json::from_str::<Value>("{\"id\":\"asd\",\"randomnumber\":1}").unwrap();
        let query_test = _QueryTest {};

        let mut messages = Messages::default();
        query_test.verify_random_number_object(json.as_object().unwrap(), &mut messages);

        assert!(messages.warnings.is_empty());
        assert!(!messages.errors.is_empty());
        assert!(messages
            .errors
            .get(0)
            .unwrap()
            .message
            .contains("does not map to an integer"));
    }

    #[test]
    fn it_should_warning_on_id_being_int_str() {
        let json = serde_json::from_str::<Value>("{\"id\":\"123\",\"randomnumber\":1}").unwrap();
        let query_test = _QueryTest {};

        let mut messages = Messages::default();
        query_test.verify_random_number_object(json.as_object().unwrap(), &mut messages);

        assert!(!messages.warnings.is_empty());
        assert!(messages.errors.is_empty());
        assert!(messages
            .warnings
            .get(0)
            .unwrap()
            .message
            .contains("int-string; should be int"));
    }

    #[test]
    fn it_should_warn_on_id_above_ten_thousand() {
        let json = serde_json::from_str::<Value>("{\"id\":12345,\"randomnumber\":4321}").unwrap();
        let query_test = _QueryTest {};

        let mut messages = Messages::default();
        query_test.verify_random_number_object(json.as_object().unwrap(), &mut messages);

        assert!(messages.errors.is_empty());
        assert!(!messages.warnings.is_empty());
        assert!(messages
            .warnings
            .get(0)
            .unwrap()
            .message
            .contains("should be between 1 and 10,000"));
    }

    #[test]
    fn it_should_warn_on_random_number_above_ten_thousand() {
        let json = serde_json::from_str::<Value>("{\"id\":1234,\"randomnumber\":43210}").unwrap();
        let query_test = _QueryTest {};

        let mut messages = Messages::default();
        query_test.verify_random_number_object(json.as_object().unwrap(), &mut messages);

        assert!(messages.errors.is_empty());
        assert!(!messages.warnings.is_empty());
        assert!(messages
            .warnings
            .get(0)
            .unwrap()
            .message
            .contains("is over 10,000"));
    }

    #[test]
    fn it_should_warn_on_extra_keys() {
        let json =
            serde_json::from_str::<Value>("{\"id\":1234,\"randomnumber\":4321,\"foo\":\"bar\"}")
                .unwrap();
        let query_test = _QueryTest {};

        let mut messages = Messages::default();
        query_test.verify_random_number_object(json.as_object().unwrap(), &mut messages);

        assert!(messages.errors.is_empty());
        assert!(!messages.warnings.is_empty());
        assert!(messages
            .warnings
            .get(0)
            .unwrap()
            .message
            .contains("extra key is being included"));
    }

    #[test]
    fn it_should_pass_count_one() {
        let query_test = _QueryTest {};
        let mut messages = Messages::default();
        query_test.verify_with_length("[{\"id\":1234,\"randomnumber\":4321}]", 1, &mut messages);

        assert!(messages.errors.is_empty());
        assert!(messages.warnings.is_empty());
    }

    #[test]
    fn it_should_pass_count_two_and_more_proof_by_induction() {
        let query_test = _QueryTest {};
        let mut messages = Messages::default();
        query_test.verify_with_length(
            "[{\"id\":1234,\"randomnumber\":4321},{\"id\":4567,\"randomnumber\":1234}]",
            2,
            &mut messages,
        );

        assert!(messages.errors.is_empty());
        assert!(messages.warnings.is_empty());
    }

    #[test]
    fn it_should_warn_on_object_instead_of_array() {
        let query_test = _QueryTest {};
        let mut messages = Messages::default();
        query_test.verify_with_length("{\"id\":1234,\"randomnumber\":4321}", 1, &mut messages);

        assert!(messages.errors.is_empty());
        assert!(!messages.warnings.is_empty());
        assert!(messages
            .warnings
            .get(0)
            .unwrap()
            .message
            .contains("JSON is an object, not an array"));
    }
}
