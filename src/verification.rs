//! The `Verification` module is the mechanism by which `TFBVerifier` communicates
//! verification data to the `TFBToolset`.
//! By default, nearly anything printing to stdout/stderr will simply be
//! consumed by the toolset and then printed to stdout/stderr. However, in
//! order to pass data about the state of the verification, we serialize
//! messages specifically for the consumption by the toolset that will not be
//! printed.
use colored::Colorize;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Warning {
    pub body: String,
    pub url: String,
    pub headers: String,
    pub message: String,
}
#[derive(Clone)]
pub struct Error {
    pub body: String,
    pub url: String,
    pub headers: String,
    pub message: String,
}

/// The mechanism for message interfacing with the calling `TFBToolset`. Every
/// `verify` implementation should instantiate a new `Messages` object for any
/// `url` and set the `body` and `headers` after a successful request is made.
/// These values are the context for any verification errors and warnings.
///
/// After setting the context, callers should invoke the convenience functions
/// `warning` and `error` to communicate with the calling `TFBToolset` that
/// such events occurred.
#[derive(Clone)]
pub struct Messages {
    pub warnings: Vec<Warning>,
    pub errors: Vec<Error>,
    url: String,
    body: String,
    headers: String,
}
impl Messages {
    pub fn default() -> Self {
        Self {
            warnings: Vec::new(),
            errors: Vec::new(),
            url: "".to_string(),
            body: "".to_string(),
            headers: "".to_string(),
        }
    }

    pub fn new(url: &str) -> Self {
        Self {
            warnings: Vec::new(),
            errors: Vec::new(),
            url: url.to_string(),
            body: "".to_string(),
            headers: "".to_string(),
        }
    }

    pub fn body(&mut self, body: &str) {
        self.body = body.to_string();
    }

    pub fn headers(&mut self, headers: &HashMap<String, String>) {
        self.headers = get_headers_as_string(headers);
    }

    /// Captures and sends an error message.
    pub fn error<T, F>(&mut self, message: T, short_message: F)
    where
        T: std::fmt::Display,
        F: std::fmt::Display,
    {
        send_error(&message, &short_message);

        let error = Error {
            url: self.url.clone(),
            body: self.body.clone(),
            headers: self.headers.clone(),
            message: message.to_string(),
        };

        self.errors.push(error);
    }

    /// Captures and sends a warning message.
    pub fn warning<T, F>(&mut self, message: T, short_message: F)
    where
        T: std::fmt::Display,
        F: std::fmt::Display,
    {
        send_warning(&message, &short_message);

        let warning = Warning {
            body: self.body.clone(),
            url: self.url.clone(),
            headers: self.headers.clone(),
            message: message.to_string(),
        };
        self.warnings.push(warning);
    }

    /// Prints out the results and if there are no errors, sends the passed message.
    pub fn output_verification_results(&self) {
        if self.errors.is_empty() && self.warnings.is_empty() {
            println!("   {}", "PASS".green());
        }
        if !self.warnings.is_empty() {
            println!("   {}", "WARN".yellow());
            for warning in &self.warnings {
                println!("     {}", warning.message);
                println!("     See https://github.com/TechEmpower/FrameworkBenchmarks/wiki/Project-Information-Framework-Tests-Overview#specific-test-requirements");
                if !warning.url.is_empty() {
                    println!("{}", warning.url);
                }
                if !warning.headers.is_empty() {
                    println!("{}", warning.headers);
                }
                if !warning.body.is_empty() {
                    println!("{}", warning.body);
                }
            }
        }
        if !self.errors.is_empty() {
            println!("   {}", "ERROR".red());
            for error in &self.errors {
                println!("     {}", error.message);
                println!("     See https://github.com/TechEmpower/FrameworkBenchmarks/wiki/Project-Information-Framework-Tests-Overview#specific-test-requirements");
                if !error.url.is_empty() {
                    println!("{}", error.url);
                }
                if !error.headers.is_empty() {
                    println!("{}", error.headers);
                }
                if !error.body.is_empty() {
                    println!("{}", error.body);
                }
            }
        }
    }
}

//
// PRIVATES
//

fn get_headers_as_string(headers: &HashMap<String, String>) -> String {
    let mut header_str = String::new();
    for entry in headers {
        header_str.push_str(&format!("'{}':'{}', ", entry.0, entry.1));
    }
    header_str.pop();
    header_str.pop();
    format!("{}{}{}", '{', header_str, '}')
}

/// Prints and returns a serialized `warning` message.
fn send_warning<T, F>(message: T, short_message: F) -> String
where
    T: std::fmt::Display,
    F: std::fmt::Display,
{
    let mut map = HashMap::new();
    let mut messages = HashMap::new();
    messages.insert("message", message.to_string());
    messages.insert("short_message", short_message.to_string());
    map.insert("warning", messages);
    let to_ret = serde_json::to_string(&map).unwrap();
    println!("{}", to_ret);
    to_ret
}

/// Prints and returns a serialized `error` message.
fn send_error<T, F>(message: T, short_message: F) -> String
where
    T: std::fmt::Display,
    F: std::fmt::Display,
{
    let mut map = HashMap::new();
    let mut messages = HashMap::new();
    messages.insert("message", message.to_string());
    messages.insert("short_message", short_message.to_string());
    map.insert("error", messages);
    let to_ret = serde_json::to_string(&map).unwrap();
    println!("{}", to_ret);
    to_ret
}

//
// TESTS
//

#[cfg(test)]
mod tests {
    use crate::verification::{send_error, send_warning};
    use serde_json::Value;

    #[test]
    fn it_can_serialize_a_warning_verification() {
        let serialized = send_warning("Returning too many bytes", "Too many bytes");
        let json = serde_json::from_str::<Value>(&serialized).unwrap();
        assert!(!json["warning"].is_null());
        assert_eq!(json["warning"]["message"], "Returning too many bytes");
        assert_eq!(json["warning"]["short_message"], "Too many bytes");
    }

    #[test]
    fn it_can_serialize_an_error_verification() {
        let serialized = send_error("Incorrect response body", "Incorrect response");
        let json = serde_json::from_str::<Value>(&serialized).unwrap();
        assert!(!json["error"].is_null());
        assert_eq!(json["error"]["message"], "Incorrect response body");
        assert_eq!(json["error"]["short_message"], "Incorrect response");
    }
}
