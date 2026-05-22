use std::env;
use std::fs;
use std::io::{self, BufWriter, Read, Write};
use std::process;
use unjdbc::process_jdbc_json;

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

    let results = process_jdbc_json(&input_json).unwrap_or_else(|err| {
        eprintln!("Error processing JDBC JSON: {}", err);
        process::exit(1);
    });

    let stdout = io::stdout();
    let lock = stdout.lock();
    let mut writer = BufWriter::new(lock);

    for output in results {
        if writeln!(writer, "{}", output).is_err() {
            break;
        }
    }
    let _ = writer.flush();
}
