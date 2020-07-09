use crate::database::DatabaseVerifier;
use crate::error::VerifierResult;
use crate::message::Messages;
use crate::request::{get_response_body, get_response_headers, ContentType};
use crate::test_type::Verifier;
use html5ever::tendril::*;
use html5ever::tokenizer::Token::{CharacterTokens, DoctypeToken, TagToken};
use html5ever::tokenizer::{
    BufferQueue, Token, TokenSink, TokenSinkResult, Tokenizer, TokenizerOpts,
};

const FORTUNES: &str = "<!doctype html><html><head><title>Fortunes</title></head><body><table><tr><th>id</th><th>message</th></tr><tr><td>11</td><td>&lt;script&gt;alert(&quot;This should not be displayed in a browser alert box.&quot;);&lt;/script&gt;</td></tr><tr><td>4</td><td>A bad random number generator: 1, 1, 1, 1, 1, 4.33e+67, 1, 1, 1</td></tr><tr><td>5</td><td>A computer program does what you tell it to do, not what you want it to do.</td></tr><tr><td>2</td><td>A computer scientist is someone who fixes things that aren&apos;t broken.</td></tr><tr><td>8</td><td>A list is only as strong as its weakest link. — Donald Knuth</td></tr><tr><td>0</td><td>Additional fortune added at request time.</td></tr><tr><td>3</td><td>After enough decimal places, nobody gives a damn.</td></tr><tr><td>7</td><td>Any program that runs right is obsolete.</td></tr><tr><td>10</td><td>Computers make very fast, very accurate mistakes.</td></tr><tr><td>6</td><td>Emacs is a nice operating system, but I prefer UNIX. — Tom Christaensen</td></tr><tr><td>9</td><td>Feature: A bug with seniority.</td></tr><tr><td>1</td><td>fortune: No such file or directory</td></tr><tr><td>12</td><td>フレームワークのベンチマーク</td></tr></table></body></html>";

pub struct Fortune {
    pub concurrency_levels: Vec<i64>,
    pub database_verifier: Box<dyn DatabaseVerifier>,
}
impl Verifier for Fortune {
    /// Parses the given HTML string and asks the FortuneHTMLParser whether
    /// the parsed string is a valid fortune response.
    fn verify(&self, url: &str) -> VerifierResult<Messages> {
        let mut messages = Messages::new(url);

        // Initialization for query counting
        let repetitions = 2;
        let concurrency = *self.concurrency_levels.iter().max().unwrap();
        let expected_queries = repetitions * concurrency;
        let expected_rows = 12 * expected_queries;

        let response_headers = get_response_headers(&url)?;
        messages.headers(&response_headers);
        self.verify_headers(&response_headers, &url, ContentType::Html, &mut messages);

        let response_body = get_response_body(&url, &mut messages);
        messages.body(&response_body);

        self.verify_fortune(&response_body, &mut messages);
        self.database_verifier.verify_queries_count(
            url,
            "fortune",
            concurrency,
            repetitions,
            expected_queries,
            expected_rows,
        );
        self.verify_fortunes_are_dynamically_sized(&mut messages);

        Ok(messages)
    }
}
impl Fortune {
    /// Returns whether the HTML input parsed by this parser is valid against
    /// our known "fortune" spec. The parsed data in 'body' is joined on empty
    /// strings and checked for equality against our spec.
    fn verify_fortune(&self, response_body: &str, messages: &mut Messages) {
        let mut fortune_accumulator = String::new();
        let sink = FortunesAccumulator {
            accumulator: &mut fortune_accumulator,
        };
        let chunk = ByteTendril::from(response_body.replace('\n', "").replace('\r', "").as_bytes());
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
        assert!(input.is_empty());
        tok.end();

        if fortune_accumulator.to_lowercase() != FORTUNES.to_lowercase() {
            // todo - report a useful diff rather than spitting them out raw.
            messages.error(
                format!(
                    "Invalid fortunes; expected {} but received {}",
                    FORTUNES, fortune_accumulator
                ),
                "Invalid Fortunes",
            );
        }
    }

    /// Checks that test implementations are using dynamically sized data
    /// structures when gathering fortunes from the database.
    ///
    /// In practice, this function will connect to the database and add several
    /// thousand fortunes, request the test implementation for its fortune test
    /// again, and compare to expected output.
    fn verify_fortunes_are_dynamically_sized(&self, _messages: &mut Messages) {
        // todo
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
                    .push_str(&normalize(&String::from_utf8_lossy(b.as_bytes())));
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

/// Normalizes the input string to the format present in the `FORTUNES` const
/// for the purposes of equality checking.
fn normalize(input: &str) -> String {
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
    use crate::message::Messages;
    use crate::test_type::fortune::{normalize, Fortune, FORTUNES};

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
        let mut normalized = normalize("<script>");
        assert_eq!(normalized, good);

        normalized = normalize("&#60;script&#62;");
        assert_eq!(normalized, good);

        normalized = normalize("&#060;script&#062;");
        assert_eq!(normalized, good);

        normalized = normalize("&#x3c;script&#x3e;");
        assert_eq!(normalized, good);
    }

    #[test]
    fn it_should_normalize_apostrophes() {
        let good = "aren&apos;t";
        let mut normalized = normalize("aren't");
        assert_eq!(normalized, good);

        normalized = normalize("aren&#39;t");
        assert_eq!(normalized, good);

        normalized = normalize("aren&#039;t");
        assert_eq!(normalized, good);

        normalized = normalize("aren&#x27;t");
        assert_eq!(normalized, good);
    }

    #[test]
    fn it_should_normalize_quotation_marks() {
        let good = "&quot;This should not be displayed in a browser alert box.&quot;";
        let mut normalized = normalize("\"This should not be displayed in a browser alert box.\"");
        assert_eq!(normalized, good);

        normalized = normalize("&#34;This should not be displayed in a browser alert box.&#34;");
        assert_eq!(normalized, good);

        normalized = normalize("&#034;This should not be displayed in a browser alert box.&#034;");
        assert_eq!(normalized, good);

        normalized = normalize("&#x22;This should not be displayed in a browser alert box.&#x22;");
        assert_eq!(normalized, good);
    }

    #[test]
    fn it_should_normalize_plus_sign() {
        let good = "4.33e+67";
        let mut normalized = normalize("4.33e&#43;67");
        assert_eq!(normalized, good);

        normalized = normalize("4.33e&#043;67");
        assert_eq!(normalized, good);

        normalized = normalize("4.33e&#x2b;67");
        assert_eq!(normalized, good);
    }

    #[test]
    fn it_should_normalize_slash() {
        let good = "/script";
        let mut normalized = normalize("&#47;script");
        assert_eq!(normalized, good);

        normalized = normalize("&#047;script");
        assert_eq!(normalized, good);

        normalized = normalize("&#x2f;script");
        assert_eq!(normalized, good);
    }

    #[test]
    fn it_should_normalize_parens() {
        let good = "()";
        let mut normalized = normalize("&#40;&#41;");
        assert_eq!(normalized, good);

        normalized = normalize("&#040;&#041;");
        assert_eq!(normalized, good);

        normalized = normalize("&#x28;&#x29;");
        assert_eq!(normalized, good);
    }
}
