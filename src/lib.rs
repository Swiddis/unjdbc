use sonic_rs::{JsonContainerTrait, JsonValueTrait, Object, Value};
use std::io::{Read, Write};

fn parse_jdbc_json(input: &str) -> Result<Value, sonic_rs::Error> {
    sonic_rs::from_str(input)
}

fn extract_field_names(schema: &[Value]) -> Vec<&str> {
    schema
        .iter()
        .filter_map(|field| {
            field
                .as_object()
                .and_then(|f| f.get(&"name"))
                .and_then(|n| n.as_str())
        })
        .collect()
}

fn transform_row(row: &[Value], field_names: &[&str]) -> Object {
    let mut record = Object::with_capacity(field_names.len());
    for (&field_name, value) in field_names.iter().zip(row.iter()) {
        record.insert(field_name, value.clone());
    }
    record
}

pub fn convert_jdbc<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
) -> Result<(), String> {
    let mut input_json = String::new();
    reader
        .read_to_string(&mut input_json)
        .map_err(|e| e.to_string())?;

    let parsed: Value = parse_jdbc_json(&input_json).map_err(|e| e.to_string())?;

    let obj = parsed.as_object().ok_or("Expected JSON object at root")?;

    let schema = obj
        .get(&"schema")
        .and_then(|s| s.as_array())
        .ok_or("Missing or invalid 'schema' field")?;

    let field_names = extract_field_names(schema);

    let datarows = obj
        .get(&"datarows")
        .and_then(|d| d.as_array())
        .ok_or("Missing or invalid 'datarows' field")?;

    for row in datarows.iter().filter_map(|row| row.as_array()) {
        let record = transform_row(row, &field_names);
        let json_str = sonic_rs::to_string(&record).map_err(|e| e.to_string())?;
        writeln!(writer, "{}", json_str).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

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
        let mut reader = Cursor::new(input.as_bytes());
        let mut output = Vec::new();

        convert_jdbc(&mut reader, &mut output).unwrap();

        let records = parse_output_lines(&output);
        assert_eq!(records.len(), 10);

        let first = records[0].as_object().unwrap();
        assert_eq!(first.get(&"@timestamp").unwrap().as_str().unwrap(), "2023-01-01 00:35:22");
        assert!(first.get(&"message").unwrap().as_str().unwrap().contains("toucan finger throat"));
    }

    #[test]
    fn test_request_logs_sample() {
        let input = include_str!("../samples/request_logs.json");
        let mut reader = Cursor::new(input.as_bytes());
        let mut output = Vec::new();

        convert_jdbc(&mut reader, &mut output).unwrap();

        let records = parse_output_lines(&output);
        assert_eq!(records.len(), 10);

        let first = records[0].as_object().unwrap();
        assert_eq!(first.get(&"trace_id").unwrap().as_str().unwrap(), "0b03a46a-db71-404a-b637-e945b1a0f0eb");
        assert_eq!(first.get(&"bytes_received").unwrap().as_i64().unwrap(), 188);
        assert_eq!(first.get(&"latency_ms").unwrap().as_i64().unwrap(), 75);
    }

    #[test]
    fn test_big5_sample() {
        let input = include_str!("../samples/big5.json");
        let mut reader = Cursor::new(input.as_bytes());
        let mut output = Vec::new();

        convert_jdbc(&mut reader, &mut output).unwrap();

        let records = parse_output_lines(&output);
        assert_eq!(records.len(), 10);

        let first = records[0].as_object().unwrap();
        assert!(first.contains_key(&"agent"));
        assert!(first.contains_key(&"process"));
        assert!(first.contains_key(&"@timestamp"));

        let agent = first.get(&"agent").unwrap().as_object().unwrap();
        assert_eq!(agent.get(&"name").unwrap().as_str().unwrap(), "crimsonleader");
    }

    #[test]
    fn test_empty_datarows() {
        let input = r#"{"schema":[{"name":"id","type":"int"}],"datarows":[]}"#;
        let mut reader = Cursor::new(input.as_bytes());
        let mut output = Vec::new();

        convert_jdbc(&mut reader, &mut output).unwrap();

        let records = parse_output_lines(&output);
        assert_eq!(records.len(), 0);
    }

    #[test]
    fn test_single_row() {
        let input = r#"{"schema":[{"name":"name","type":"string"},{"name":"age","type":"int"}],"datarows":[["Alice",30]]}"#;
        let mut reader = Cursor::new(input.as_bytes());
        let mut output = Vec::new();

        convert_jdbc(&mut reader, &mut output).unwrap();

        let records = parse_output_lines(&output);
        assert_eq!(records.len(), 1);

        let first = records[0].as_object().unwrap();
        assert_eq!(first.get(&"name").unwrap().as_str().unwrap(), "Alice");
        assert_eq!(first.get(&"age").unwrap().as_i64().unwrap(), 30);
    }
}
