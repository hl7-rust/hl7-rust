//! Command line: read a message and print what it means.
//!
//! The library's three modes are all reachable here — a tree for
//! exploring, a path query for extracting, a schema for a vendor dialect,
//! and validation for checking — because the first thing anyone does with
//! an unfamiliar HL7 message is look at it, and that should not require
//! writing a program.

use hl7_2::generic::Node;
use hl7_2::{Dictionary, Message, Options, Severity, Version};
use std::io::{Read, Write};
use std::process::ExitCode;
use std::sync::Arc;

const USAGE: &str = "\
hl7-v2 — read, query, check, and modify HL7 v2 messages

Usage: hl7-v2 [OPTIONS] [FILE]

Reads FILE, or standard input when FILE is absent or `-`. Input may hold one
message, several, or an HL7 batch file; each message is handled separately.

Output (the first one given wins; the default is --tree):
  -t, --tree             print the message as an indented tree
  -q, --query PATH       print the value(s) at PATH, one per line
  -c, --check            print validation diagnostics
  -e, --er7              print the message back as ER7

Options:
  -s, --set PATH=VALUE   set a value before printing; may be repeated
  -n, --null PATH        set a value to the HL7 explicit null \"\"
  -v, --hl7-version VER  read as this HL7 release (2.1 ... 2.9) whatever
                         MSH-12 says
  -d, --dictionary FILE  read through this JSON dictionary (schema mode)
  -f, --flat             do not group segments into the message structure
  -p, --paths            show each node's path in tree output
  -S, --strict           fail on any validation error
  -o, --output FILE      write to FILE instead of standard output
  -h, --help             print this help
  -V, --version          print the crate version

Exit status is 0 on success, 1 on a usage or parse error, and 2 when
--check or --strict found something wrong with the message.
";

/// Which of the four outputs to print.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Output {
    Tree,
    Query(String),
    Check,
    Er7,
}

/// One edit to apply before printing.
#[derive(Debug)]
enum Edit {
    Set(String, String),
    Null(String),
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("hl7-v2: {message}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let mut arguments = std::env::args().skip(1);
    let mut output = None;
    let mut edits: Vec<Edit> = Vec::new();
    let mut options = Options::new();
    let mut dictionary_path = None;
    let mut input_path = None;
    let mut output_path = None;
    let mut flat = false;
    let mut show_paths = false;

    while let Some(argument) = arguments.next() {
        let mut want = |what: &str| {
            arguments
                .next()
                .ok_or_else(|| format!("{argument} needs {what}"))
        };
        match argument.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return Ok(ExitCode::SUCCESS);
            }
            "-V" | "--version" => {
                println!("hl7-v2 {}", env!("CARGO_PKG_VERSION"));
                return Ok(ExitCode::SUCCESS);
            }
            "-t" | "--tree" => output.get_or_insert(Output::Tree),
            "-c" | "--check" => output.get_or_insert(Output::Check),
            "-e" | "--er7" => output.get_or_insert(Output::Er7),
            "-q" | "--query" => output.get_or_insert(Output::Query(want("a path")?)),
            "-f" | "--flat" => {
                flat = true;
                continue;
            }
            "-p" | "--paths" => {
                show_paths = true;
                continue;
            }
            "-S" | "--strict" => {
                options.strict = true;
                continue;
            }
            "-s" | "--set" => {
                let assignment = want("PATH=VALUE")?;
                let (path, value) = assignment
                    .split_once('=')
                    .ok_or_else(|| format!("--set needs PATH=VALUE, got {assignment:?}"))?;
                edits.push(Edit::Set(path.to_string(), value.to_string()));
                continue;
            }
            "-n" | "--null" => {
                edits.push(Edit::Null(want("a path")?));
                continue;
            }
            "-v" | "--hl7-version" => {
                let text = want("an HL7 version")?;
                let version: Version = text.parse().map_err(|error| format!("{error}"))?;
                options.version = Some(version);
                continue;
            }
            "-d" | "--dictionary" => {
                dictionary_path = Some(want("a file")?);
                continue;
            }
            "-o" | "--output" => {
                output_path = Some(want("a file")?);
                continue;
            }
            "-" => {
                input_path = None;
                continue;
            }
            other if other.starts_with('-') && other.len() > 1 => {
                return Err(format!("unknown option {other:?}; try --help"));
            }
            _ => {
                if input_path.is_some() {
                    return Err("only one input file may be given".to_string());
                }
                input_path = Some(argument);
                continue;
            }
        };
    }

    if let Some(path) = dictionary_path {
        let text = std::fs::read_to_string(&path)
            .map_err(|error| format!("cannot read dictionary {path}: {error}"))?;
        let dictionary = Dictionary::from_json(&text, &path)
            .map_err(|error| format!("cannot load dictionary {path}: {error}"))?;
        options.dictionary = Some(Arc::new(dictionary));
    }

    let input = match &input_path {
        Some(path) => {
            std::fs::read_to_string(path).map_err(|error| format!("cannot read {path}: {error}"))?
        }
        None => {
            let mut text = String::new();
            std::io::stdin()
                .read_to_string(&mut text)
                .map_err(|error| format!("cannot read standard input: {error}"))?;
            text
        }
    };

    let output = output.unwrap_or(Output::Tree);
    let mut text = String::new();
    let mut found_problems = false;
    let messages = hl7_2::split_messages(&input);
    if messages.is_empty() {
        return Err("input contains no HL7 messages".to_string());
    }
    for (index, one) in messages.iter().enumerate() {
        let mut message = hl7_2::parse_with_options(one, &options).map_err(|error| {
            found_problems = true;
            format!("message {}: {error}", index + 1)
        })?;
        for edit in &edits {
            match edit {
                Edit::Set(path, value) => message.set(path, value),
                Edit::Null(path) => message.set_null(path),
            }
            .map_err(|error| format!("message {}: {error}", index + 1))?;
        }
        // A blank line keeps successive trees apart; the line-per-value
        // and ER7 outputs are already self-delimiting, and a caller piping
        // them into another tool does not want blank lines in the stream.
        if index > 0 && output == Output::Tree {
            text.push('\n');
        }
        found_problems |= render(&message, &output, flat, show_paths, &mut text)?;
    }

    match output_path {
        Some(path) => {
            std::fs::write(&path, &text).map_err(|error| format!("cannot write {path}: {error}"))?
        }
        None => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            stdout
                .write_all(text.as_bytes())
                .and_then(|()| stdout.flush())
                .map_err(|error| format!("cannot write output: {error}"))?;
        }
    }
    Ok(if found_problems {
        ExitCode::from(2)
    } else {
        ExitCode::SUCCESS
    })
}

/// Append one message's output, reporting whether anything was found wrong.
fn render(
    message: &Message,
    output: &Output,
    flat: bool,
    show_paths: bool,
    text: &mut String,
) -> Result<bool, String> {
    match output {
        Output::Tree => {
            write_node(&message.tree_with_options(!flat), 0, show_paths, text);
            Ok(false)
        }
        Output::Er7 => {
            text.push_str(&message.to_er7());
            text.push('\n');
            Ok(false)
        }
        Output::Query(path) => {
            let values = message
                .get_all(path)
                .map_err(|error| format!("{path}: {error}"))?;
            for value in values {
                text.push_str(&value);
                text.push('\n');
            }
            Ok(false)
        }
        Output::Check => {
            let diagnostics = message.validate();
            let failed = diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == Severity::Error);
            if diagnostics.is_empty() {
                text.push_str("ok\n");
            }
            for diagnostic in diagnostics {
                text.push_str(&diagnostic.to_string());
                text.push('\n');
            }
            Ok(failed)
        }
    }
}

/// One node and everything under it, two spaces per level.
fn write_node(node: &Node, depth: usize, show_paths: bool, text: &mut String) {
    text.push_str(&"  ".repeat(depth));
    text.push_str(node.name());
    if node.is_leaf() {
        text.push_str(" = ");
        text.push_str(if node.is_null() { "\"\"" } else { node.text() });
    }
    if show_paths && !node.path().is_empty() {
        text.push_str("  [");
        text.push_str(node.path());
        text.push(']');
    }
    text.push('\n');
    for child in node.children() {
        write_node(child, depth + 1, show_paths, text);
    }
}
