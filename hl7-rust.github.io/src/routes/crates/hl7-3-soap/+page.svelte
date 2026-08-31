<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import CrateMeta from '$lib/components/CrateMeta.svelte';
  import RelatedCrates from '$lib/components/RelatedCrates.svelte';
  import Callout from '$lib/components/Callout.svelte';
  import { crateBySlug } from '$lib/data/crates';

  const crate = crateBySlug('hl7-3-soap');

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

  const receiving = `use hl7_3_soap::{Fault, message, response};

fn handle(request_body: &str) -> (u16, String) {
    match accept(request_body) {
        Ok(control_id) => (200, response::success(&control_id)),
        Err(fault) => (fault.status, fault.to_envelope()),
    }
}

fn accept(request_body: &str) -> Result<String, Fault> {
    let envelope = hl7_3_soap::parse(request_body)?;
    let payload = envelope.payload()?;
    message::check(payload, &["PRPA_IN201305UV02".to_string()], &[])?;
    // ...decode the payload with hl7-3, and forward it, here...
    Ok(message::control_id(payload).unwrap_or_default().to_string())
}`;

  const sending = `use hl7_3_soap::{envelope, response::{self, Outcome}};

let body = envelope::wrap_xml(
    r#"<PRPA_IN201305UV02><id extension="9"/></PRPA_IN201305UV02>"#,
);
// ...POST body with Content-Type: text/xml; charset=utf-8...

match response::evaluate(status, &reply) {
    Outcome::Accepted => {}
    Outcome::Rejected(reason) => eprintln!("not delivered: {reason}"),
}`;
</script>

<DocPage lede={crate.tagline} {contents}>
  <CrateMeta {crate} />

  <h2 id="what">What it is</h2>
  <p>
    Unlike v2 — where <a href="/crates/hl7-2-mllp/">MLLP</a> is the usual transport and SOAP is the
    exception — SOAP <em>is</em> HL7 v3's own historically dominant transport. v3 was designed
    alongside SOAP and WS-*, and real deployments carry it that way: NHS England's Personal
    Demographics Service, IHE profiles built on v3.
  </p>
  <p>
    This crate is that transport, and it is deliberately the same shape as its
    <a href="/crates/hl7-2-soap/"><code>hl7-2-soap</code></a> cousin: it does the protocol and
    nothing else.
  </p>

  <h2 id="does">What it does, and what it does not</h2>
  <p><strong>It does:</strong></p>
  <ul>
    <li>
      Parse a SOAP envelope and take the single business payload out of its body — a complete HL7 v3
      message, root element named for the interaction.
    </li>
    <li>Faults, each carrying the HTTP status that belongs with it.</li>
    <li>
      Read which interaction a payload is, its control ID, and its claimed assigning authority, and
      check a payload against what the interface accepts.
    </li>
    <li>
      Build the real HL7 v3 acknowledgement — <code>MCCI_IN000002UV01</code>, with
      <code>acknowledgement/typeCode</code> — and read one as accepted or rejected.
    </li>
    <li>Serve a WSDL that describes the endpoint at the address it was reached on.</li>
  </ul>
  <p><strong>It does not:</strong></p>
  <p>
    No HTTP client and no HTTP server. No RIM decoding and no domain-payload interpretation either —
    <a href="/crates/hl7-3/"><code>hl7-3</code></a> owns those.
  </p>

  <h2 id="receiving">Receiving</h2>
  <CodeSample language="rust" code={receiving} />

  <h2 id="sending">Sending</h2>
  <CodeSample language="rust" code={sending} />

  <h2 id="opinions">Three things it is opinionated about</h2>
  <p>
    The same three as its v2 cousin, for the same reasons.
    <strong>One payload per body</strong> — SOAP permits several, no HL7 interface means several, and
    a body with none or many is a fault rather than a silent choice of the first child.
    <strong>Prefixes do not matter</strong> — <code>soapenv:</code>, <code>soap:</code>,
    <code>SOAP-ENV:</code>, and unprefixed are all read alike, because insisting on one prefix
    rejects valid documents from every other tool. <strong>A fault carries its HTTP status</strong> —
    <code>Client</code> is a 400 and must not be retried, <code>Client.Authorization</code> is a 403,
    <code>Server</code> is a 500 and should be.
  </p>

  <h2 id="responses">Reading a response</h2>
  <p>
    <code>response::evaluate</code> reads the same three places <code>hl7-2-soap</code> does, in the
    order that cannot be talked out of a rejection: a non-2xx status, then a <code>Fault</code>
    element (even under HTTP 200), then <code>acknowledgement/typeCode/@code</code>, then acceptance.
  </p>
  <p>
    <code>AA</code> and <code>CA</code> are accepted, matching v2's <code>MSA-1</code> — the two
    standards draw the acceptance codes from the same conceptual vocabulary.
  </p>

  <h2 id="dependencies">Dependencies</h2>
  <p>
    One: <a href="/crates/hl7-2-xml-lite-helper/"><code>hl7-2-xml-lite-helper</code></a>, re-exported
    here as <code>xml</code>, shared with <code>hl7-2-soap</code> and the other crates in the family
    that read XML. It has no dependencies of its own, so the audit surface stays small.
  </p>
  <Callout heading="It deliberately does not depend on hl7-3">
    <p>
      A SOAP envelope is XML, and reading one requires no HL7 knowledge beyond the names of a few
      elements. Keeping the transport free of the model means a router can move v3 traffic without
      decoding it — and that a caller who only needs the envelope does not compile the RIM.
    </p>
  </Callout>

  <RelatedCrates slugs={crate.related} />
</DocPage>
