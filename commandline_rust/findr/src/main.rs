use anyhow::Result;
use clap::{builder::PossibleValue, ArgAction, Parser, ValueEnum};
use regex::Regex;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(value_name = "PATH", default_value = ".")]
    paths: Vec<String>,

    #[arg(value_name="NAME", short('n'), long("name"), value_parser(Regex::new), action(ArgAction::Append), num_args(0..))]
    names: Vec<Regex>,

    #[arg(value_name="TYPE", short('t'), long("type"), value_parser(clap::value_parser!(EntryType)), action(ArgAction::Append), num_args(0..))]
    entry_types: Vec<EntryType>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum EntryType {
    Dir,
    File,
    Link,
}

impl ValueEnum for EntryType {
    fn value_variants<'a>() -> &'a [Self] {
        &[EntryType::Dir, EntryType::File, EntryType::Link]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            EntryType::Dir => PossibleValue::new("d"),
            EntryType::File => PossibleValue::new("f"),
            EntryType::Link => PossibleValue::new("l"),
        })
    }
}

fn main() {
    if let Err(e) = run(Args::parse()) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<()> {
    for path in args.paths {
        for entry in WalkDir::new(path) {
            match entry {
                Err(e) => eprintln!("{}", e),
                Ok(entry) => {
                    if !args.names.is_empty() {
                        for name in &args.names {
                            if name.is_match(entry.file_name().to_str().unwrap()) {
                                if !args.entry_types.is_empty() {
                                    for etype in &args.entry_types {
                                        match etype {
                                            EntryType::Dir => {
                                                if entry.file_type().is_dir() {
                                                    println!("{}", entry.path().display());
                                                    break;
                                                }
                                            }
                                            EntryType::File => {
                                                if entry.file_type().is_file() {
                                                    println!("{}", entry.path().display());
                                                    break;
                                                }
                                            }
                                            EntryType::Link => {
                                                if entry.file_type().is_symlink() {
                                                    println!("{}", entry.path().display());
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    println!("{}", entry.path().display());
                                }
                            }
                        }
                    } else {
                        if !args.entry_types.is_empty() {
                            for etype in &args.entry_types {
                                match etype {
                                    EntryType::Dir => {
                                        if entry.file_type().is_dir() {
                                            println!("{}", entry.path().display());
                                            break;
                                        }
                                    }
                                    EntryType::File => {
                                        if entry.file_type().is_file() {
                                            println!("{}", entry.path().display());
                                            break;
                                        }
                                    }
                                    EntryType::Link => {
                                        if entry.file_type().is_symlink() {
                                            println!("{}", entry.path().display());
                                            break;
                                        }
                                    }
                                }
                            }
                        } else {
                            println!("{}", entry.path().display());
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
