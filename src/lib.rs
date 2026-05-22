use sonic_rs::{JsonContainerTrait, JsonValueTrait, Object, Value};
use std::io::Write;

pub fn parse_jdbc_json(input: &str) -> Result<Value, sonic_rs::Error> {
    sonic_rs::from_str(input)
}

pub fn extract_field_names(schema: &[Value]) -> Vec<&str> {
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

pub fn transform_row(row: &[Value], field_names: &[&str]) -> Object {
    let mut record = Object::with_capacity(field_names.len());
    for (&field_name, value) in field_names.iter().zip(row.iter()) {
        record.insert(field_name, value.clone());
    }
    record
}

pub fn transform_datarows(datarows: &[Value], field_names: &[&str]) -> Vec<String> {
    let mut results = Vec::with_capacity(datarows.len());
    for row in datarows.iter().filter_map(|row| row.as_array()) {
        let record = transform_row(row, field_names);
        results.push(sonic_rs::to_string(&record).unwrap());
    }
    results
}

pub fn process_jdbc_json_to_writer<W: Write>(
    input_json: &str,
    writer: &mut W,
) -> Result<(), String> {
    let parsed: Value = parse_jdbc_json(input_json).map_err(|e| e.to_string())?;

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
