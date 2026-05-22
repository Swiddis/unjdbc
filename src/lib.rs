use sonic_rs::{JsonContainerTrait, JsonValueTrait, Object, Value};

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
    let mut record = Object::new();
    for (i, value) in row.iter().enumerate() {
        if let Some(&field_name) = field_names.get(i) {
            record.insert(field_name, value.clone());
        }
    }
    record
}

pub fn transform_datarows(datarows: &[Value], field_names: &[&str]) -> Vec<String> {
    datarows
        .iter()
        .filter_map(|row| row.as_array())
        .map(|row_array| {
            let record = transform_row(row_array, field_names);
            sonic_rs::to_string(&record).unwrap()
        })
        .collect()
}

pub fn process_jdbc_json(input_json: &str) -> Result<Vec<String>, String> {
    let parsed: Value = parse_jdbc_json(input_json).map_err(|e| e.to_string())?;

    let obj = parsed
        .as_object()
        .ok_or("Expected JSON object at root")?;

    let schema = obj
        .get(&"schema")
        .and_then(|s| s.as_array())
        .ok_or("Missing or invalid 'schema' field")?;

    let field_names = extract_field_names(schema);

    let datarows = obj
        .get(&"datarows")
        .and_then(|d| d.as_array())
        .ok_or("Missing or invalid 'datarows' field")?;

    Ok(transform_datarows(datarows, &field_names))
}
