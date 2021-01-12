use crate::benchmark::BenchmarkCommands;
use crate::error::VerifierResult;
use crate::request::{get_response_body, get_response_headers, ContentType};
use crate::test_type::Executor;
use crate::verification::Messages;
use std::cmp::min;

pub struct Plaintext {
    pub pipeline_concurrency_levels: Vec<u32>,
}
impl Executor for Plaintext {
    fn retrieve_benchmark_commands(&self, url: &str) -> VerifierResult<BenchmarkCommands> {
        let primer_command = self.get_wrk_command(url, 5, 8);
        let warmup_command = self.get_wrk_command(
            url,
            15,
            *self.pipeline_concurrency_levels.iter().max().unwrap(),
        );
        let mut benchmark_commands = Vec::default();
        for concurrency in &self.pipeline_concurrency_levels {
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

        let response_headers = get_response_headers(&url, &mut messages)?;
        messages.headers(&response_headers);
        self.verify_headers(
            &response_headers,
            &url,
            ContentType::Plaintext,
            &mut messages,
        );
        if let Some(response_body) = get_response_body(&url, &mut messages) {
            messages.body(&response_body);

            self.verify_plaintext(&response_body, &mut messages);
        }

        Ok(messages)
    }
}
impl Plaintext {
    fn verify_plaintext(&self, response_body: &str, messages: &mut Messages) {
        let body = response_body.to_lowercase();
        let expected = "hello, world!";
        let extra_bytes = body.len() - expected.len();

        if !body.contains(expected) {
            messages.error(
                format!("Could not find 'Hello, World!' in response: '{}'", body),
                "Invalid response body",
            );
        }

        if extra_bytes > 0 {
            messages.warning(
                format!("Server is returning {} more bytes than are required. This may negatively affect benchmark performance.", extra_bytes),
                "Additional response byte(s)"
            );
        }
    }

    fn get_wrk_command(&self, url: &str, duration: u32, concurrency: u32) -> Vec<String> {
        vec![
            "wrk",
            "-H",
            "Host: tfb-server",
            "-H",
            "Accept: text/plain,text/html;q=0.9,application/xhtml+xml;q=0.9,application/xml;q=0.8,*/*;q=0.7",
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
            &format!("{}", min(concurrency, num_cpus::get() as u32)),
            url,
            "-s",
            "pipeline.lua",
            "--",
            "16",
        ].iter().map(|item| item.to_string()).collect()
    }
}

//
// TESTS
//

#[cfg(test)]
mod tests {
    use crate::test_type::plaintext::Plaintext;
    use crate::verification::Messages;

    #[test]
    fn it_should_succeed_on_correct_body() {
        let plaintext = Plaintext {
            pipeline_concurrency_levels: vec![256, 1024, 4096, 16384],
        };
        let mut messages = Messages::default();
        plaintext.verify_plaintext("Hello, World!", &mut messages);
        assert!(messages.errors.is_empty());
        assert!(messages.warnings.is_empty());
    }

    #[test]
    fn it_should_fail_on_incorrect_message() {
        let plaintext = Plaintext {
            pipeline_concurrency_levels: vec![256, 1024, 4096, 16384],
        };
        let mut messages = Messages::default();
        plaintext.verify_plaintext("World, Hello!", &mut messages);
        let mut found = false;
        for error in messages.errors {
            if error
                .message
                .contains("Could not find 'Hello, World!' in response")
            {
                found = true;
                break;
            }
        }
        assert!(found);
    }
}
