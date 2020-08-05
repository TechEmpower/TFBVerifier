use crate::benchmark::BenchmarkCommands;
use crate::error::VerifierResult;
use crate::request::{get_response_body, get_response_headers, ContentType};
use crate::test_type::Executor;
use crate::verification::Messages;

pub struct Unknown {
    pub(crate) test_type: String,
}
impl Executor for Unknown {
    fn retrieve_benchmark_commands(&self, _url: &str) -> VerifierResult<BenchmarkCommands> {
        unimplemented!();
    }

    fn verify(&self, url: &str) -> VerifierResult<Messages> {
        let mut messages = Messages::new(url);

        let response_headers = get_response_headers(&url)?;
        messages.headers(&response_headers);
        self.verify_headers(
            &response_headers,
            &url,
            ContentType::Plaintext,
            &mut messages,
        );
        let response_body = get_response_body(&url, &mut messages);
        messages.body(&response_body);

        messages.error(
            &format!("Unknown test type: {}", self.test_type),
            "Unknown Test",
        );

        Ok(messages)
    }
}
