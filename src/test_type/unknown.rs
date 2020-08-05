use crate::error::VerifierResult;
use crate::message::Messages;
use crate::request::{get_response_body, get_response_headers, ContentType};
use crate::test_type::Executor;

pub struct Unknown {
    pub(crate) test_type: String,
}
impl Executor for Unknown {
    fn benchmark(&self, _url: &str) -> VerifierResult<()> {
        // todo

        Ok(())
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
