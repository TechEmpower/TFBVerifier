use crate::benchmark::BenchmarkCommands;
use crate::error::VerifierResult;
use crate::request::{get_response_body, get_response_headers, ContentType};
use crate::test_type::Executor;
use crate::verification::Messages;
use serde_json::Value;
use std::cmp::min;

pub struct Json {
    pub concurrency_levels: Vec<usize>,
}
impl Executor for Json {
    fn retrieve_benchmark_commands(&self, url: &str) -> VerifierResult<BenchmarkCommands> {
        let primer_command = self.get_wrk_command(url, 5, 8);
        let warmup_command =
            self.get_wrk_command(url, 15, *self.concurrency_levels.iter().max().unwrap());
        let mut benchmark_commands = Vec::default();
        for concurrency in &self.concurrency_levels {
            benchmark_commands.push(self.get_wrk_command(url, 15, *concurrency));
        }

        Ok(BenchmarkCommands {
            primer_command,
            warmup_command,
            benchmark_commands,
        })
    }

    fn verify(&self, url: &str) -> VerifierResult<Messages> {
        let mut messages = Messages::new(url);

        let response_headers = get_response_headers(&url)?;
        messages.headers(&response_headers);
        self.verify_headers(&response_headers, &url, ContentType::Json, &mut messages);
        let response_body = get_response_body(&url, &mut messages);
        messages.body(&response_body);

        self.verify_json(&response_body, &mut messages);

        Ok(messages)
    }
}
impl Json {
    fn get_wrk_command(&self, url: &str, duration: usize, concurrency: usize) -> Vec<String> {
        vec![
            "wrk",
            "-H",
            "Host: tfb-server",
            "-H",
            "Accept: application/json,text/html;q=0.9,application/xhtml+xml;q=0.9,application/xml;q=0.8,*/*;q=0.7",
            "-H",
            "Connection: keep-alive",
            "--latency",
            "-d",
            &format!("{}", duration),
            "-c",
            &format!("{}", concurrency),
            "--timeout",
            "8",
            "-t",
            &format!("{}", min(concurrency, num_cpus::get())),
            url,
        ].iter().map(|item| item.to_string()).collect()
    }

    fn verify_json(&self, response_body: &str, messages: &mut Messages) {
        if response_body.len() > 27 {
            messages.warning(
                format!(
                    "{} additional response byte(s) found. Consider removing unnecessary whitespace.",
                    (response_body.len() - 27)
                ),
                "Additional response byte(s)"
            );
        }

        match serde_json::from_str::<Value>(&response_body.to_lowercase()) {
            Err(e) => {
                messages.error(format!("Invalid JSON: {:?}", e), "Invalid JSON");
            }
            Ok(json_object) => {
                if json_object["message"].is_null() {
                    messages.error("Missing required key 'message'", "Missing key 'message'");
                } else {
                    if let Some(map) = json_object.as_object() {
                        for entry in map {
                            if entry.0 != "message" {
                                messages.warning(
                                    format!(
                                        "Too many JSON key/value pairs, consider removing: {}",
                                        entry.0
                                    ),
                                    "Too many JSON key/value pairs",
                                );
                            }
                        }
                    }
                    if let Some(str) = json_object["message"].as_str() {
                        if str != "hello, world!" {
                            messages.error(
                                format!("Expected message of 'hello, world!', got '{}'", str),
                                "Invalid response body",
                            );
                        }
                    } else {
                        messages.error(
                            format!(
                                "Expected message of 'hello, world!', got '{}'",
                                json_object["message"].to_string()
                            ),
                            "Invalid response body",
                        )
                    }
                }
            }
        };
    }
}

//
// TESTS
//

#[cfg(test)]
mod tests {
    use crate::test_type::json::Json;
    use crate::verification::Messages;

    #[test]
    fn it_should_succeed_on_correct_body() {
        let json = Json {
            concurrency_levels: vec![16, 32, 64, 128, 256, 512],
        };
        let mut messages = Messages::default();
        json.verify_json("{\"message\":\"Hello, World!\"}", &mut messages);
        assert!(messages.errors.is_empty());
        assert!(messages.warnings.is_empty());
    }

    #[test]
    fn it_should_error_on_valid_json_but_bad_message() {
        let json = Json {
            concurrency_levels: vec![16, 32, 64, 128, 256, 512],
        };
        let mut messages = Messages::default();
        json.verify_json("{\"message\":{}}", &mut messages);
        let mut found = false;
        for error in messages.errors {
            if error
                .message
                .contains("Expected message of 'hello, world!', got")
            {
                found = true;
                break;
            }
        }
        assert!(found);
    }

    #[test]
    fn it_should_error_on_invalid_json_hello_world_object() {
        let json = Json {
            concurrency_levels: vec![16, 32, 64, 128, 256, 512],
        };
        let mut messages = Messages::default();
        json.verify_json("{\"message\":", &mut messages);
        assert_eq!(messages.errors.len(), 1);
        assert!(messages
            .errors
            .get(0)
            .unwrap()
            .message
            .contains("Invalid JSON"));
    }

    #[test]
    fn it_should_warn_on_additional_keys() {
        let json = Json {
            concurrency_levels: vec![16, 32, 64, 128, 256, 512],
        };
        let mut messages = Messages::default();
        json.verify_json(
            "{\"message\":\"Hello, World!\",\"new_key\":\"extra keys are bad\"}",
            &mut messages,
        );
        let mut found = false;
        for warning in messages.warnings {
            if warning.message.contains("Too many JSON key/value pairs") {
                found = true;
                break;
            }
        }
        assert!(found);
    }

    #[test]
    fn it_should_warn_on_additional_bytes() {
        let json = Json {
            concurrency_levels: vec![16, 32, 64, 128, 256, 512],
        };
        let mut messages = Messages::default();
        json.verify_json(
            "{\"message\":\"Hello, World!\",\"new_key\":\"so many bytes!!!\"}",
            &mut messages,
        );
        let mut found = false;
        for warning in messages.warnings {
            if warning
                .message
                .contains("additional response byte(s) found")
            {
                found = true;
                break;
            }
        }
        assert!(found);
    }

    #[test]
    fn it_should_error_on_missing_message_key() {
        let json = Json {
            concurrency_levels: vec![16, 32, 64, 128, 256, 512],
        };
        let mut messages = Messages::default();
        json.verify_json("{\"not_message\":\"Hello, World!\"}", &mut messages);
        assert_eq!(messages.errors.len(), 1);
        assert!(messages
            .errors
            .get(0)
            .unwrap()
            .message
            .contains("Missing required key 'message'"));
    }

    #[test]
    fn it_should_error_on_invalid_hello_world_value() {
        let json = Json {
            concurrency_levels: vec![16, 32, 64, 128, 256, 512],
        };
        let mut messages = Messages::default();
        json.verify_json("{\"message\":\"Hello, Moto!\"}", &mut messages);
        assert_eq!(messages.errors.len(), 1);
        assert!(messages
            .errors
            .get(0)
            .unwrap()
            .message
            .contains("Expected message of 'hello, world!'"));
    }
}
