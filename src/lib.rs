use anyhow::{Result, anyhow};
use logos::{Lexer, Logos};
use std::io::Write;

/// Token scheme designed for efficiently copying input to formatted output.
/// We only really care about object markers and spans of bytes.
///
/// For numbers and strings, regex is from logos handbook.
#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\r\n]+")]
enum Token {
    // object markers
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,

    // values
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("null")]
    Null,
    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?")]
    Number,
    #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#)]
    String,
}

struct Scanner<'a, 'b, W: Write> {
    lexer: Lexer<'a, Token>,
    writer: &'b W,
    fields: Vec<&'a str>,
    stack: Vec<char>,
}

impl<'a, 'b, W: Write> Scanner<'a, 'b, W> {
    fn take(self: &mut Self, token: Token) -> Result<Token> {
        match self.lexer.next() {
            Some(Ok(t)) if t == token => Ok(token),
            Some(Ok(t)) => Err(anyhow!("expected {:?}, found {:?}", token, t)),
            Some(Err(_)) => Err(anyhow!("invalid token, expected {:?}", token)),
            None => Err(anyhow!("premature eof, expected {:?}", token)),
        }
    }

    fn seek_key(self: &mut Self, _key: &str) -> Result<()> {
        todo!()
    }

    fn seek_end_of_object(self: &mut Self) -> Result<()> {
        todo!()
    }

    fn scan_schema(self: &mut Self) -> Result<()> {
        todo!()
    }

    fn scan_datarows(self: &mut Self) -> Result<()> {
        todo!()
    }

    fn scan_jdbc(self: &mut Self) -> Result<()> {
        self.take(Token::LeftBrace)?;
        self.seek_key("schema")?;
        self.scan_schema()?;
        self.seek_key("datarows")?;
        self.scan_datarows()?;
        self.seek_end_of_object()?;
        Ok(())
    }
}

pub fn convert_jdbc<W: Write>(input_json: &str, writer: &mut W) -> Result<()> {
    let lexer = Token::lexer(input_json);
    let mut scanner = Scanner {
        lexer,
        writer,
        fields: Vec::new(),
        stack: Vec::new(),
    };
    scanner.scan_jdbc()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sonic_rs::{JsonContainerTrait, JsonValueTrait, Value};

    fn parse_output_lines(output: &[u8]) -> Vec<Value> {
        String::from_utf8(output.to_vec())
            .unwrap()
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| sonic_rs::from_str(line).unwrap())
            .collect()
    }

    #[test]
    fn test_messages_sample() {
        let input = include_str!("../samples/messages.json");
        let mut output = Vec::new();

        convert_jdbc(&input, &mut output).unwrap();

        let records = parse_output_lines(&output);
        assert_eq!(records.len(), 10);

        let first = records[0].as_object().unwrap();
        assert_eq!(
            first.get(&"@timestamp").unwrap().as_str().unwrap(),
            "2023-01-01 00:35:22"
        );
        assert!(
            first
                .get(&"message")
                .unwrap()
                .as_str()
                .unwrap()
                .contains("toucan finger throat")
        );
    }

    #[test]
    fn test_request_logs_sample() {
        let input = include_str!("../samples/request_logs.json");
        let mut output = Vec::new();

        convert_jdbc(&input, &mut output).unwrap();

        let records = parse_output_lines(&output);
        assert_eq!(records.len(), 10);

        let first = records[0].as_object().unwrap();
        assert_eq!(
            first.get(&"trace_id").unwrap().as_str().unwrap(),
            "0b03a46a-db71-404a-b637-e945b1a0f0eb"
        );
        assert_eq!(first.get(&"bytes_received").unwrap().as_i64().unwrap(), 188);
        assert_eq!(first.get(&"latency_ms").unwrap().as_i64().unwrap(), 75);
    }

    #[test]
    fn test_big5_sample() {
        let input = include_str!("../samples/big5.json");
        let mut output = Vec::new();

        convert_jdbc(&input, &mut output).unwrap();

        let records = parse_output_lines(&output);
        assert_eq!(records.len(), 10);

        let first = records[0].as_object().unwrap();
        assert!(first.contains_key(&"agent"));
        assert!(first.contains_key(&"process"));
        assert!(first.contains_key(&"@timestamp"));

        let agent = first.get(&"agent").unwrap().as_object().unwrap();
        assert_eq!(
            agent.get(&"name").unwrap().as_str().unwrap(),
            "crimsonleader"
        );
    }

    #[test]
    fn test_empty_datarows() {
        let input = r#"{"schema":[{"name":"id","type":"int"}],"datarows":[]}"#;
        let mut output = Vec::new();

        convert_jdbc(&input, &mut output).unwrap();

        let records = parse_output_lines(&output);
        assert_eq!(records.len(), 0);
    }

    #[test]
    fn test_single_row() {
        let input = r#"{"schema":[{"name":"name","type":"string"},{"name":"age","type":"int"}],"datarows":[["Alice",30]]}"#;
        let mut output = Vec::new();

        convert_jdbc(&input, &mut output).unwrap();

        let records = parse_output_lines(&output);
        assert_eq!(records.len(), 1);

        let first = records[0].as_object().unwrap();
        assert_eq!(first.get(&"name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(first.get(&"age").unwrap().as_i64().unwrap(), 30);
    }
}
