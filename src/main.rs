use std::env;
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::process;
use unjdbc::process_jdbc_json_to_writer;

fn main() {
    let args: Vec<String> = env::args().collect();

    let stdin = io::stdin();
    let stdout = io::stdout();

    let result = if args.len() > 1 {
        let filename = &args[1];
        let file = File::open(filename).unwrap_or_else(|err| {
            eprintln!("Error opening file '{}': {}", filename, err);
            process::exit(1);
        });
        let mut reader = BufReader::new(file);
        let mut writer = BufWriter::new(stdout.lock());
        process_jdbc_json_to_writer(&mut reader, &mut writer)
    } else {
        let mut reader = BufReader::new(stdin.lock());
        let mut writer = BufWriter::new(stdout.lock());
        process_jdbc_json_to_writer(&mut reader, &mut writer)
    };

    if let Err(err) = result {
        eprintln!("Error processing JDBC JSON: {}", err);
        process::exit(1);
    }
}
