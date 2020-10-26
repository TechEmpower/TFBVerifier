use crate::benchmark::BenchmarkCommands;
use crate::database::DatabaseInterface;
use crate::error::VerifierResult;
use crate::request::{get_response_body, get_response_headers, ContentType};
use crate::test_type::Executor;
use crate::verification::Messages;
use html5ever::tendril::*;
use html5ever::tokenizer::Token::{CharacterTokens, DoctypeToken, TagToken};
use html5ever::tokenizer::{
    BufferQueue, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts,
};
use std::cmp::min;

const FORTUNES: &str = "<!doctype html><html><head><title>Fortunes</title></head><body><table><tr><th>id</th><th>message</th></tr><tr><td>11</td><td>&lt;script&gt;alert(&quot;This should not be displayed in a browser alert box.&quot;);&lt;/script&gt;</td></tr><tr><td>4</td><td>A bad random number generator: 1, 1, 1, 1, 1, 4.33e+67, 1, 1, 1</td></tr><tr><td>5</td><td>A computer program does what you tell it to do, not what you want it to do.</td></tr><tr><td>2</td><td>A computer scientist is someone who fixes things that aren&apos;t broken.</td></tr><tr><td>8</td><td>A list is only as strong as its weakest link. — Donald Knuth</td></tr><tr><td>0</td><td>Additional fortune added at request time.</td></tr><tr><td>3</td><td>After enough decimal places, nobody gives a damn.</td></tr><tr><td>7</td><td>Any program that runs right is obsolete.</td></tr><tr><td>10</td><td>Computers make very fast, very accurate mistakes.</td></tr><tr><td>6</td><td>Emacs is a nice operating system, but I prefer UNIX. — Tom Christaensen</td></tr><tr><td>9</td><td>Feature: A bug with seniority.</td></tr><tr><td>1</td><td>fortune: No such file or directory</td></tr><tr><td>12</td><td>フレームワークのベンチマーク</td></tr></table></body></html>";

pub struct Fortune {
    pub concurrency_levels: Vec<u32>,
    pub database_verifier: Box<dyn DatabaseInterface>,
}
impl Executor for Fortune {
    fn retrieve_benchmark_commands(&self, url: &str) -> VerifierResult<BenchmarkCommands> {
        let primer_command = self.get_wrk_command(url, 5, 8);
        let warmup_command =
            self.get_wrk_command(url, 15, *self.concurrency_levels.iter().max().unwrap());
        let mut benchmark_commands = Vec::default();
        for concurrency in &self.concurrency_levels {
            benchmark_commands.push(self.get_wrk_command(url, 15, *concurrency));
        }

        Ok(BenchmarkCommands {
            primer_command,
            warmup_command,
            benchmark_commands,
        })
    }

    /// Parses the given HTML string and asks the FortuneHTMLParser whether
    /// the parsed string is a valid fortune response.
    fn verify(&self, url: &str) -> VerifierResult<Messages> {
        let mut messages = Messages::new(url);

        // Initialization for query counting
        let repetitions = 2;
        let concurrency = *self.concurrency_levels.iter().max().unwrap();
        let expected_queries = repetitions * concurrency;
        let expected_rows = 12 * expected_queries;

        let response_headers = get_response_headers(&url, &mut messages)?;
        messages.headers(&response_headers);
        self.verify_headers(&response_headers, &url, ContentType::Html, &mut messages);

        let response_body = get_response_body(&url, &mut messages);
        let mut accumulator = String::new();
        for line in response_body.lines() {
            accumulator.push_str(line);
        }
        messages.body(&accumulator);

        let verified = self.verify_fortune(&response_body, &mut messages);
        self.database_verifier.verify_queries_count(
            url,
            "fortune",
            concurrency,
            repetitions,
            expected_queries,
            &mut messages,
        );
        self.database_verifier.verify_rows_count(
            url,
            "fortune",
            concurrency,
            repetitions,
            expected_rows,
            12,
            &mut messages,
        );
        if verified {
            self.verify_fortunes_are_dynamically_sized(&url, &mut messages);
        }
        // Note: we call this again because internally `verify_fortunes_are...`
        // will set the body to its extra-large variant and we don't want to
        // output that.
        messages.body(&response_body);

        Ok(messages)
    }
}
impl Fortune {
    fn get_wrk_command(&self, url: &str, duration: u32, concurrency: u32) -> Vec<String> {
        vec![
            "wrk",
            "-H",
            "Host: tfb-server",
            "-H",
            "Accept: application/json,text/html;q=0.9,application/xhtml+xml;q=0.9,application/xml;q=0.8,*/*;q=0.7",
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
        ].iter().map(|item| item.to_string()).collect()
    }

    /// Returns whether the HTML input parsed by this parser is valid against
    /// our known "fortune" spec.
    fn verify_fortune(&self, response_body: &str, messages: &mut Messages) -> bool {
        let fortunes = normalize_html(response_body);

        if fortunes.to_lowercase() != FORTUNES.to_lowercase() {
            // todo - report a useful diff rather than spitting them out raw.
            messages.error(
                format!(
                    "Invalid fortunes; expected {} but received {}",
                    FORTUNES, fortunes
                ),
                "Invalid Fortunes",
            );
            false
        } else {
            true
        }
    }

    /// Checks that test implementations are using dynamically sized data
    /// structures when gathering fortunes from the database.
    ///
    /// In practice, this function will connect to the database and add one
    /// thousand fortunes, request the test implementation for its fortune test
    /// again, and compare to expected output.
    ///
    /// Note: this function presupposes that `verify_fortune` was called prior
    /// to this call and that it succeeded. The assumption is that if that
    /// holds true, then simply adding more `fortune`s to the underlying
    /// database and requesting the same `url` will cause a dynamically
    /// generated HTML response with those extra `fortune`s, so that equality
    /// checking of the output (in the same way as `verify_fortune`) will still
    /// hold true.
    fn verify_fortunes_are_dynamically_sized(&self, url: &str, messages: &mut Messages) {
        // Future improvement - generate random `message` columns, query the
        // database for the fortune table (now with 1,000 more random rows),
        // and create our view here. We can then check string equality with
        // the test's fortune implementation.
        self.database_verifier.insert_one_thousand_fortunes();
        let mut more_fortunes = String::from("<!doctype html><html><head><title>Fortunes</title></head><body><table><tr><th>id</th><th>message</th></tr><tr><td>11</td><td>&lt;script&gt;alert(&quot;This should not be displayed in a browser alert box.&quot;);&lt;/script&gt;</td></tr><tr><td>4</td><td>A bad random number generator: 1, 1, 1, 1, 1, 4.33e+67, 1, 1, 1</td></tr><tr><td>5</td><td>A computer program does what you tell it to do, not what you want it to do.</td></tr><tr><td>2</td><td>A computer scientist is someone who fixes things that aren&apos;t broken.</td></tr><tr><td>8</td><td>A list is only as strong as its weakest link. — Donald Knuth</td></tr><tr><td>0</td><td>Additional fortune added at request time.</td></tr><tr><td>3</td><td>After enough decimal places, nobody gives a damn.</td></tr><tr><td>7</td><td>Any program that runs right is obsolete.</td></tr><tr><td>10</td><td>Computers make very fast, very accurate mistakes.</td></tr><tr><td>6</td><td>Emacs is a nice operating system, but I prefer UNIX. — Tom Christaensen</td></tr><tr><td>9</td><td>Feature: A bug with seniority.</td></tr><tr><td>1</td><td>fortune: No such file or directory</td></tr><tr><td>12</td><td>フレームワークのベンチマーク</td></tr>");
        for i in 0..1_000 {
            more_fortunes.push_str(&format!(
                "<tr><td>{}</td><td>フレームワークのベンチマーク</td></tr>",
                i + 13
            ));
        }
        more_fortunes.push_str("</table></body></html>");

        let response_body = get_response_body(&url, messages);
        let mut accumulator = String::new();
        for line in response_body.lines() {
            accumulator.push_str(line);
        }
        // truncate the single-line for rendering
        accumulator = accumulator[..500].to_string();
        accumulator.push_str("...");
        messages.body(&accumulator);

        let fortunes = normalize_html(&response_body);

        // We explicitly *do not* check that the strings are equal here because
        // of how different implementations will order equal strings. E.g. we
        // added a bunch of copies of the last fortune above, and we order by
        // that column - it is valid to put them in any order because they are
        // all equal. Instead, after normalizing both, we check that we have
        // the same character count.
        if fortunes.chars().count() != more_fortunes.chars().count() {
            messages.error(
                format!(
                    "Fortunes not dynamically sized. Expected length: {}; actual length: {}",
                    more_fortunes.len(),
                    fortunes.len()
                ),
                "Non-dynamic Fortune",
            );
        }
    }
}

struct FortunesAccumulator<'accum> {
    accumulator: &'accum mut String,
}
impl<'accum> TokenSink for FortunesAccumulator<'accum> {
    type Handle = ();
    fn process_token(&mut self, token: Token, _line_number: u64) -> TokenSinkResult<()> {
        match token {
            DoctypeToken(doctype) => {
                if let Some(name) = &doctype.name {
                    self.accumulator.push_str(&format!("<!doctype {}>", name));
                }
            }
            CharacterTokens(b) => {
                self.accumulator
                    .push_str(&normalize_text(&String::from_utf8_lossy(b.as_bytes())));
            }
            TagToken(tag) => match tag.kind {
                html5ever::tokenizer::StartTag => {
                    self.accumulator.push_str(&format!("<{}>", tag.name));
                }
                html5ever::tokenizer::EndTag => {
                    self.accumulator.push_str(&format!("</{}>", tag.name));
                }
            },
            _ => {}
        }
        TokenSinkResult::Continue
    }
}

//
// PRIVATES
//

/// Normalizes the input HTML to the format present in the `FORTUNES` const.
fn normalize_html(input: &str) -> String {
    let mut fortune_accumulator = String::new();
    let sink = FortunesAccumulator {
        accumulator: &mut fortune_accumulator,
    };
    let chunk = ByteTendril::from(input.replace('\n', "").replace('\r', "").as_bytes());
    let mut input = BufferQueue::new();
    input.push_back(chunk.try_reinterpret().unwrap());

    let mut tok = Tokenizer::new(
        sink,
        TokenizerOpts {
            profile: false,
            ..Default::default()
        },
    );
    let _ = tok.feed(&mut input);
    tok.end();

    fortune_accumulator
}

/// Normalizes the input string to the format present in the `FORTUNES` const
/// for the purposes of equality checking.
fn normalize_text(input: &str) -> String {
    input
        // After a LOT of debate, these are now considered valid in data.
        // The reason for this approach is because a few tests use tools
        // which determine at compile time whether or not a string needs
        // a given type of html escaping, and our fortune test has
        // apostrophes and quotes in html data rather than as an html
        // attribute etc.
        // example:
        // <td>
        //   A computer scientist is someone who fixes things that aren't
        //   broken.
        // </td>
        // Semantically, that apostrophe does not NEED to be escaped. The
        // same is currently true for our quotes.
        // In fact, in data (read: between two html tags) even the '>' need
        // not be replaced as long as the '<' are all escaped. We replace
        // them with their escapings here in order to have a normalized
        // string for equality comparison at the end.
        .replace("'", "&apos;")
        .replace("\"", "&quot;")
        .replace(">", "&gt;")
        .replace("<", "&lt;")
        // `&#34;` is a valid escaping, but we are normalizing it so that
        // our final parse can just be checked for equality.
        .replace("&#34;", "&quot;")
        .replace("&#034;", "&quot;")
        .replace("&#x22;", "&quot;")
        // `&#39;` is a valid escaping of `'`, but it is not required, so
        // we normalize for equality checking.
        .replace("&#39;", "&apos;")
        .replace("&#039;", "&apos;")
        .replace("&#x27;", "&apos;")
        // Again, `&#43;` is a valid escaping of the `+`, but it is not
        // required, so we need to normalize for out final parse and
        // equality check.
        .replace("&#43;", "+")
        .replace("&#043;", "+")
        .replace("&#x2b;", "+")
        // Again, `&#62;` is a valid escaping of `>`, but we need to
        // normalize to "&gt;" for equality checking.
        .replace("&#62;", "&gt;")
        .replace("&#062;", "&gt;")
        .replace("&#x3e;", "&gt;")
        // Again, `&#60;` is a valid escaping of `<`, but we need to
        // normalize to `&lt;` for equality checking.
        .replace("&#60;", "&lt;")
        .replace("&#060;", "&lt;")
        .replace("&#x3c;", "&lt;")
        // Not sure why some are escaping `/`
        .replace("&#47;", "/")
        .replace("&#047;", "/")
        .replace("&#x2f;", "/")
        // "&#40;" is a valid escaping of "(", but it is not required, so
        // we need to normalize for out final parse and equality check.
        .replace("&#40;", "(")
        .replace("&#040;", "(")
        .replace("&#x28;", "(")
        // "&#41;" is a valid escaping of ")", but it is not required, so
        // we need to normalize for out final parse and equality check.
        .replace("&#41;", ")")
        .replace("&#041;", ")")
        .replace("&#x29;", ")")
}

//
// TESTS
//

#[cfg(test)]
mod tests {
    use crate::database::mysql::Mysql;
    use crate::test_type::fortune::{normalize_text, Fortune, FORTUNES};
    use crate::verification::Messages;

    #[test]
    fn it_should_pass_with_identity_fortunes() {
        let mut messages = Messages::default();
        let valid = FORTUNES;
        let fortune = Fortune {
            concurrency_levels: vec![16, 32, 64, 128, 256, 512],
            database_verifier: Box::new(Mysql {}),
        };

        fortune.verify_fortune(valid, &mut messages);
    }

    #[test]
    fn it_should_normalize_lt_and_gt() {
        let good = "&lt;script&gt;";
        let mut normalized = normalize_text("<script>");
        assert_eq!(normalized, good);

        normalized = normalize_text("&#60;script&#62;");
        assert_eq!(normalized, good);

        normalized = normalize_text("&#060;script&#062;");
        assert_eq!(normalized, good);

        normalized = normalize_text("&#x3c;script&#x3e;");
        assert_eq!(normalized, good);
    }

    #[test]
    fn it_should_normalize_apostrophes() {
        let good = "aren&apos;t";
        let mut normalized = normalize_text("aren't");
        assert_eq!(normalized, good);

        normalized = normalize_text("aren&#39;t");
        assert_eq!(normalized, good);

        normalized = normalize_text("aren&#039;t");
        assert_eq!(normalized, good);

        normalized = normalize_text("aren&#x27;t");
        assert_eq!(normalized, good);
    }

    #[test]
    fn it_should_normalize_quotation_marks() {
        let good = "&quot;This should not be displayed in a browser alert box.&quot;";
        let mut normalized =
            normalize_text("\"This should not be displayed in a browser alert box.\"");
        assert_eq!(normalized, good);

        normalized =
            normalize_text("&#34;This should not be displayed in a browser alert box.&#34;");
        assert_eq!(normalized, good);

        normalized =
            normalize_text("&#034;This should not be displayed in a browser alert box.&#034;");
        assert_eq!(normalized, good);

        normalized =
            normalize_text("&#x22;This should not be displayed in a browser alert box.&#x22;");
        assert_eq!(normalized, good);
    }

    #[test]
    fn it_should_normalize_plus_sign() {
        let good = "4.33e+67";
        let mut normalized = normalize_text("4.33e&#43;67");
        assert_eq!(normalized, good);

        normalized = normalize_text("4.33e&#043;67");
        assert_eq!(normalized, good);

        normalized = normalize_text("4.33e&#x2b;67");
        assert_eq!(normalized, good);
    }

    #[test]
    fn it_should_normalize_slash() {
        let good = "/script";
        let mut normalized = normalize_text("&#47;script");
        assert_eq!(normalized, good);

        normalized = normalize_text("&#047;script");
        assert_eq!(normalized, good);

        normalized = normalize_text("&#x2f;script");
        assert_eq!(normalized, good);
    }

    #[test]
    fn it_should_normalize_parens() {
        let good = "()";
        let mut normalized = normalize_text("&#40;&#41;");
        assert_eq!(normalized, good);

        normalized = normalize_text("&#040;&#041;");
        assert_eq!(normalized, good);

        normalized = normalize_text("&#x28;&#x29;");
        assert_eq!(normalized, good);
    }
}
