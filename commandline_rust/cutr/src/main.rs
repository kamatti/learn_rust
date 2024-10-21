use anyhow::{bail, Result};
use clap::Parser;
use core::str;
use csv::StringRecord;
use regex::Regex;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    num::NonZeroUsize,
    ops::Range,
};
use unicode_segmentation::UnicodeSegmentation;

type PositionList = Vec<Range<usize>>;

#[derive(Debug, Parser)]
#[command(author, about, version)]
struct Args {
    #[arg(default_value = "-")]
    files: Vec<String>,

    #[arg(short, long, default_value = "\t")]
    delimiter: String,

    #[command(flatten)]
    extract: ArgsExtract,
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
struct ArgsExtract {
    #[arg(short, long)]
    fields: Option<String>,

    #[arg(short, long)]
    bytes: Option<String>,

    #[arg(short, long)]
    chars: Option<String>,
}

#[derive(Debug)]
enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    if args.delimiter.len() != 1 {
        bail!("--delim \"{}\" must be a single byte", args.delimiter);
    }
    let delimiter: u8 = *args.delimiter.as_bytes().first().unwrap();

    let extract = match (args.extract.fields, args.extract.bytes, args.extract.chars) {
        (Some(fields), None, None) => Extract::Fields(parse_pos(fields)?),
        (None, Some(bytes), None) => Extract::Bytes(parse_pos(bytes)?),
        (None, None, Some(chars)) => Extract::Chars(parse_pos(chars)?),
        _ => bail!("Must have --fields, --bytes, or -- chars"),
    };

    for filename in args.files {
        match open(&filename) {
            Err(e) => eprint!("{}: {}", filename, e),
            Ok(file) => match &extract {
                Extract::Fields(pos) => {
                    let mut rdr = csv::ReaderBuilder::new()
                        .delimiter(delimiter)
                        .has_headers(false)
                        .from_reader(file);

                    let mut wdr = csv::WriterBuilder::new()
                        .delimiter(delimiter)
                        .from_writer(io::stdout());

                    for record in rdr.records() {
                        wdr.write_record(extract_fields(&record?, pos))?;
                    }
                }
                Extract::Bytes(pos) => {
                    for line in file.lines() {
                        println!("{}", extract_bytes(&line?, pos));
                    }
                }
                Extract::Chars(pos) => {
                    for line in file.lines() {
                        println!("{}", extract_chars(&line?, pos));
                    }
                }
            },
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

fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    let mut res = String::from("");
    let chars: Vec<&str> = line.graphemes(true).collect();
    for pos in char_pos {
        if let Some(ch) = chars.get(pos.clone()) {
            res.push_str(ch.concat().as_str());
        }
    }

    res
}

fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    let mut res = String::from("");
    let bytes: Vec<u8> = line.bytes().collect();
    for pos in byte_pos {
        if let Some(bytes) = bytes.get(pos.clone()) {
            res.push_str(String::from_utf8_lossy(bytes).as_ref());
        }
    }

    res
}

fn extract_fields(line: &StringRecord, field_pos: &[Range<usize>]) -> Vec<String> {
    let mut res: Vec<String> = vec![];

    for pos in field_pos {
        for pos in pos.start..pos.end {
            if let Some(field) = line.get(pos) {
                res.push(field.to_string());
            }
        }
    }

    res
}

fn parse_pos(range: String) -> Result<PositionList> {
    if range.is_empty() {
        bail!("");
    }
    let err_msg = |x| format!(r#"illegal list value: "{x}""#);
    // 有効な範囲指定: \d+(-\d+)?
    let valid_pattern = Regex::new(r"^(?<first>\d+)(-(?<second>\d+))?$").unwrap();
    let mut ranges = vec![];

    for splitted in range.split(',') {
        if let Some(caps) = valid_pattern.captures(splitted) {
            let first = caps.name("first").unwrap();
            if let Some(second) = caps.name("second") {
                // exist both "first" and "second"
                match (
                    first.as_str().parse::<NonZeroUsize>(),
                    second.as_str().parse::<NonZeroUsize>(),
                ) {
                    (Ok(f), Ok(s)) if f < s => ranges.push(f.get() - 1..s.get()),
                    (Ok(f), Ok(s)) if f >= s => bail!(
                        "First number in range ({}) must be lower than second number ({})",
                        f,
                        s
                    ),
                    (Err(_), _) => bail!(err_msg("0")),
                    (_, _) => bail!(err_msg(splitted)),
                }
            } else {
                // exist only "first"
                if let Ok(first) = first.as_str().parse::<NonZeroUsize>() {
                    let f = first.get();
                    ranges.push(f - 1..f);
                } else {
                    bail!(err_msg("0"));
                }
            }
        } else {
            bail!(err_msg(splitted));
        }
    }

    Ok(ranges)
}

#[cfg(test)]
mod unit_tests {
    use super::{extract_bytes, extract_chars, extract_fields, parse_pos};
    use csv::StringRecord;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_pos() {
        // The empty string is an error
        assert!(parse_pos("".to_string()).is_err());

        // Zero is an error
        let res = parse_pos("0".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "0""#);

        let res = parse_pos("0-1".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "0""#);

        // A leading "+" is an error
        let res = parse_pos("+1".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "+1""#,);

        let res = parse_pos("+1-2".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "+1-2""#,
        );

        let res = parse_pos("1-+2".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            r#"illegal list value: "1-+2""#,
        );

        // Any non-number is an error
        let res = parse_pos("a".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a""#);

        let res = parse_pos("1,a".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a""#);

        let res = parse_pos("1-a".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "1-a""#,);

        let res = parse_pos("a-1".to_string());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), r#"illegal list value: "a-1""#,);

        // Wonky ranges
        let res = parse_pos("-".to_string());
        assert!(res.is_err());

        let res = parse_pos(",".to_string());
        assert!(res.is_err());

        let res = parse_pos("1,".to_string());
        assert!(res.is_err());

        let res = parse_pos("1-".to_string());
        assert!(res.is_err());

        let res = parse_pos("1-1-1".to_string());
        assert!(res.is_err());

        let res = parse_pos("1-1-a".to_string());
        assert!(res.is_err());

        // First number must be less than second
        let res = parse_pos("1-1".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );

        let res = parse_pos("2-1".to_string());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );

        // All the following are acceptable
        let res = parse_pos("1".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("01".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("1,3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("001,0003".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("1-3".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("0001-03".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("1,7,3-5".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);

        let res = parse_pos("15,19-20".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }

    #[test]
    fn test_extract_fields() {
        let rec = StringRecord::from(vec!["Captain", "Sham", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2]), &["Sham"]);
        assert_eq!(extract_fields(&rec, &[0..1, 2..3]), &["Captain", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1, 3..4]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2, 0..1]), &["Sham", "Captain"]);
    }

    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..1]), "".to_string());
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 1..2, 4..5]), "áb".to_string());
    }

    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    }
}
