//! Convert HL7 v2.xml XML Schema files into the JSON dictionary the
//! `hl7-2` crates read.
//!
//! HL7 publishes the v2.xml encoding as a set of XML Schema documents, and
//! sites that customise HL7 customise those documents: a hospital's `PID`
//! is the standard's `PID` plus or minus whatever the local system actually
//! sends. `hl7-2` reads a dictionary rather than a schema, which is what
//! lets one build serve every release and every dialect — so this crate is
//! the bridge between the two. Point it at a directory of schemas and it
//! writes the dictionary that describes them.
//!
//! ```no_run
//! use hl7_2_from_xsd_into_json_dictionary::{Options, convert_directory};
//!
//! let document = convert_directory("schemas/paris".as_ref(), &Options::default())?;
//! std::fs::write("paris.json", document.to_json())?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! The dictionary that comes out states cardinality as well as data types,
//! which is what lets `hl7-2-from-er7-into-xml` in schema mode produce a
//! document that validates against the very schemas it was built from. See
//! `spec/index.md` for the conversion rules (source of truth).

// No `unsafe` anywhere in this crate, enforced rather than merely true:
// `forbid` cannot be lifted by an `allow` further down, so this is a
// property a reviewer can rely on without reading the sources. See
// SECURITY.md.
#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]

pub mod dictionary;
pub mod schema;

/// The XML reader this crate is built on, re-exported so a caller can name
/// [`xml::Element`] without adding its own dependency.
pub use hl7_2_xml_lite_helper as xml;

pub use dictionary::{Document, Field, Item};

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// What to put in the document beyond what the schemas say.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// A name for what this dictionary describes, recorded in the
    /// description. The reading crate takes the dictionary's name as an
    /// argument, so this is documentation, not data.
    pub name: Option<String>,
    /// Override the release the base-file prefix implies.
    pub version: Option<String>,
    /// A bundled release to layer this document over, e.g. `2.5`. Without
    /// it the document stands alone, which is what a full set of schemas
    /// justifies.
    pub inherits: Option<String>,
    /// `CODE_TRIGGER` to the structure that carries it, e.g. `ADT_A28` to
    /// `ADT_A05`. Not derivable from the schemas: a schema directory holds
    /// `ADT_A05.xsd` but never says which trigger events arrive as one.
    pub aliases: BTreeMap<String, String>,
    /// Convert only these structures. Empty means every structure schema in
    /// the directory.
    pub structures: Vec<String>,
}

/// Why a directory of schemas could not be read as a dictionary.
#[derive(Debug)]
pub enum Error {
    /// A file could not be read.
    Io(PathBuf, std::io::Error),
    /// A file is not well-formed XML.
    Xml(PathBuf, xml::Error),
    /// No structure schema names a `<prefix>_segments.xsd` to include, so
    /// the base files cannot be found.
    NoPrefix(PathBuf),
    /// A structure schema does not declare the structure its filename names.
    NoStructure(PathBuf, String),
    /// `--structure` named something the directory has no schema for.
    UnknownStructure(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(path, error) => write!(f, "{}: {error}", path.display()),
            Error::Xml(path, error) => write!(f, "{}: {error}", path.display()),
            Error::NoPrefix(path) => write!(
                f,
                "{}: no structure schema includes a '<prefix>_segments.xsd', \
                 so the base schemas cannot be found",
                path.display()
            ),
            Error::NoStructure(path, id) => write!(
                f,
                "{}: declares no '{id}{}' complexType, so it does not describe \
                 the structure its filename names",
                path.display(),
                schema::CONTENT
            ),
            Error::UnknownStructure(id) => write!(f, "no schema for structure {id}"),
        }
    }
}

impl std::error::Error for Error {}

/// Convert a directory of HL7 v2.xml schemas into a dictionary document.
///
/// # Errors
///
/// [`Error`] when the directory cannot be listed or a file cannot be read
/// ([`Error::Io`]), when a file is not well-formed XML ([`Error::Xml`]),
/// when no structure schema names a base prefix to include
/// ([`Error::NoPrefix`]), when a structure schema does not declare the
/// structure its filename names ([`Error::NoStructure`]), or when
/// `options.structures` names one the directory has no schema for
/// ([`Error::UnknownStructure`]).
pub fn convert_directory(directory: &Path, options: &Options) -> Result<Document, Error> {
    let structure_files = structure_files(directory)?;

    let mut prefix = None;
    for path in &structure_files {
        let root = read_xml(path)?;
        if let Some(found) = schema::included_prefix(&root) {
            prefix = Some(found);
            break;
        }
    }
    let Some(prefix) = prefix else {
        return Err(Error::NoPrefix(directory.to_path_buf()));
    };

    let base = directory.join(&prefix);
    let mut types = schema::Types::default();
    types.absorb(&read_xml(&with_suffix(&base, "_types.xsd"))?);
    types.absorb(&read_xml(&with_suffix(&base, "_fields.xsd"))?);
    let segments = schema::segments(&read_xml(&with_suffix(&base, "_segments.xsd"))?, &types);

    let mut structures = BTreeMap::new();
    for path in &structure_files {
        let id = structure_id(path);
        if !options.structures.is_empty() && !options.structures.contains(&id) {
            continue;
        }
        let root = read_xml(path)?;
        let items = schema::structure(&root, &id)
            .ok_or_else(|| Error::NoStructure(path.clone(), id.clone()))?;
        structures.insert(id, items);
    }
    for wanted in &options.structures {
        if !structures.contains_key(wanted) {
            return Err(Error::UnknownStructure(wanted.clone()));
        }
    }

    let source = directory.file_name().map_or_else(
        || directory.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    );
    let mut description = format!(
        "Generated by hl7-2-from-xsd-into-json-dictionary from the HL7 v2.xml \
         schemas in {source}/ (base prefix {prefix}). \
         Edit the schemas and regenerate; do not hand-edit."
    );
    if let Some(name) = &options.name {
        description = format!("{name}: {description}");
    }

    Ok(Document {
        version: Some(
            options
                .version
                .clone()
                .unwrap_or_else(|| version_from_prefix(&prefix)),
        ),
        description: Some(description),
        inherits: options.inherits.clone(),
        types: types.composites(),
        segments,
        aliases: options.aliases.clone(),
        structures,
    })
}

/// `2_5_1` -> `2.5.1`, which is how a dictionary spells a release.
#[must_use]
pub fn version_from_prefix(prefix: &str) -> String {
    prefix.replace('_', ".")
}

/// Every structure schema in a directory, in sorted order.
///
/// A structure file is one that is not a base file, and the base files all
/// end in `_types.xsd`, `_fields.xsd`, or `_segments.xsd`.
fn structure_files(directory: &Path) -> Result<Vec<PathBuf>, Error> {
    let entries =
        std::fs::read_dir(directory).map_err(|error| Error::Io(directory.to_path_buf(), error))?;
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                return false;
            };
            // Both comparisons ignore case: schemas arrive as they were
            // exported, and a vendor shipping `ADT_A05.XSD` should not be
            // silently skipped on a filesystem that does not care either.
            let name = name.to_ascii_lowercase();
            path.extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("xsd"))
                && !["_types.xsd", "_fields.xsd", "_segments.xsd"]
                    .iter()
                    .any(|base| name.ends_with(base))
        })
        .collect();
    paths.sort();
    Ok(paths)
}

fn structure_id(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn with_suffix(base: &Path, suffix: &str) -> PathBuf {
    let mut name = base.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

fn read_xml(path: &Path) -> Result<xml::Element, Error> {
    let text =
        std::fs::read_to_string(path).map_err(|error| Error::Io(path.to_path_buf(), error))?;
    xml::parse(&text).map_err(|error| Error::Xml(path.to_path_buf(), error))
}
