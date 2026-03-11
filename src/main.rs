use serde::Deserialize;
use serde_json::{Map, Value};
use std::env;
use std::fs;
use std::io::{self, Read};
use std::process;

#[derive(Debug, Deserialize)]
struct JdbcResponse {
    schema: Vec<SchemaField>,
    datarows: Vec<Vec<Value>>,
}

#[derive(Debug, Deserialize)]
struct SchemaField {
    name: String,
}

fn convert_jdbc_to_json(jdbc: JdbcResponse) -> Vec<Map<String, Value>> {
    jdbc.datarows
        .into_iter()
        .map(|row| {
            let mut obj = Map::new();
            for (i, value) in row.into_iter().enumerate() {
                if let Some(field) = jdbc.schema.get(i) {
                    obj.insert(field.name.clone(), value);
                }
            }
            obj
        })
        .collect()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let input_json = if args.len() > 1 {
        // Read from file
        let filename = &args[1];
        fs::read_to_string(filename).unwrap_or_else(|err| {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        })
    } else {
        // Read from stdin
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .unwrap_or_else(|err| {
                eprintln!("Error reading from stdin: {}", err);
                process::exit(1);
            });
        buffer
    };

    let jdbc_response: JdbcResponse = serde_json::from_str(&input_json).unwrap_or_else(|err| {
        eprintln!("Error parsing JDBC JSON: {}", err);
        process::exit(1);
    });

    let result = convert_jdbc_to_json(jdbc_response);

    // Output as ndjson (newline-delimited JSON)
    for record in result {
        let output = serde_json::to_string(&record).unwrap_or_else(|err| {
            eprintln!("Error serializing output: {}", err);
            process::exit(1);
        });
        println!("{}", output);
    }
}
