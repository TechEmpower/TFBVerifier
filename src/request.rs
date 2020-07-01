use crate::error::VerifierResult;
use crate::logger::{log, LogOptions};
use crate::message::Messages;
use colored::Colorize;
use curl::easy::{Easy, Easy2, Handler, WriteError};
use std::collections::HashMap;

pub enum ContentType {
    Json,
    Plaintext,
    Html,
}

struct Collector(Vec<u8>);

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}

pub fn get_response_body(url: &str, headers: &HashMap<String, String>) -> VerifierResult<String> {
    log(
        format!("Accessing URL {}", url).cyan(),
        LogOptions {
            border: None,
            border_bottom: None,
            quiet: false,
        },
    );
    let mut easy = Easy2::new(Collector(Vec::new()));
    easy.url(url)?;
    easy.perform()?;

    let mut messages = Messages::new(url);
    messages.headers(headers);
    match easy.response_code() {
        Ok(200) => {}
        Ok(code) => messages.error(
            format!("Non-200 response from {}: {}", url, code),
            "Non-200 response",
        ),
        Err(e) => messages.error(
            format!("Error requesting {}: {}", url, e.to_string()),
            "Request error",
        ),
    };

    Ok(String::from_utf8_lossy(&easy.get_ref().0).to_string())
}

pub fn get_response_headers(url: &str) -> VerifierResult<HashMap<String, String>> {
    let mut headers = HashMap::new();
    let mut handle = Easy::new();
    handle.url(url).unwrap();

    let mut header_vec = Vec::new();
    {
        let mut transfer = handle.transfer();
        transfer
            .header_function(|header| {
                header_vec.push(String::from_utf8_lossy(header).to_string());
                true
            })
            .unwrap();
        transfer.perform().unwrap();
    }
    for header in header_vec {
        let split: Vec<&str> = header.split(": ").collect();
        if split.len() == 2 {
            let key = split.get(0).unwrap().trim().to_string().clone();
            let value = split.get(1).unwrap().trim().to_string().clone();
            headers.insert(key, value);
        }
    }

    Ok(headers)
}

//
// TESTS
//

#[cfg(test)]
mod tests {
    use crate::request::get_response_headers;

    #[test]
    fn what_headers() {
        let serialized = get_response_headers(&"http://www.google.com".to_string()).unwrap();

        for header in serialized {
            if header.0 == "Vary" {
                assert_eq!(header.1, "Accept-Encoding".to_string());
            }
        }
    }
}
