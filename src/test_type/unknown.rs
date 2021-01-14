use crate::benchmark::BenchmarkCommands;
use crate::database::DatabaseInterface;
use crate::error::VerifierResult;
use crate::test_type::Executor;
use crate::verification::Messages;
use std::thread;
use std::time::Duration;

pub struct Unknown {
    pub(crate) test_type: String,
    pub database_verifier: Box<dyn DatabaseInterface>,
}
impl Executor for Unknown {
    fn wait_for_database_to_be_available(&self) {
        self.database_verifier.wait_for_database_to_be_available();
    }

    fn retrieve_benchmark_commands(&self, _url: &str) -> VerifierResult<BenchmarkCommands> {
        Ok(BenchmarkCommands::default())
    }

    fn verify(&self, url: &str) -> VerifierResult<Messages> {
        let mut messages = Messages::new(url);

        messages.error(
            &format!("Unknown test type: {}", self.test_type),
            "Unknown Test",
        );

        // TODO - remove this... no longer necessary.
        // We sleep here because we need to ensure that there is enough time
        // for the Toolset to be able to attach and listen to this running
        // container.
        thread::sleep(Duration::from_secs(3));

        Ok(messages)
    }
}
