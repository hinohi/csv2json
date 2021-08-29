use std::path::{Path, PathBuf};
use std::str::FromStr;

use structopt::{clap::Shell, StructOpt};

use crate::map::Map;

mod map;

#[derive(Debug, StructOpt)]
#[structopt(name = "csv2json")]
struct Opts {
    #[structopt(name = "CSV", parse(from_os_str), help = "CSV file path")]
    csv: Vec<PathBuf>,
    #[structopt(short, long, help = "By default, it is predicted from the extension")]
    delimiter: Option<String>,
    #[structopt(
        short,
        long,
        help = "Dump JSON Array object instead of Key-Value object"
    )]
    array: bool,
    #[structopt(
        short = "-H",
        long,
        name = "mode",
        possible_values = &EmitHeader::variants(),
        help = "Change emit header mode"
    )]
    header: Option<EmitHeader>,
    #[structopt(
        long,
        name = "shell",
        possible_values = &Shell::variants(),
        help = "Generate tab-completion scripts for your shell"
    )]
    gen_completion: Option<Shell>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum EmitHeader {
    FistFileOnly,
    No,
    Always,
}

impl EmitHeader {
    fn variants() -> [&'static str; 4] {
        ["first-file-only", "ff", "no", "always"]
    }
}

impl FromStr for EmitHeader {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "first-file-only" | "ff" => Ok(EmitHeader::FistFileOnly),
            "no" => Ok(EmitHeader::No),
            "always" => Ok(EmitHeader::Always),
            _ => Err(String::from(s)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum CliError {
    #[error("invalid delimiter: `{0}`")]
    InvalidDelimiter(String),
    #[error("{0}")]
    IOError(#[from] std::io::Error),
    #[error("{0}")]
    CSVError(#[from] csv::Error),
    #[error("{0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

fn main() -> Result<(), CliError> {
    let opts: Opts = Opts::from_args();
    if let Some(shell) = opts.gen_completion {
        completion(shell);
        return Ok(());
    }
    let mut csv = opts.csv;
    if csv.is_empty() {
        csv.push(PathBuf::from("-"));
    }
    run(&csv, opts.delimiter, opts.array, opts.header)
}

fn parse_delimiter(d: String) -> Result<u8, CliError> {
    if &d == "\\t" {
        return Ok(b'\t');
    }
    let b = d.as_bytes();
    if b.len() != 1 {
        return Err(CliError::InvalidDelimiter(d));
    }
    Ok(b[0])
}

fn detect_delimiter(path: &Path, delimiter: Option<u8>) -> u8 {
    delimiter.unwrap_or_else(|| {
        if let Some(ext) = path.extension() {
            match ext.to_ascii_lowercase().to_string_lossy().as_ref() {
                "csv" => b',',
                "tsv" => b'\t',
                _ => b',',
            }
        } else {
            b','
        }
    })
}

fn run(
    csv: &[PathBuf],
    delimiter: Option<String>,
    array: bool,
    header: Option<EmitHeader>,
) -> Result<(), CliError> {
    let delimiter = if let Some(d) = delimiter {
        Some(parse_delimiter(d)?)
    } else {
        None
    };
    let stdout = std::io::stdout();
    let mut writer = stdout.lock();
    let header = header.unwrap_or_else(|| {
        if array {
            EmitHeader::Always
        } else {
            EmitHeader::No
        }
    });
    for (i, path) in csv.iter().enumerate() {
        let delimiter = detect_delimiter(path, delimiter);
        let reader: Box<dyn std::io::Read> = if path.to_str() == Some("-") {
            Box::new(std::io::stdin())
        } else {
            Box::new(std::fs::File::open(path)?)
        };
        let csv_reader = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(false)
            .from_reader(reader);
        let header = match header {
            EmitHeader::FistFileOnly => i == 0,
            EmitHeader::No => false,
            EmitHeader::Always => true,
        };
        pipe(csv_reader, &mut writer, array, header)?
    }
    Ok(())
}

fn pipe<R, W>(
    mut reader: csv::Reader<R>,
    writer: &mut W,
    array: bool,
    header: bool,
) -> Result<(), CliError>
where
    R: std::io::Read,
    W: std::io::Write,
{
    let mut records = reader.records();
    let first_record = match records.next() {
        Some(first_record) => first_record?,
        None => return Ok(()),
    };
    let first_record = first_record.iter().collect::<Vec<_>>();
    if array {
        if header {
            serde_json::to_writer(&mut *writer, &first_record)?;
            writeln!(writer)?;
        }
        for record in records {
            let record = record?;
            let record = record.iter().collect::<Vec<_>>();
            serde_json::to_writer(&mut *writer, &record)?;
            writeln!(writer)?;
        }
    } else {
        if header {
            let map = Map::new(&first_record, &first_record);
            serde_json::to_writer(&mut *writer, &map)?;
            writeln!(writer)?;
        }
        for record in records {
            let record = record?;
            let record = record.iter().collect::<Vec<_>>();
            let map = Map::new(&first_record, &record);
            serde_json::to_writer(&mut *writer, &map)?;
            writeln!(writer)?;
        }
    }
    Ok(())
}

fn completion(shell: Shell) {
    Opts::clap().gen_completions_to(env!("CARGO_BIN_NAME"), shell, &mut std::io::stdout())
}
