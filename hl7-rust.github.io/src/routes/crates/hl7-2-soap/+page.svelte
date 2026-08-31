<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import CrateMeta from '$lib/components/CrateMeta.svelte';
  import RelatedCrates from '$lib/components/RelatedCrates.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { crateBySlug } from '$lib/data/crates';

  const crate = crateBySlug('hl7-2-soap');

  const contents = [
    { id: 'what', label: 'What it is' },
    { id: 'does', label: 'What it does, and what it does not' },
    { id: 'receiving', label: 'Receiving' },
    { id: 'sending', label: 'Sending' },
    { id: 'opinions', label: 'Three things it is opinionated about' },
    { id: 'responses', label: 'Reading a response' },
    { id: 'dependencies', label: 'Dependencies' },
    { id: 'related', label: 'Related crates' }
  ];

  const receiving = `use hl7_2_soap::{Fault, message, response};

fn handle(request_body: &str) -> (u16, String) {
    match accept(request_body) {
        Ok(control_id) => (200, response::success(&control_id)),
        Err(fault) => (fault.status, fault.to_envelope()),
    }
}

fn accept(request_body: &str) -> Result<String, Fault> {
    let envelope = hl7_2_soap::parse(request_body)?;
    let payload = envelope.payload()?;
    message::check(payload, &["ADT_A05".to_string()], &[])?;
    // ...validate and forward the payload here...
    Ok(message::control_id(payload).unwrap_or_default().to_string())
}`;

  const sending = `use hl7_2_soap::{message, response::{self, Outcome}};

let body = message::wrap_er7("MSH|^~\\\\&|APP||||1||ADT^A01|9|P|2.5");
// ...POST body with Content-Type: text/xml; charset=utf-8...

match response::evaluate(status, &reply) {
    Outcome::Accepted => {}
    Outcome::Rejected(reason) => eprintln!("not delivered: {reason}"),
}`;

  const faults = `Client                  400   do not retry — the request is wrong
Client.Authorization    403   do not retry — the caller is not allowed
Server                  500   retry — the far side had a bad moment`;
</script>

<DocPage lede={crate.tagline} {contents}>
  <CrateMeta {crate} />

  <h2 id="what">What it is</h2>
  <p>
    MLLP is how HL7 v2 usually moves, and
    <a href="/crates/hl7-2-mllp/"><code>hl7-2-mllp</code></a> is that transport. SOAP is the other
    one — what an estate ends up with when the messages have to cross a boundary that speaks HTTP, or
    when the system at the far end was built by a team who had a WSDL and no socket.
  </p>
  <p>
    This crate is that transport, and it is deliberately the same shape as its MLLP sibling: it does
    the protocol and nothing else.
  </p>

  <h2 id="does">What it does, and what it does not</h2>
  <p><strong>It does:</strong></p>
  <ul>
    <li>Parse a SOAP envelope and take the single business payload out of its body.</li>
    <li>Faults, each carrying the HTTP status that belongs with it.</li>
    <li>
      Read a v2.xml payload, or ER7 wrapped in one, and check a payload against what the interface
      accepts.
    </li>
    <li>Build the reply, and read one as accepted or rejected.</li>
    <li>Serve a WSDL that describes the endpoint at the address it was reached on.</li>
  </ul>
  <p><strong>It does not:</strong></p>
  <p>
    No HTTP client and no HTTP server: it turns bytes into meaning and back, and leaves the socket to
    whatever you already use. No HL7 validation and no format conversion either —
    <a href="/crates/hl7-2/"><code>hl7-2</code></a> and the
    <a href="/guides/converting/"><code>hl7-2-from-*</code></a> crates own those.
  </p>

  <h2 id="receiving">Receiving</h2>
  <CodeSample language="rust" code={receiving} />

  <h2 id="sending">Sending</h2>
  <CodeSample language="rust" code={sending} />
  <p>
    Note <code>message::wrap_er7</code>: an interface that speaks pipe-delimited ER7 rather than
    v2.xml can still go over SOAP, with the message carried inside the envelope.
  </p>

  <h2 id="opinions">Three things it is opinionated about</h2>
  <h3>One payload per body</h3>
  <p>
    SOAP permits several; no HL7 interface means several. A body with none or many is a fault rather
    than a silent choice of the first child.
  </p>

  <h3>Prefixes do not matter</h3>
  <p>
    The same envelope arrives as <code>soapenv:</code>, <code>soap:</code>,
    <code>SOAP-ENV:</code> or unprefixed depending on which stack sent it. Elements are matched on
    their local name, so all four are read alike.
  </p>

  <h3>A fault carries its HTTP status</h3>
  <CodeSample language="text" code={faults} />
  <Callout type="warning" heading="Getting the pairing wrong is expensive both ways">
    <p>
      A <code>Client</code> fault returned as a 500 is how a poison message becomes an infinite retry
      loop. A <code>Server</code> fault returned as a 400 is how a message that would have gone
      through a moment later is dropped instead.
    </p>
  </Callout>

  <h2 id="responses">Reading a response</h2>
  <p>
    There is no equivalent of <code>MSA-1</code> in SOAP — there are three places to look and no
    agreement about which wins. <code>response::evaluate</code> reads all three, in the order that
    cannot be talked out of a rejection: a non-2xx status, then a <code>Fault</code> element (even
    under HTTP 200, because some stacks answer 200 and put the refusal in the body), then a
    <code>Status</code> element, then acceptance.
  </p>
  <p>
    <code>Status</code> is accepted for <code>AA</code>, <code>CA</code>, and
    <code>Success</code>. Both conventions are in the field — an implementation that echoes the HL7
    acknowledgement code, and one that writes a word — and a sender that knows only one will retry
    forever against an endpoint that speaks the other. The crate this one was generalised from had
    exactly that split between its own receiver and its own sender.
  </p>

  <h2 id="dependencies">Dependencies</h2>
  <p>
    One: <a href="/crates/hl7-2-xml-lite-helper/"><code>hl7-2-xml-lite-helper</code></a>, re-exported
    here as <code>xml</code>. It reads the envelopes this crate carries, matching elements on their
    local name.
  </p>
  <p>
    It is shared rather than owned because three crates in this family needed the same XML subset and
    each had written its own copy: this crate,
    <a href="/crates/hl7-2-from-xml-into-er7/"><code>hl7-2-from-xml-into-er7</code></a>, and
    <a href="/crates/hl7-2-from-xsd-into-json-dictionary/"
      ><code>hl7-2-from-xsd-into-json-dictionary</code></a
    >. One reader that keeps both text and attributes replaced all three: the audit surface is
    unchanged, and there is one parser to read instead of three.
  </p>

  <RelatedCrates slugs={crate.related} />
</DocPage>
