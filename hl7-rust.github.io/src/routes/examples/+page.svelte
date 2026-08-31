<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';

  const contents = [
    { id: 'reading', label: 'Reading a message' },
    { id: 'writing', label: 'Writing and answering' },
    { id: 'dialects', label: 'Dialects and dictionaries' },
    { id: 'typed', label: 'Typed structs' },
    { id: 'converting', label: 'Converting formats' },
    { id: 'transports', label: 'Transports' },
    { id: 'v3', label: 'HL7 v3' },
    { id: 'shell', label: 'Shell one-liners' },
    { id: 'runnable', label: 'Runnable examples in the repository' }
  ];

  const parse = `use hl7::v2;

let message = v2::parse(text)?;
assert_eq!(message.structure_id(), "ORU_R01");`;

  const forceVersion = `use hl7_2::Version;

let options = hl7_2::Options::new().with_version(Version::V2_3);
let message = hl7_2::parse_with_options(text, &options)?;`;

  const getValue = `let family = message.get("PID-5.1")?;            // Option<String>
let second = message.get("OBX[2]-5.2")?;        // the second OBX
let deep   = message.get("PID-3.4.2")?;         // down to a subcomponent`;

  const everyValue = `let all = message.get_all("OBX-5")?;             // every OBX's field 5
let reps = message.repetitions("PID-3")?;       // every repetition of one field`;

  const walk = `for observation in message.tree().find_all("OBX") {
    println!("{} = {}", observation.path(), observation.text());
}`;

  const typeOf = `// What does the dictionary say this field is?
assert_eq!(message.type_of("PID-3")?.as_deref(), Some("CX"));`;

  const batchRead = `for text in hl7_2::split_messages(batch) {
    match hl7_2::parse(&text) {
        Ok(message) => handle(&message),
        Err(error) => eprintln!("skipping: {error}"),
    }
}`;

  const setValue = `let mut message = hl7_2::parse(text)?;
message.set("PID-5.2", "EVELYN")?;       // delimiters in the value are escaped
message.set_null("PID-11")?;             // the explicit HL7 null
message.clear("PID-12")?;                // remove the value entirely`;

  const addSegment = `message.append_segment("NTE");
message.set("NTE[2]-1", "2")?;
message.set("NTE[2]-3", "Amended after review.")?;
let er7 = message.to_er7();`;

  const buildMessage = `let message = hl7_2::Builder::new(hl7_2::Version::V2_5)
    .message_type("ADT", "A01")
    .control_id("MSG00042")
    .timestamp("20260814080000")
    .sending("MYAPP", "MYFACILITY")
    .receiving("EHR", "CLINIC")
    .processing_id("P")
    .segment("PID")
    .set("PID-5.1", "EVERYWOMAN")
    .build_valid()?;`;

  const ackExample = `let ack = hl7_2::builder::acknowledge(&message, "AA", "ACK00001", "20260814080100")
    .build_valid()?;
assert_eq!(ack.get("MSA-2")?.as_deref(), Some("MSG00042"));`;

  const validate = `for diagnostic in message.validate() {
    println!("{diagnostic}");
}

let clean = message
    .validate()
    .iter()
    .all(|d| d.severity != hl7_2::Severity::Error);`;

  const strict = `let options = hl7_2::Options::new().strict();
match hl7_2::parse_with_options(text, &options) {
    Ok(message) => { /* conformant */ }
    Err(hl7_2::Error::Invalid(diagnostics)) => { /* every error-level finding */ }
    Err(other) => { /* not a message at all */ }
}`;

  const dictionaryInline = `use std::sync::Arc;

let dictionary = hl7_2::Dictionary::from_json(r#"{
  "inherits": "2.5",
  "segments": { "ZAC": ["SI", "XPN", "DT"] }
}"#, "acme")?;

let options = hl7_2::Options::new().with_dictionary(Arc::new(dictionary));
let message = hl7_2::parse_with_options(text, &options)?;`;

  const dictionaryFile = `let text = std::fs::read_to_string("acme.json")?;
let dictionary = hl7_2::Dictionary::from_json(&text, "acme")?;

assert_eq!(dictionary.field_type("PID", 3), Some("CX"));
assert!(dictionary.field_cardinality("PID", 3).repeats);`;

  const dictionaryBuild = `use hl7_2_from_xsd_into_json_dictionary::{Options, convert_directory};

let document = convert_directory("schemas/paris".as_ref(), &Options::default())?;
std::fs::write("paris.json", document.to_json())?;`;

  const structRead = `use hl7_2::{FromHl7, Raw};

#[derive(FromHl7)]
struct Admission {
    #[hl7("PID-3.1")]  patient_id: String,
    #[hl7("PID-7.1")]  birth_date: Option<String>,
    #[hl7("PID-3")]    all_identifiers: Vec<String>,
    #[hl7(raw)]        raw: Raw,
}

let admission: Admission = hl7_2::parse(text)?.decode()?;
assert_eq!(admission.raw.get("ZPD-1")?.as_deref(), Some("local"));`;

  const structWrite = `use hl7_2::{FromHl7, ToHl7};

#[derive(FromHl7, ToHl7)]
struct Patient {
    #[hl7("PID-5.1")] family: String,
    #[hl7("PID-8")]   sex: Option<String>,
}

let message = hl7_2::Builder::new(hl7_2::Version::V2_5)
    .message_type("ADT", "A01")
    .control_id("1")
    .segment("PID")
    .encode(&patient)
    .build_valid()?;`;

  const toXml = `let xml = hl7_2_from_er7_into_xml::convert(er7)?;
let json = hl7_2_from_er7_into_json::convert(er7)?;`;

  const convertOptions = `use hl7_2_from_er7_into_json::{Options, convert_with_options};

let json = convert_with_options(er7, Options { flat: true, compact: true })?;`;

  const fromXml = `let er7 = hl7_2_from_xml_into_er7::convert(xml)?;
let er7 = hl7_2_from_json_into_er7::convert(json)?;

// Or keep the message, to query or edit it:
let message = hl7_2_from_xml_into_er7::parse(xml)?;
assert_eq!(message.query("PID-5.1")?.as_deref(), Some("TEST"));`;

  const mllpFrame = `use hl7_2_mllp as mllp;

let frame = mllp::encode(message.as_bytes());
assert_eq!(mllp::decode(&frame)?, message.as_bytes());`;

  const mllpStream = `use hl7_2_mllp::Framer;

let mut framer = Framer::new();
framer.push(b"\\x0bMSH|one\\x1c\\r\\x0bMSH|t");
framer.push(b"wo\\x1c\\r");

assert_eq!(framer.next_frame()?.unwrap(), b"MSH|one");
assert_eq!(framer.next_frame()?.unwrap(), b"MSH|two");
assert_eq!(framer.next_frame()?, None);`;

  const mllpServe = `use hl7_2_mllp::{IoTransport, Transport};
use std::net::TcpListener;

let listener = TcpListener::bind("127.0.0.1:2575")?;
for stream in listener.incoming() {
    let mut transport = IoTransport::new(stream?);
    while let Some(message) = transport.receive()? {
        // ... one whole HL7 message ...
    }
}`;

  const soapReceive = `use hl7_2_soap::{Fault, message};

fn accept(request_body: &str) -> Result<String, Fault> {
    let envelope = hl7_2_soap::parse(request_body)?;
    let payload = envelope.payload()?;
    message::check(payload, &["ADT_A05".to_string()], &[])?;
    Ok(message::control_id(payload).unwrap_or_default().to_string())
}`;

  const soapSend = `use hl7_2_soap::{message, response::{self, Outcome}};

let body = message::wrap_er7("MSH|^~\\\\&|APP||||1||ADT^A01|9|P|2.5");
match response::evaluate(status, &reply) {
    Outcome::Accepted => {}
    Outcome::Rejected(reason) => eprintln!("not delivered: {reason}"),
}`;

  const v3Envelope = `use hl7_3::message;

let parsed = message::parse(xml)?;
let observation = parsed.control_act.unwrap().domain.unwrap();
let act = hl7_3::rim::Act::from_element(&observation);
assert_eq!(act.class_code, "OBS");`;

  const v3Types = `use hl7_3::{Ivl, NullFlavor, Pq};

let dose = hl7_3::xml::parse(r#"<doseQuantity value="5" unit="mg"/>"#)?;
assert_eq!(Pq::from_element(&dose).unit.as_deref(), Some("mg"));

let value = hl7_3::xml::parse(r#"<value nullFlavor="ASKU"/>"#)?;
assert_eq!(NullFlavor::of(&value), Some(NullFlavor::AskedButUnknown));`;

  const xmlHelper = `let xml = r#"<order id="7"><item qty="2">widget</item></order>"#;
let root = hl7_2_xml_lite_helper::parse(xml)?;

assert_eq!(root.attribute("id"), Some("7"));
assert_eq!(root.child("item").unwrap().text, "widget");
assert_eq!(root.find("item").unwrap().text, "widget");   // first descendant`;

  const shell = `# Look at an unfamiliar message
hl7-v2 --paths message.hl7

# Every result value in a directory
cat inbox/*.hl7 | hl7-v2 --query OBX-5

# Check, with an exit status a script can act on
hl7-v2 --check message.hl7

# Read a vendor dialect
hl7-v2 --dictionary acme.json message.hl7

# Edit and re-emit
hl7-v2 --set 'PID-8=F' --er7 message.hl7

# Convert, both ways
hl7-2-from-er7-into-xml message.hl7 > message.xml
hl7-2-from-xml-into-er7 message.xml

# Round trip as a smoke test
hl7-2-from-er7-into-xml message.hl7 | hl7-2-from-xml-into-er7 | diff - <(hl7-v2 --er7 message.hl7)`;

  const runnable = `# Two programs that talk to each other
cd hl7-2-mllp
cargo run --example tcp_listener     # accepts, reads, acknowledges
cargo run --example tcp_sender       # sends, waits, checks the echo`;
</script>

<DocPage
  lede="Short, copyable snippets grouped by what you are trying to do. Each links to the guide that explains it; nothing here is a substitute for reading the crate's own specification."
  {contents}
>
  <h2 id="reading">Reading a message</h2>
  <p>Parse — see <a href="/guides/parsing/">Parsing</a>.</p>
  <CodeSample language="rust" caption="Parse, and read the structure the header claims" code={parse} />
  <CodeSample language="rust" caption="Force a release when MSH-12 lies" code={forceVersion} />
  <p>Read — see <a href="/guides/navigating/">Navigating</a>.</p>
  <CodeSample language="rust" caption="One value, by path" code={getValue} />
  <CodeSample language="rust" caption="Every value, and every repetition" code={everyValue} />
  <CodeSample language="rust" caption="Walk the tree, and ask each node its own path" code={walk} />
  <CodeSample language="rust" caption="Ask the dictionary what a field is" code={typeOf} />
  <CodeSample language="rust" caption="A file holding one message, several, or a batch" code={batchRead} />

  <h2 id="writing">Writing and answering</h2>
  <p>See <a href="/guides/modifying/">Modifying and building</a>.</p>
  <CodeSample language="rust" caption="Set, null, and clear" code={setValue} />
  <CodeSample language="rust" caption="Add a segment, then render" code={addSegment} />
  <CodeSample language="rust" caption="Build a message from nothing" code={buildMessage} />
  <CodeSample language="rust" caption="Acknowledge, echoing the control ID" code={ackExample} />
  <p>Check — see <a href="/guides/validating/">Validating</a>.</p>
  <CodeSample language="rust" caption="Diagnostics, split by severity" code={validate} />
  <CodeSample language="rust" caption="Refuse anything that does not conform, up front" code={strict} />

  <h2 id="dialects">Dialects and dictionaries</h2>
  <p>See <a href="/guides/dictionaries/">Vendor dictionaries</a>.</p>
  <CodeSample language="rust" caption="A dialect stated inline" code={dictionaryInline} />
  <CodeSample language="rust" caption="A dialect from a file, and reading it back" code={dictionaryFile} />
  <CodeSample language="rust" caption="Generate one from a directory of XSDs" code={dictionaryBuild} />

  <h2 id="typed">Typed structs</h2>
  <p>See <a href="/guides/struct-mode/">Struct mode and derive</a>.</p>
  <CodeSample language="rust" caption="Read into a struct, keeping the escape hatch" code={structRead} />
  <CodeSample language="rust" caption="Write a struct out into a message" code={structWrite} />

  <h2 id="converting">Converting formats</h2>
  <p>See <a href="/guides/converting/">Converting formats</a>.</p>
  <CodeSample language="rust" caption="ER7 into XML and JSON" code={toXml} />
  <CodeSample language="rust" caption="With options" code={convertOptions} />
  <CodeSample language="rust" caption="And back again" code={fromXml} />

  <h2 id="transports">Transports</h2>
  <p>See <a href="/guides/mllp/">MLLP</a> and <a href="/guides/soap/">SOAP</a>.</p>
  <CodeSample language="rust" caption="Frame one message" code={mllpFrame} />
  <CodeSample language="rust" caption="Reassemble a chopped-up stream" code={mllpStream} />
  <CodeSample language="rust" caption="Serve MLLP over TCP" code={mllpServe} />
  <CodeSample language="rust" caption="Receive a SOAP request" code={soapReceive} />
  <CodeSample language="rust" caption="Send one, and read the reply" code={soapSend} />

  <h2 id="v3">HL7 v3</h2>
  <p>See <a href="/guides/hl7-v3/">HL7 v3</a>.</p>
  <CodeSample language="rust" caption="Read the three-level envelope" code={v3Envelope} />
  <CodeSample language="rust" caption="Data types, and explicit absence" code={v3Types} />
  <CodeSample language="rust" caption="The shared XML reader, on its own" code={xmlHelper} />

  <h2 id="shell">Shell one-liners</h2>
  <p>See <a href="/docs/cli/">Command line</a>.</p>
  <CodeSample language="sh" code={shell} />

  <h2 id="runnable">Runnable examples in the repository</h2>
  <p>
    <code>hl7-2-mllp</code> ships two programs that talk to each other. The listener is commented
    with what it shows and what a production listener also needs — worth reading before writing your
    own.
  </p>
  <CodeSample language="sh" code={runnable} />
  <p>
    Several crates also carry a <code>samples/</code> directory — real ER7 files, and the exact XML
    and JSON documents the golden tests produce. They are the fastest way to see what a conversion
    actually emits.
  </p>
</DocPage>
