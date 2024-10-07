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
    let included = |etypes: &Vec<EntryType>, entry: &DirEntry| {
        etypes.is_empty()
            || etypes.iter().any(|etypes| match etypes {
                EntryType::Link => entry.file_type().is_symlink(),
                EntryType::Dir => entry.file_type().is_dir(),
                EntryType::File => entry.file_type().is_file(),
            })
    };

    let matched = |names: &Vec<Regex>, entry: &DirEntry| {
        names.is_empty()
            || names
                .iter()
                .any(|name| name.is_match(&entry.file_name().to_string_lossy()))
    };

    for path in args.paths {
        for entry in WalkDir::new(path) {
            match entry {
                Err(e) => eprintln!("{}", e),
                Ok(entry) => {
                    if matched(&args.names, &entry) {
                        if included(&args.entry_types, &entry) {
                            println!("{}", entry.path().display());
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
