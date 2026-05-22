use sonic_rs::{JsonContainerTrait, JsonValueTrait, Object, Value};
use std::env;
use std::fs;
use std::io::{self, BufWriter, Read, Write};
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let input_json = if args.len() > 1 {
        let filename = &args[1];
        fs::read_to_string(filename).unwrap_or_else(|err| {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        })
    } else {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .unwrap_or_else(|err| {
                eprintln!("Error reading from stdin: {}", err);
                process::exit(1);
            });
        buffer
    };

    let parsed: Value = sonic_rs::from_str(&input_json).unwrap_or_else(|err| {
        eprintln!("Error parsing JDBC JSON: {}", err);
        process::exit(1);
    });

    let obj = parsed.as_object().unwrap_or_else(|| {
        eprintln!("Expected JSON object at root");
        process::exit(1);
    });

    let schema = obj.get(&"schema").and_then(|s| s.as_array()).unwrap_or_else(|| {
        eprintln!("Missing or invalid 'schema' field");
        process::exit(1);
    });

    let field_names: Vec<&str> = schema
        .iter()
        .filter_map(|field| {
            field
                .as_object()
                .and_then(|f| f.get(&"name"))
                .and_then(|n| n.as_str())
        })
        .collect();

    let datarows = obj.get(&"datarows").and_then(|d| d.as_array()).unwrap_or_else(|| {
        eprintln!("Missing or invalid 'datarows' field");
        process::exit(1);
    });

    let stdout = io::stdout();
    let lock = stdout.lock();
    let mut writer = BufWriter::new(lock);

    for row in datarows.iter() {
        if let Some(row_array) = row.as_array() {
            let mut record = Object::new();
            for (i, value) in row_array.iter().enumerate() {
                if let Some(&field_name) = field_names.get(i) {
                    record.insert(field_name, value.clone());
                }
            }

            let output = sonic_rs::to_string(&record).unwrap_or_else(|err| {
                eprintln!("Error serializing output: {}", err);
                let _ = writer.flush();
                process::exit(1);
            });
            if writeln!(writer, "{}", output).is_err() {
                break;
            }
        }
    }
    let _ = writer.flush();
}
