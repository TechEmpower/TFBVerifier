//! The `benchmark` module is the mechanism by which `TFBVerifier` communicates
//! benchmark instructions to the `TFBToolset`.
//! By default, nearly anything printing to stdout/stderr will simply be
//! consumed by the toolset and then printed to stdout/stderr. However, in
//! order to pass data about *how* to benchmark any `TestType`, we serialize
//! messages specifically for the consumption by the toolset that will not be
//! printed.

use serde::Serialize;

/// A `Benchmark` is used for describing how to invoke a benchmarker for the
/// given `TestType`
/// Note: no actual benchmark*ing* occurs here - rather, the `Executor`
/// implementations for each `TestType` will produce a `Benchmark` for
/// consumption by the `TFBToolset`.
#[derive(Serialize, Clone, Debug)]
pub struct BenchmarkCommands {
    pub primer_command: Vec<String>,
    pub warmup_command: Vec<String>,
    pub benchmark_commands: Vec<Vec<String>>,
}
impl Default for BenchmarkCommands {
    fn default() -> Self {
        Self {
            primer_command: Vec::default(),
            warmup_command: Vec::default(),
            benchmark_commands: Vec::default(),
        }
    }
}

/// Prints and returns a serialized `Benchmark` message.
pub fn send_benchmark_commands(benchmark: BenchmarkCommands) -> String {
    let to_ret = serde_json::to_string(&benchmark).unwrap();
    println!("{}", to_ret);
    to_ret
}
