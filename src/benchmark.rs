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
    pub primer_command: String,
    pub warmup_command: String,
    pub benchmark_commands: Vec<String>,
}

/// Prints and returns a serialized `Benchmark` message.
pub fn send_benchmark_commands(benchmark: BenchmarkCommands) -> String {
    let to_ret = serde_json::to_string(&benchmark).unwrap();
    println!("{}", to_ret);
    to_ret
}

//
// TESTS
//

#[cfg(test)]
mod tests {
    use crate::benchmark::{send_benchmark_commands, BenchmarkCommands};
    use serde_json::Value;

    #[test]
    fn it_can_serialize_benchmark_commands() {
        let benchmark = BenchmarkCommands {
            primer_command: "wrk -H 'Host: 10.0.0.1' -H 'Accept: application/json,text/html;q=0.9,application/xhtml+xml;q=0.9,application/xml;q=0.8,*/*;q=0.7' -H 'Connection: keep-alive' --latency -d 5 -c 8 --timeout 8 -t 8 http://10.0.0.1:8080/json".to_string(),
            warmup_command: "wrk -H 'Host: 10.0.0.1' -H 'Accept: application/json,text/html;q=0.9,application/xhtml+xml;q=0.9,application/xml;q=0.8,*/*;q=0.7' -H 'Connection: keep-alive' --latency -d 15 -c 512 --timeout 8 -t 28 http://10.0.0.1:8080/json".to_string(),
            benchmark_commands: vec![
                "wrk -H 'Host: 10.0.0.1' -H 'Accept: application/json,text/html;q=0.9,application/xhtml+xml;q=0.9,application/xml;q=0.8,*/*;q=0.7' -H 'Connection: keep-alive' --latency -d 15 -c 16 --timeout 8 -t 16 http://10.0.0.1:8080/json".to_string(),
                "wrk -H 'Host: 10.0.0.1' -H 'Accept: application/json,text/html;q=0.9,application/xhtml+xml;q=0.9,application/xml;q=0.8,*/*;q=0.7' -H 'Connection: keep-alive' --latency -d 15 -c 32 --timeout 8 -t 28 http://10.0.0.1:8080/json".to_string(),
                "wrk -H 'Host: 10.0.0.1' -H 'Accept: application/json,text/html;q=0.9,application/xhtml+xml;q=0.9,application/xml;q=0.8,*/*;q=0.7' -H 'Connection: keep-alive' --latency -d 15 -c 64 --timeout 8 -t 28 http://10.0.0.1:8080/json".to_string(),
                "wrk -H 'Host: 10.0.0.1' -H 'Accept: application/json,text/html;q=0.9,application/xhtml+xml;q=0.9,application/xml;q=0.8,*/*;q=0.7' -H 'Connection: keep-alive' --latency -d 15 -c 128 --timeout 8 -t 28 http://10.0.0.1:8080/json".to_string(),
                "wrk -H 'Host: 10.0.0.1' -H 'Accept: application/json,text/html;q=0.9,application/xhtml+xml;q=0.9,application/xml;q=0.8,*/*;q=0.7' -H 'Connection: keep-alive' --latency -d 15 -c 256 --timeout 8 -t 28 http://10.0.0.1:8080/json".to_string(),
                "wrk -H 'Host: 10.0.0.1' -H 'Accept: application/json,text/html;q=0.9,application/xhtml+xml;q=0.9,application/xml;q=0.8,*/*;q=0.7' -H 'Connection: keep-alive' --latency -d 15 -c 512 --timeout 8 -t 28 http://10.0.0.1:8080/json".to_string(),
            ],
        };

        let serialized = send_benchmark_commands(benchmark);
        let json = serde_json::from_str::<Value>(&serialized).unwrap();
        assert!(!json["primer_command"].is_null());
        assert!(!json["warmup_command"].is_null());
        assert!(!json["benchmark_commands"].is_null());
        assert!(json["benchmark_commands"].as_array().is_some());
        assert_eq!(json["benchmark_commands"].as_array().unwrap().len(), 6);
    }
}
