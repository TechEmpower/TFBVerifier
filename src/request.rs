use crate::error::VerifierError::{CurlError, Non200Response, RequestError};
use crate::error::VerifierResult;
use crate::logger::{log, LogOptions};
use crate::verification::Messages;
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

pub fn request(url: &str) -> VerifierResult<Vec<u8>> {
    let mut easy = Easy2::new(Collector(Vec::new()));
    easy.url(url)?;
    easy.perform()?;

    match easy.response_code() {
        Ok(200) => Ok(easy.get_ref().0.clone()),
        Ok(code) => Err(Non200Response(url.to_string(), code)),
        Err(e) => Err(RequestError(url.to_string(), e.to_string())),
    }
}

pub fn get_response_body(url: &str, messages: &mut Messages) -> Option<String> {
    log(
        format!("Accessing URL {}", url).cyan(),
        LogOptions {
            border: None,
            border_bottom: None,
            quiet: false,
        },
    );

    match request(url) {
        Ok(bytes) => Some(String::from_utf8_lossy(&*bytes).to_string()),
        Err(e) => match e {
            Non200Response(url, code) => {
                messages.error(
                    format!("Non-200 response from {}: {}", url, code),
                    "Non-200 response",
                );
                None
            }
            RequestError(url, err_string) => {
                messages.error(
                    format!("Error requesting {}: {}", url, err_string),
                    "Request error",
                );
                None
            }
            _ => {
                messages.error(
                    format!("Unknown error requesting {}: {:?}", url, e),
                    "Unknown error",
                );
                None
            }
        },
    }
}

pub fn get_response_headers(
    url: &str,
    messages: &mut Messages,
) -> VerifierResult<HashMap<String, String>> {
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
        match transfer.perform() {
            Ok(_) => {}
            Err(e) => {
                messages.error(
                    format!("Error requesting headers for url: {}, {:?}", url, e),
                    "Header(s) Error",
                );
                return Err(CurlError(e));
            }
        };
    }
    for header in header_vec {
        let split: Vec<&str> = header.split(":").collect();
        if split.len() >= 2 {
            let key = split.get(0).unwrap().trim().to_string().clone();
            let value = split[1..].join(":").trim().to_string().clone();
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
    use crate::verification::Messages;

    #[test]
    fn what_headers() {
        let url = "http://www.google.com";
        let mut messages = Messages::new(url);
        let serialized = get_response_headers(url, &mut messages).unwrap();

        for header in serialized {
            if header.0 == "Vary" {
                assert_eq!(header.1, "Accept-Encoding".to_string());
            }
        }
    }
}
