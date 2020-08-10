use crate::benchmark::BenchmarkCommands;
use crate::error::VerifierResult;
use crate::test_type::Executor;
use crate::verification::Messages;
use std::thread;
use std::time::Duration;

pub struct Unknown {
    pub(crate) test_type: String,
}
impl Executor for Unknown {
    fn retrieve_benchmark_commands(&self, _url: &str) -> VerifierResult<BenchmarkCommands> {
        Ok(BenchmarkCommands::default())
    }

    fn verify(&self, url: &str) -> VerifierResult<Messages> {
        let mut messages = Messages::new(url);

        messages.error(
            &format!("Unknown test type: {}", self.test_type),
            "Unknown Test",
        );

        // We sleep here because we need to ensure that there is enough time
        // for the Toolset to be able to attach and listen to this running
        // container.
        thread::sleep(Duration::from_secs(3));

        Ok(messages)
    }
}
