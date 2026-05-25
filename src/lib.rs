use anyhow::{Context, Result, anyhow};
use logos::{Lexer, Logos};
use std::io::Write;

/// Token scheme designed for efficiently copying input to formatted output.
/// We only really care about object markers and spans of bytes.
///
/// For numbers and strings, regex is from logos handbook.
#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\r\n]+")]
enum Token<'source> {
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
    Number(&'source str),
    #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#)]
    String(&'source str),
}

struct Scanner<'source, 'dest, W: Write> {
    lexer: Lexer<'source, Token<'source>>,
    writer: &'dest mut W,
    fields: Vec<&'source str>,
    stack: Vec<u8>,
}

impl<'source, 'dest, W: Write> Scanner<'source, 'dest, W> {
    fn next(self: &mut Self) -> Result<Token<'source>> {
        match self.lexer.next() {
            Some(Ok(t)) => Ok(t),
            Some(Err(_)) => Err(anyhow!("invalid token")),
            None => Err(anyhow!("unexpected eof")),
        }
    }

    fn take(self: &mut Self, token: Token<'static>) -> Result<Token<'source>> {
        match self.next()? {
            t if t == token => Ok(token),
            t => Err(anyhow!("expected {token:?}, found {t:?}")),
        }
    }

    fn take_string(self: &mut Self) -> Result<Token<'source>> {
        let token = self.next()?;
        match token {
            Token::String(_) => Ok(token),
            _ => Err(anyhow!("expected a string, found {token:?}")),
        }
    }

    fn skip_value(self: &mut Self) -> Result<()> {
        let ssize = self.stack.len();

        match self.next()? {
            Token::Null | Token::True | Token::False | Token::Number(_) | Token::String(_) => {
                return Ok(());
            }
            Token::LeftBrace => self.stack.push(b'{'),
            Token::LeftBracket => self.stack.push(b'['),
            other => {
                return Err(anyhow!("expected a value, found {other:?}"));
            }
        }

        loop {
            match self.next()? {
                Token::LeftBrace => self.stack.push(b'{'),
                Token::LeftBracket => todo!(),
                Token::RightBrace => {
                    if self.stack.pop() != Some(b'{') {
                        return Err(anyhow!("mismatched brackets: matching {{, got ]"));
                    }
                    if self.stack.len() == ssize {
                        return Ok(());
                    }
                }
                Token::RightBracket => {
                    if self.stack.pop() != Some(b'[') {
                        return Err(anyhow!("mismatched brackets: matching [, got }}"));
                    }
                    if self.stack.len() == ssize {
                        return Ok(());
                    }
                }
                // there might be invalid json in here -- for the scope of our tool we really don't care
                _ => continue,
            }
        }
    }

    fn emit_value(self: &mut Self) -> Result<()> {
        // long method, basically the same system as skip_value but we emit tokens as we go
        let ssize = self.stack.len();

        match self.next()? {
            Token::Null => {
                write!(&mut self.writer, "null")?;
                return Ok(());
            }
            Token::True => {
                write!(&mut self.writer, "true")?;
                return Ok(());
            }
            Token::False => {
                write!(&mut self.writer, "false")?;
                return Ok(());
            }
            Token::Number(s) => {
                write!(&mut self.writer, "{s}")?;
                return Ok(());
            }
            Token::String(s) => {
                write!(&mut self.writer, "{s}")?;
                return Ok(());
            }
            Token::LeftBrace => {
                write!(&mut self.writer, "{{")?;
                self.stack.push(b'{');
            }
            Token::LeftBracket => {
                write!(&mut self.writer, "[")?;
                self.stack.push(b'[');
            }
            other => {
                return Err(anyhow!("expected a value, found {other:?}"));
            }
        }

        loop {
            match self.next()? {
                Token::LeftBrace => {
                    write!(&mut self.writer, "{{")?;
                    self.stack.push(b'{');
                }
                Token::LeftBracket => {
                    write!(&mut self.writer, "[")?;
                    self.stack.push(b'[');
                }
                Token::RightBrace => {
                    if self.stack.pop() != Some(b'{') {
                        return Err(anyhow!("mismatched brackets: matching {{, got ]"));
                    }
                    write!(&mut self.writer, "}}")?;
                    if self.stack.len() == ssize {
                        return Ok(());
                    }
                }
                Token::RightBracket => {
                    if self.stack.pop() != Some(b'[') {
                        return Err(anyhow!("mismatched brackets: matching [, got }}"));
                    }
                    write!(&mut self.writer, "]")?;
                    if self.stack.len() == ssize {
                        return Ok(());
                    }
                }
                Token::Null => {
                    write!(&mut self.writer, "null")?;
                }
                Token::True => {
                    write!(&mut self.writer, "true")?;
                }
                Token::False => {
                    write!(&mut self.writer, "false")?;
                }
                Token::Number(s) => {
                    write!(&mut self.writer, "{s}")?;
                }
                Token::String(s) => {
                    write!(&mut self.writer, "{s}")?;
                }
                Token::Colon => {
                    write!(&mut self.writer, ": ")?;
                }
                Token::Comma => {
                    write!(&mut self.writer, ", ")?;
                }
            }
        }
    }

    fn exit_object(self: &mut Self) -> Result<bool> {
        match self.next()? {
            Token::RightBrace => Ok(true),
            Token::Comma => Ok(false),
            other => Err(anyhow!(
                "expected comma or end-of-object marker, found {other:?}"
            )),
        }
    }

    fn exit_array(self: &mut Self) -> Result<bool> {
        match self.next()? {
            Token::RightBracket => Ok(true),
            Token::Comma => Ok(false),
            other => Err(anyhow!(
                "expected comma or end-of-array marker, found {other:?}"
            )),
        }
    }

    fn seek_key(self: &mut Self, key: &str) -> Result<()> {
        while let Some(current) = self.lexer.next() {
            let current = current.map_err(|_| anyhow!("invalid token"))?;
            let Token::String(currkey) = current else {
                return Err(anyhow!("expected an object key, got {:?}", current));
            };
            self.take(Token::Colon)?;
            if key == currkey {
                return Ok(());
            }
            self.skip_value()?;
            if self.exit_object()? {
                break;
            }
        }

        Err(anyhow!("expected to find key {key}, but didn't"))
    }

    fn seek_end_of_object(self: &mut Self) -> Result<()> {
        loop {
            if self.exit_object()? {
                return Ok(());
            }
            self.take_string()?;
            self.take(Token::Colon)?;
            self.skip_value()?;
        }
    }

    fn scan_schema(self: &mut Self) -> Result<()> {
        self.take(Token::LeftBracket)?;
        let mut next = self.next()?;
        if next == Token::RightBracket {
            return Ok(());
        }
        loop {
            if next != Token::LeftBrace {
                return Err(anyhow!("expected an object, found {next:?}"));
            }
            self.seek_key("\"name\"")?;
            match self.take_string()? {
                Token::String(s) => self.fields.push(s),
                _ => unreachable!(),
            }
            self.seek_end_of_object()?;
            if self.exit_array()? {
                break;
            }
            next = self.next()?;
        }
        if self.exit_object()? {
            return Err(anyhow!(
                "object should continue after schema (for datarows), but terminates"
            ));
        }
        Ok(())
    }

    fn scan_datarow(self: &mut Self) -> Result<()> {
        write!(&mut self.writer, "{{")?;
        let mut idx = 0;
        while idx < self.fields.len() {
            write!(&mut self.writer, "{}: ", self.fields[idx])?;
            self.emit_value()?;
            idx += 1;
            if idx < self.fields.len() {
                self.take(Token::Comma)
                    .context("ran out of elements in datarow")?;
                write!(&mut self.writer, ", ")?;
            }
        }
        self.take(Token::RightBracket)?;
        write!(&mut self.writer, "}}\n")?;
        Ok(())
    }

    fn scan_datarows(self: &mut Self) -> Result<()> {
        self.take(Token::LeftBracket)?;
        let mut next = self.next()?;
        if next == Token::RightBracket {
            return Ok(());
        }
        loop {
            if next != Token::LeftBracket {
                return Err(anyhow!(
                    "datarows entries should be lists, but starts with {next:?}"
                ));
            }
            self.scan_datarow()?;
            if self.exit_array()? {
                break;
            }
            next = self.next()?;
        }
        Ok(())
    }

    fn scan_jdbc(self: &mut Self) -> Result<()> {
        self.take(Token::LeftBrace)?;
        self.seek_key("\"schema\"")
            .context("failed to seek schema key")?;
        self.scan_schema().context("failed to scan schema")?;
        self.seek_key("\"datarows\"")
            .context("failed to seek datarows key")?;
        self.scan_datarows().context("failed to scan datarows")?;
        self.seek_end_of_object()
            .context("failed to find end of object after datarows")?;
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
    scanner
        .scan_jdbc()
        .context("failed to scan the jdbc input")?;
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
