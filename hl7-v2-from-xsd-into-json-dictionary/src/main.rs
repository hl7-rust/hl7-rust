use hl7_v2_from_xsd_into_json_dictionary::{Options, convert_directory};
use std::io::Write;
use std::process::ExitCode;

const USAGE: &str = "\
Convert HL7 v2.xml XML Schema files into the JSON dictionary hl7-2 reads.

Usage: hl7-v2-from-xsd-into-json-dictionary [OPTIONS] <DIRECTORY>

Arguments:
  <DIRECTORY>  Directory holding <prefix>_types.xsd, <prefix>_fields.xsd,
               <prefix>_segments.xsd, and one schema per message structure

Options:
  -o, --output <FILE>            Write JSON to FILE instead of stdout
      --name <NAME>              What this dictionary describes, for its description
      --version-id <VERSION>     Override the release the base-file prefix implies
      --inherits <RELEASE>       Layer this document over a bundled release, e.g. 2.5
      --alias <CODE_TRIGGER=ID>  A message type carried by another structure;
                                 repeatable, e.g. --alias ADT_A28=ADT_A05
      --structure <ID>           Convert only this structure; repeatable,
                                 default is every structure schema present
  -h, --help                     Print help
  -V, --version                  Print version";

fn main() -> ExitCode {
    let mut options = Options::default();
    let mut directory: Option<String> = None;
    let mut output: Option<String> = None;
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            "-V" | "--version" => {
                println!(
                    "hl7-v2-from-xsd-into-json-dictionary {}",
                    env!("CARGO_PKG_VERSION")
                );
                return ExitCode::SUCCESS;
            }
            "-o" | "--output" => match args.next() {
                Some(value) => output = Some(value),
                None => return fail("missing value for --output"),
            },
            "--name" => match args.next() {
                Some(value) => options.name = Some(value),
                None => return fail("missing value for --name"),
            },
            "--version-id" => match args.next() {
                Some(value) => options.version = Some(value),
                None => return fail("missing value for --version-id"),
            },
            "--inherits" => match args.next() {
                Some(value) => options.inherits = Some(value),
                None => return fail("missing value for --inherits"),
            },
            "--structure" => match args.next() {
                Some(value) => options.structures.push(value),
                None => return fail("missing value for --structure"),
            },
            "--alias" => match args.next() {
                Some(value) => match value.split_once('=') {
                    Some((from, to)) if !from.is_empty() && !to.is_empty() => {
                        options.aliases.insert(from.to_string(), to.to_string());
                    }
                    _ => return fail(&format!("expected CODE_TRIGGER=STRUCTURE, got {value:?}")),
                },
                None => return fail("missing value for --alias"),
            },
            _ if arg.starts_with('-') && arg != "-" => {
                return fail(&format!("unknown option: {arg}"));
            }
            _ => {
                if directory.is_some() {
                    return fail("more than one directory given");
                }
                directory = Some(arg);
            }
        }
    }

    let Some(directory) = directory else {
        return fail("no schema directory given (try --help)");
    };

    let document = match convert_directory(directory.as_ref(), &options) {
        Ok(document) => document,
        Err(error) => return fail(&error.to_string()),
    };
    let text = document.to_json();

    match output {
        Some(path) => {
            if let Err(error) = std::fs::write(&path, text) {
                return fail(&format!("writing {path}: {error}"));
            }
        }
        None => {
            if let Err(error) = std::io::stdout().write_all(text.as_bytes()) {
                return fail(&format!("writing stdout: {error}"));
            }
        }
    }
    ExitCode::SUCCESS
}

fn fail(message: &str) -> ExitCode {
    eprintln!("hl7-v2-from-xsd-into-json-dictionary: {message}");
    ExitCode::FAILURE
}
