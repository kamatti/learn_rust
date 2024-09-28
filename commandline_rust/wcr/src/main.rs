use anyhow::Result;
use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(default_value = "-", value_name = "FILES")]
    files: Vec<String>,

    #[arg(short, long)]
    lines: bool,

    #[arg(short, long)]
    words: bool,

    #[arg(short('c'), long)]
    bytes: bool,

    #[arg(short('m'), long, conflicts_with("bytes"))]
    chars: bool,
}

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run(mut args: Args) -> Result<()> {
    // 全てのフラグがFalseだったらlines, words, bytesをTrueにする
    if !args.lines && !args.words && !args.bytes && !args.chars {
        args.lines = true;
        args.words = true;
        args.bytes = true;
    }

    let mut total_bytes = 0usize;
    let mut total_chars = 0usize;
    let mut total_lines = 0usize;
    let mut total_words = 0usize;
    for filename in &args.files {
        match open(filename) {
            Err(e) => eprintln!("{}: {}", filename, e),
            Ok(file) => {
                let file_info = count(file)?;
                if args.lines {
                    total_lines += file_info.num_lines;
                    print!("{:>8}", file_info.num_lines);
                }
                if args.words {
                    total_words += file_info.num_words;
                    print!("{:>8}", file_info.num_words);
                }
                if args.bytes {
                    total_bytes += file_info.num_bytes;
                    print!("{:>8}", file_info.num_bytes);
                }
                if args.chars {
                    total_chars += file_info.num_chars;
                    print!("{:>8}", file_info.num_chars);
                }
                if filename != "-" {
                    print!(" {filename}")
                }
                println!("");
            }
        }
    }
    if args.files.len() >= 2 {
        if args.lines {
            print!("{:>8}", total_lines);
        }
        if args.words {
            print!("{:>8}", total_words);
        }
        if args.bytes {
            print!("{:>8}", total_bytes);
        }
        if args.chars {
            print!("{:>8}", total_chars);
        }
        println!(" total");
    }
    Ok(())
}

fn open(filename: &str) -> Result<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

pub fn count(mut file: impl BufRead) -> Result<FileInfo> {
    let mut num_bytes = 0usize;
    let mut num_chars = 0usize;
    let mut num_lines = 0usize;
    let mut num_words = 0usize;

    // 1行ずつ読み込んで数え上げないと巨大なファイルを扱えないかも？
    // read_lineで一行ずつ読み込んで数え上げようとしてもファイルが一行だったらだめ？
    let bytes = file.bytes().collect::<Result<Vec<_>, _>>()?;
    let utf8_bytes = String::from_utf8_lossy(&bytes);
    num_bytes = utf8_bytes.bytes().count();
    num_chars = utf8_bytes.chars().count();
    num_words = utf8_bytes.split_whitespace().count();
    num_lines = utf8_bytes.split_terminator('\n').count();

    Ok(FileInfo {
        num_bytes,
        num_chars,
        num_lines,
        num_words,
    })
}

#[cfg(test)]
mod tests {
    use super::{count, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_bytes: 48,
            num_chars: 48,
            num_lines: 1,
            num_words: 10,
        };
        assert_eq!(info.unwrap(), expected);
    }
}
