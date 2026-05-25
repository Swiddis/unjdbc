use anyhow::Result;
use std::env;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::process;
use unjdbc::convert_jdbc;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let stdin = io::stdin();
    let stdout = io::stdout();

    if args.len() > 1 {
        let filename = &args[1];
        let mut file = File::open(filename)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        let mut writer = BufWriter::new(stdout.lock());
        convert_jdbc(&buf, &mut writer)?;
        writer.flush()?;
    } else {
        let mut reader = BufReader::new(stdin.lock());
        let mut buf = String::new();
        reader.read_to_string(&mut buf).unwrap_or_else(|err| {
            eprintln!("Error reading stdin: {}", err);
            process::exit(1);
        });
        let mut writer = BufWriter::new(stdout.lock());
        convert_jdbc(&buf, &mut writer)?;
        writer.flush()?;
    }
    Ok(())
}
