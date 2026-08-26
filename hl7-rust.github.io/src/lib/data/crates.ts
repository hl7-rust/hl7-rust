/**
 * The crate catalog.
 *
 * One entry per Cargo workspace member of https://github.com/hl7-rust/hl7-rust.
 * Everything on this site that lists, links to, or describes a crate reads it
 * from here, so a new crate is one entry rather than a search-and-replace.
 *
 * `version` is the version published on crates.io at the time of writing; it is
 * shown as "latest known", never as a version constraint to copy.
 */

export type CrateCategory = 'core' | 'transport' | 'conversion' | 'tooling';

export interface Crate {
  /** Cargo package name, which is also the directory name in the workspace. */
  name: string;
  /** Rust module path (`hl7_2`), for prose that names the `use` line. */
  ident: string;
  /** URL slug under /crates/. Same as `name`. */
  slug: string;
  /** One line, sentence case, no trailing period. */
  tagline: string;
  /** A paragraph: what it is and when you would reach for it. */
  summary: string;
  category: CrateCategory;
  version: string;
  /** Has a normative spec/index.md. */
  spec: boolean;
  /** Installed binary name, if the crate ships one. */
  binary?: string;
  /** Cargo features, in declaration order. */
  features?: { name: string; default: boolean; effect: string }[];
  /** Direct dependencies, as the crate's own README states them. */
  dependencies: string[];
  /** Crates in this catalog that are worth reading next. */
  related: string[];
}

export const CATEGORIES: Record<CrateCategory, { label: string; blurb: string }> = {
  core: {
    label: 'Core',
    blurb: 'The standards themselves, and the umbrella crate that re-exports them.'
  },
  transport: {
    label: 'Transports',
    blurb: 'Getting messages across a network, and answering them.'
  },
  conversion: {
    label: 'Format conversions',
    blurb: 'ER7 in both directions against both target formats.'
  },
  tooling: {
    label: 'Tooling and helpers',
    blurb: 'The macros, the dictionary builder, and the shared XML reader.'
  }
};

export const CRATES: Crate[] = [
  {
    name: 'hl7',
    ident: 'hl7',
    slug: 'hl7',
    tagline: 'The umbrella crate: one module per HL7 standard',
    summary:
      'A thin re-export and nothing else: `hl7::v2` is the `hl7-2` crate and `hl7::v3` is `hl7-3`, with room left for `hl7::fhir`. Nothing lives at the root, because a "message", a "segment", and a "code" mean different things in each standard and one flat namespace would only invite mixing them up.',
    category: 'core',
    version: '0.1.2',
    spec: false,
    features: [
      { name: 'derive', default: false, effect: 'Forwards to `hl7-2`’s `derive` feature.' }
    ],
    dependencies: ['hl7-2', 'hl7-3'],
    related: ['hl7-2', 'hl7-3']
  },
  {
    name: 'hl7-2',
    ident: 'hl7_2',
    slug: 'hl7-2',
    tagline: 'HL7 v2 itself: parse, navigate, validate, modify, render',
    summary:
      'Releases 2.1 through 2.9, in three modes that share one set of internals — generic for the vendor you have never seen, schema-based for the vendor whose format is not frozen, struct-based for the feed that does not change. Also a command-line tool. This is what everything else in the workspace builds on.',
    category: 'core',
    version: '0.2.4',
    spec: true,
    binary: 'hl7-v2',
    features: [
      {
        name: 'derive',
        default: false,
        effect: 'Adds `#[derive(FromHl7)]` and `#[derive(ToHl7)]`; pulls in `hl7-2-derive`.'
      }
    ],
    dependencies: ['er7'],
    related: ['hl7', 'hl7-2-derive', 'hl7-2-mllp', 'hl7-2-from-er7-into-xml']
  },
  {
    name: 'hl7-3',
    ident: 'hl7_3',
    slug: 'hl7-3',
    tagline: 'HL7 v3: the RIM backbone, the data types, the message envelope',
    summary:
      'The Reference Information Model backbone classes, the data types they are built from, and the three-level message envelope — a foundation, not a complete implementation. It is the part of v3 that is the same in every domain, which is what CDA and national registries such as NHS England’s Personal Demographics Service are built on.',
    category: 'core',
    version: '0.1.4',
    spec: true,
    features: [
      { name: 'derive', default: false, effect: 'Adds `#[derive(FromElement)]`; pulls in `hl7-3-derive`.' }
    ],
    dependencies: ['hl7-2-xml-lite-helper'],
    related: ['hl7', 'hl7-3-derive', 'hl7-3-soap', 'hl7-2-xml-lite-helper']
  },
  {
    name: 'hl7-2-mllp',
    ident: 'hl7_2_mllp',
    slug: 'hl7-2-mllp',
    tagline: 'MLLP: HL7 v2 framed on a TCP stream',
    summary:
      'The Minimal Lower Layer Protocol is three bytes of framing and nothing more. This crate is what people actually need on top of it: whole messages out of a chopped-up stream, an acknowledgement that names the message it answers, and a bound on what a broken peer can allocate.',
    category: 'transport',
    version: '0.1.4',
    spec: true,
    features: [
      { name: 'ack', default: true, effect: 'Acknowledgement generation; pulls in `hl7-2`.' },
      { name: 'clock', default: false, effect: '`acknowledge_now`; pulls in `chrono`. Implies `ack`.' },
      { name: 'noncompliance', default: false, effect: 'The default framing tolerance becomes lenient.' }
    ],
    dependencies: ['hl7-2 (feature `ack`)', 'chrono (feature `clock`)'],
    related: ['hl7-2', 'hl7-2-soap']
  },
  {
    name: 'hl7-2-soap',
    ident: 'hl7_2_soap',
    slug: 'hl7-2-soap',
    tagline: 'HL7 v2 carried in a SOAP envelope over HTTP',
    summary:
      'The other v2 transport — what an estate ends up with when messages have to cross a boundary that speaks HTTP, or when the far end was built by a team who had a WSDL and no socket. The envelope, faults that carry their HTTP status, payload carriage, WSDL, and response evaluation. No HTTP client and no HTTP server.',
    category: 'transport',
    version: '0.1.2',
    spec: true,
    dependencies: ['hl7-2-xml-lite-helper'],
    related: ['hl7-2', 'hl7-2-mllp', 'hl7-3-soap']
  },
  {
    name: 'hl7-3-soap',
    ident: 'hl7_3_soap',
    slug: 'hl7-3-soap',
    tagline: 'HL7 v3 carried in a SOAP envelope over HTTP',
    summary:
      'For v3, SOAP is not the alternative transport — it is the transport. v3 was designed alongside SOAP/WS-*, and real deployments carry it that way. Same shape as its `hl7-2-soap` cousin, plus the real HL7 v3 acknowledgement (`MCCI_IN000002UV01`).',
    category: 'transport',
    version: '0.1.2',
    spec: true,
    dependencies: ['hl7-2-xml-lite-helper'],
    related: ['hl7-3', 'hl7-2-soap']
  },
  {
    name: 'hl7-2-from-er7-into-xml',
    ident: 'hl7_2_from_er7_into_xml',
    slug: 'hl7-2-from-er7-into-xml',
    tagline: 'ER7 → the official v2.xml XML representation',
    summary:
      'Pipe-delimited ER7 into `urn:hl7-org:v2xml`, with components named after their HL7 v2.5 data types. Since 0.5.0 the tables and grammars come from the `hl7-2` dictionary, which is what lets `--dictionary` convert against a vendor’s own XML Schema instead of the bundled v2.5 release.',
    category: 'conversion',
    version: '0.6.1',
    spec: true,
    binary: 'hl7-2-from-er7-into-xml',
    dependencies: ['er7', 'hl7-2'],
    related: ['hl7-2-from-xml-into-er7', 'hl7-2-from-er7-into-json', 'hl7-2-from-xsd-into-json-dictionary']
  },
  {
    name: 'hl7-2-from-xml-into-er7',
    ident: 'hl7_2_from_xml_into_er7',
    slug: 'hl7-2-from-xml-into-er7',
    tagline: 'v2.xml XML → ER7',
    summary:
      'The inverse of the crate above, and it needs no HL7 v2.5 dictionary at all: every element name the forward crate writes already carries its own position as the number after the last dot, so reversal is a purely structural rebuild.',
    category: 'conversion',
    version: '0.6.1',
    spec: true,
    binary: 'hl7-2-from-xml-into-er7',
    dependencies: ['er7', 'hl7-2-xml-lite-helper'],
    related: ['hl7-2-from-er7-into-xml', 'hl7-2-from-json-into-er7', 'hl7-2-xml-lite-helper']
  },
  {
    name: 'hl7-2-from-er7-into-json',
    ident: 'hl7_2_from_er7_into_json',
    slug: 'hl7-2-from-er7-into-json',
    tagline: 'ER7 → typed JSON',
    summary:
      'There is no official "v2.json" to target, so this crate defines its own mapping — designed to preserve everything v2.xml preserves while using idiomatic JSON (real arrays, real `null`) instead of XML’s constructs. Same ER7 parser, same data-type tables, and same grammars as the XML sibling.',
    category: 'conversion',
    version: '0.4.3',
    spec: true,
    binary: 'hl7-2-from-er7-into-json',
    dependencies: ['er7'],
    related: ['hl7-2-from-json-into-er7', 'hl7-2-from-er7-into-xml']
  },
  {
    name: 'hl7-2-from-json-into-er7',
    ident: 'hl7_2_from_json_into_er7',
    slug: 'hl7-2-from-json-into-er7',
    tagline: 'Typed JSON → ER7',
    summary:
      'The inverse of the JSON forward crate, on the same position-carries-the-name principle as the XML reverse crate, plus a minimal dependency-free JSON reader covering the full RFC 8259 grammar.',
    category: 'conversion',
    version: '0.4.3',
    spec: true,
    binary: 'hl7-2-from-json-into-er7',
    dependencies: ['er7'],
    related: ['hl7-2-from-er7-into-json', 'hl7-2-from-xml-into-er7']
  },
  {
    name: 'hl7-2-derive',
    ident: 'hl7_2_derive',
    slug: 'hl7-2-derive',
    tagline: '#[derive(FromHl7)] and #[derive(ToHl7)] for struct mode',
    summary:
      'Map a struct’s fields to HL7 v2 message paths once, in the type definition, instead of writing the same accessor calls at every call site. You do not depend on this crate directly — it arrives through `hl7-2`’s `derive` feature, which is what keeps the default `hl7-2` build to exactly one dependency.',
    category: 'tooling',
    version: '0.1.4',
    spec: false,
    dependencies: ['syn', 'quote'],
    related: ['hl7-2', 'hl7-3-derive']
  },
  {
    name: 'hl7-3-derive',
    ident: 'hl7_3_derive',
    slug: 'hl7-3-derive',
    tagline: '#[derive(FromElement)] for HL7 v3 struct mode',
    summary:
      'Maps struct fields to XML element attributes and children. Like its v2 cousin it arrives through a feature rather than a direct dependency — and unlike it, nothing here returns a `Result`: a missing attribute reads as that field’s `Default`, matching `hl7-3`’s own degrade-don’t-reject choice.',
    category: 'tooling',
    version: '0.1.2',
    spec: false,
    dependencies: ['syn', 'quote'],
    related: ['hl7-3', 'hl7-2-derive']
  },
  {
    name: 'hl7-2-from-xsd-into-json-dictionary',
    ident: 'hl7_2_from_xsd_into_json_dictionary',
    slug: 'hl7-2-from-xsd-into-json-dictionary',
    tagline: 'HL7 v2.xml XSDs → the JSON dictionary hl7-2 reads',
    summary:
      'HL7 publishes v2.xml as schemas; `hl7-2` reads a dictionary, because one dictionary format serves every release and every local dialect from one build. This crate is the bridge, and the dictionary it writes carries cardinality as well as data types — which is what lets schema mode emit a document that validates against the schemas it came from.',
    category: 'tooling',
    version: '0.1.2',
    spec: true,
    binary: 'hl7-2-from-xsd-into-json-dictionary',
    dependencies: ['hl7-2-xml-lite-helper'],
    related: ['hl7-2', 'hl7-2-from-er7-into-xml', 'hl7-2-xml-lite-helper']
  },
  {
    name: 'hl7-2-xml-lite-helper',
    ident: 'hl7_2_xml_lite_helper',
    slug: 'hl7-2-xml-lite-helper',
    tagline: 'The small, dependency-free XML reader this family shares',
    summary:
      'Elements, attributes, text, and nesting, for documents whose shape you already know. It exists because three crates in this family had each written their own version of exactly this, and three copies of a parser is three places for a bug. Namespace prefixes are ignored rather than resolved — deliberately.',
    category: 'tooling',
    version: '0.1.2',
    spec: true,
    dependencies: [],
    related: ['hl7-2-soap', 'hl7-3-soap', 'hl7-2-from-xml-into-er7', 'hl7-2-from-xsd-into-json-dictionary']
  }
];

export const ORG = 'https://github.com/hl7-rust';
export const REPO = 'https://github.com/hl7-rust/hl7-rust';

export function crateBySlug(slug: string): Crate {
  const crate = CRATES.find((candidate) => candidate.slug === slug);
  if (!crate) throw new Error(`no crate named ${slug}`);
  return crate;
}

export function cratesIn(category: CrateCategory): Crate[] {
  return CRATES.filter((crate) => crate.category === category);
}

/** The crate's directory in the monorepo. */
export function repoUrl(crate: Crate): string {
  return `${REPO}/tree/main/${crate.name}`;
}

/** The crate's normative specification, when it has one. */
export function specUrl(crate: Crate): string | undefined {
  return crate.spec ? `${REPO}/blob/main/${crate.name}/spec/index.md` : undefined;
}

export function cratesIoUrl(crate: Crate): string {
  return `https://crates.io/crates/${crate.name}`;
}

export function docsRsUrl(crate: Crate): string {
  return `https://docs.rs/${crate.name}`;
}
