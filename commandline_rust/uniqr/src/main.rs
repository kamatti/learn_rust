use anyhow::{anyhow, Result};
use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(default_value = "-", value_name = "INPUT_FILE")]
    input_file: String,

    #[arg(value_name = "OUTPUT_FILE")]
    output_file: Option<String>,

    #[arg(short, long)]
    count: bool,
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    let mut file = open(&args.input_file).map_err(|e| anyhow!("{}: {}", args.input_file, e))?;
    let mut line = String::new();
    let mut previous = String::new();
    let mut writer: Box<dyn Write> = if let Some(out_file) = args.output_file {
        Box::new(BufWriter::new(File::create(out_file)?))
    } else {
        Box::new(BufWriter::new(io::stdout()))
    };

    if file.read_line(&mut previous)? != 0 {
        let mut conti_count = 1u64;
        loop {
            let num_bytes = file.read_line(&mut line)?;
            if previous.trim_end() == line.trim_end() {
                conti_count += 1;
            } else {
                if args.count {
                    writer.write_fmt(format_args!("{:>4} {}", conti_count, previous))?;
                    conti_count = 1;
                } else {
                    writer.write_fmt(format_args!("{}", previous))?;
                }
                previous = line.clone();
            }
            if num_bytes == 0 {
                break;
            }
            line.clear();
        }
    }
    Ok(())
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
