use anyhow::{anyhow, bail, Result};
use clap::Parser;
use csv::{ReaderBuilder, StringRecord, WriterBuilder};
use regex::Regex;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    num::NonZeroUsize,
    ops::Range,
};

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
    println!("{:#?}", args);
    Ok(())
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

fn parse_pos(range: String) -> Result<PositionList> {
    // unimplemented!();
    let mut ranges: Vec<Range<usize>> = vec![];

    // "+"とalphabetが含まれてたらエラーにする
    let error_regex = Regex::new(r"[+[:alpha:]]+")?;
    // if error_regex.is_match(&range) {
    //     bail!("illegal list value: \"{}\"", range);
    // }

    for splitted in range.split(',') {
        if error_regex.is_match(splitted) {
            bail!("illegal list value: \"{}\"", splitted);
        }
        if splitted.contains('-') {
            let mut sub = splitted.split('-');
            let (Some(pre), Some(later), None) = (sub.next(), sub.next(), sub.next()) else {
                bail!("illegal list value: \"{}\"", splitted);
            };
            let pre = pre.parse::<usize>()?;
            let later = later.parse::<usize>()?;
            if pre == 0 {
                bail!("illegal list value: \"{}\"", pre);
            } else if pre >= later {
                // bail!("illegal list value: \"{}\"", splitted);
                bail!(
                    "First number in range ({}) must be lower than second number ({})",
                    pre,
                    later
                )
            }
            ranges.push(pre - 1..later);
        } else {
            let num = splitted.parse::<usize>()?;
            if num == 0 {
                bail!("illegal list value: \"{}\"", num);
            }
            ranges.push(num - 1..num);
        }
    }

    Ok(ranges)
}

#[cfg(test)]
mod unit_tests {
    // use super::{extract_bytes, extract_chars, extract_fields, parse_pos};
    use super::parse_pos;
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

    // #[test]
    // fn test_extract_fields() {
    //     let rec = StringRecord::from(vec!["Captain", "Sham", "12345"]);
    //     assert_eq!(extract_fields(&rec, &[0..1]), &["Captain"]);
    //     assert_eq!(extract_fields(&rec, &[1..2]), &["Sham"]);
    //     assert_eq!(extract_fields(&rec, &[0..1, 2..3]), &["Captain", "12345"]);
    //     assert_eq!(extract_fields(&rec, &[0..1, 3..4]), &["Captain"]);
    //     assert_eq!(extract_fields(&rec, &[1..2, 0..1]), &["Sham", "Captain"]);
    // }

    // #[test]
    // fn test_extract_chars() {
    //     assert_eq!(extract_chars("", &[0..1]), "".to_string());
    //     assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
    //     assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
    //     assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
    //     assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
    //     assert_eq!(extract_chars("ábc", &[0..1, 1..2, 4..5]), "áb".to_string());
    // }

    // #[test]
    // fn test_extract_bytes() {
    //     assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
    //     assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
    //     assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
    //     assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
    //     assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
    //     assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    // }
}
