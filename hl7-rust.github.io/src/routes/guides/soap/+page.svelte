<script lang="ts">
  import DocPage from '$lib/components/DocPage.svelte';
  import CodeSample from '$lib/components/CodeSample.svelte';
  import Callout from '$lib/components/Callout.svelte';

  const contents = [
    { id: 'two-crates', label: 'Two crates, one shape' },
    { id: 'scope', label: 'What they do, and what they leave to you' },
    { id: 'receiving', label: 'Receiving' },
    { id: 'sending', label: 'Sending' },
    { id: 'opinions', label: 'Three things they are opinionated about' },
    { id: 'responses', label: 'Reading a response' },
    { id: 'wsdl', label: 'WSDL' },
    { id: 'v3-differences', label: 'Where the v3 crate differs' }
  ];

  const receiving2 = `use hl7_2_soap::{Fault, message, response};

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

  const sending2 = `use hl7_2_soap::{message, response::{self, Outcome}};

let body = message::wrap_er7("MSH|^~\\\\&|APP||||1||ADT^A01|9|P|2.5");
// ...POST body with Content-Type: text/xml; charset=utf-8...

match response::evaluate(status, &reply) {
    Outcome::Accepted => {}
    Outcome::Rejected(reason) => eprintln!("not delivered: {reason}"),
}`;

  const receiving3 = `use hl7_3_soap::{Fault, message, response};

fn accept(request_body: &str) -> Result<String, Fault> {
    let envelope = hl7_3_soap::parse(request_body)?;
    let payload = envelope.payload()?;
    message::check(payload, &["PRPA_IN201305UV02".to_string()], &[])?;
    // ...decode the payload with hl7-3, and forward it, here...
    Ok(message::control_id(payload).unwrap_or_default().to_string())
}`;

  const sending3 = `use hl7_3_soap::{envelope, response::{self, Outcome}};

let body = envelope::wrap_xml(
    r#"<PRPA_IN201305UV02><id extension="9"/></PRPA_IN201305UV02>"#,
);
// ...POST body with Content-Type: text/xml; charset=utf-8...

match response::evaluate(status, &reply) {
    Outcome::Accepted => {}
    Outcome::Rejected(reason) => eprintln!("not delivered: {reason}"),
}`;

  const faults = `Client                  400   do not retry — the request is wrong
Client.Authorization    403   do not retry — the caller is not allowed
Server                  500   retry — the far side had a bad moment`;
</script>

<DocPage
  lede="For HL7 v2, SOAP is the transport an estate ends up with when the messages have to cross a boundary that speaks HTTP. For HL7 v3, SOAP is the transport. Two crates, deliberately the same shape."
  {contents}
>
  <h2 id="two-crates">Two crates, one shape</h2>
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Crate</th><th>Carries</th><th>Acknowledgement</th></tr>
      </thead>
      <tbody>
        <tr>
          <td><a href="/crates/hl7-2-soap/"><code>hl7-2-soap</code></a></td>
          <td>An HL7 v2 message — v2.xml, or ER7 wrapped in one</td>
          <td>A <code>Status</code> element, or the HL7 code</td>
        </tr>
        <tr>
          <td><a href="/crates/hl7-3-soap/"><code>hl7-3-soap</code></a></td>
          <td>A complete HL7 v3 message, root element named for the interaction</td>
          <td>The real v3 <code>MCCI_IN000002UV01</code></td>
        </tr>
      </tbody>
    </table>
  </div>
  <p>
    The difference in emphasis matters. For v2, <a href="/guides/mllp/">MLLP</a> is the usual
    transport and SOAP is the exception — what you get when the system at the far end was built by a
    team who had a WSDL and no socket. For v3 it is the other way round: v3 was designed alongside
    SOAP and WS-*, and real deployments — NHS England's Personal Demographics Service, IHE profiles
    built on v3 — carry it that way.
  </p>

  <h2 id="scope">What they do, and what they leave to you</h2>
  <p><strong>They do:</strong></p>
  <ul>
    <li>Parse a SOAP envelope and take the single business payload out of its body.</li>
    <li>Faults, each carrying the HTTP status that belongs with it.</li>
    <li>Read which message a payload is and its control ID, and check it against what the interface accepts.</li>
    <li>Build the reply, and read one as accepted or rejected.</li>
    <li>Serve a WSDL describing the endpoint at the address it was reached on.</li>
  </ul>
  <p><strong>They do not:</strong></p>
  <ul>
    <li>
      <strong>No HTTP client and no HTTP server.</strong> They turn bytes into meaning and back, and
      leave the socket to whatever you already use.
    </li>
    <li>
      <strong>No HL7 validation and no format conversion.</strong>
      <a href="/crates/hl7-2/"><code>hl7-2</code></a> and the
      <a href="/guides/converting/"><code>hl7-2-from-*</code></a> crates own those.
    </li>
    <li>
      <strong>No RIM decoding</strong>, in the v3 crate's case — that is
      <a href="/crates/hl7-3/"><code>hl7-3</code></a>, which <code>hl7-3-soap</code> deliberately
      does not depend on.
    </li>
  </ul>

  <h2 id="receiving">Receiving</h2>
  <CodeSample language="rust" caption="HL7 v2" code={receiving2} />
  <p>
    <code>Fault</code> converts to an envelope and carries its own HTTP status, so the whole error
    path is two lines. <code>message::check</code> is where you say what this endpoint accepts;
    anything else is a fault rather than something that reaches your business logic.
  </p>
  <CodeSample language="rust" caption="HL7 v3 — the same shape" code={receiving3} />

  <h2 id="sending">Sending</h2>
  <CodeSample language="rust" caption="HL7 v2" code={sending2} />
  <CodeSample language="rust" caption="HL7 v3" code={sending3} />
  <p>
    Note <code>message::wrap_er7</code> in the v2 crate: an interface that speaks ER7 rather than
    v2.xml can still go over SOAP, with the pipe-delimited message carried inside the envelope.
  </p>

  <h2 id="opinions">Three things they are opinionated about</h2>
  <h3>One payload per body</h3>
  <p>
    SOAP permits several; no HL7 interface means several. A body with none or many is a fault
    rather than a silent choice of the first child — because a silent choice is how you process the
    wrong message and never find out.
  </p>

  <h3>Prefixes do not matter</h3>
  <p>
    The same envelope arrives as <code>soapenv:</code>, <code>soap:</code>,
    <code>SOAP-ENV:</code>, or unprefixed depending on which stack sent it. Elements are matched on
    their local name, so all four are read alike. Insisting on one prefix is the single most common
    way a working SOAP integration breaks when the other end changes stack.
  </p>

  <h3>A fault carries its HTTP status</h3>
  <CodeSample language="text" code={faults} label="Fault codes and their statuses" />
  <Callout type="warning" heading="Getting the pairing wrong is expensive in both directions">
    <p>
      A <code>Client</code> fault returned as a 500 becomes an infinite retry loop around a poison
      message. A <code>Server</code> fault returned as a 400 drops a message that would have gone
      through a moment later. Neither failure is visible in testing, and both are visible in
      production.
    </p>
  </Callout>

  <h2 id="responses">Reading a response</h2>
  <p>
    There is no equivalent of <code>MSA-1</code> in SOAP — there are three places to look and no
    agreement about which wins. <code>response::evaluate</code> reads all three, in the order that
    cannot be talked out of a rejection:
  </p>
  <ol>
    <li>A non-2xx HTTP status.</li>
    <li>
      A <code>Fault</code> element — <em>even under HTTP 200</em>, because some stacks answer 200
      and put the refusal in the body.
    </li>
    <li>
      The acknowledgement: a <code>Status</code> element for v2, or
      <code>acknowledgement/typeCode/@code</code> for v3.
    </li>
    <li>Otherwise, acceptance.</li>
  </ol>
  <p>
    In the v2 crate, <code>Status</code> is accepted for <code>AA</code>, <code>CA</code>, and
    <code>Success</code>. Both conventions are in the field — an implementation that echoes the HL7
    acknowledgement code, and one that writes a word — and a sender that knows only one will retry
    forever against an endpoint that speaks the other. The crate this one was generalised from had
    exactly that split between its own receiver and its own sender.
  </p>
  <p>
    In the v3 crate, <code>AA</code> and <code>CA</code> are accepted, matching v2's
    <code>MSA-1</code>: the two standards draw the acceptance codes from the same conceptual
    vocabulary.
  </p>

  <h2 id="wsdl">WSDL</h2>
  <p>
    Both crates can serve a WSDL that describes the endpoint <em>at the address it was reached
    on</em>, rather than at a hardcoded one. That matters behind a load balancer or a reverse proxy,
    where the address a client used is not the address the process is bound to — a WSDL that
    advertises the wrong host sends the next client somewhere it cannot reach.
  </p>

  <h2 id="v3-differences">Where the v3 crate differs</h2>
  <ul>
    <li>
      The payload is a complete HL7 v3 message whose root element is named for the interaction —
      <code>PRPA_IN201305UV02</code>, say — rather than an HL7 v2 message in one of two encodings.
    </li>
    <li>
      It can read the payload's claimed assigning authority, in addition to which interaction it is
      and its control ID.
    </li>
    <li>
      It builds the real HL7 v3 acknowledgement, <code>MCCI_IN000002UV01</code>, with
      <code>acknowledgement/typeCode</code> — not an invented shape.
    </li>
    <li>
      It has no dependency on <code>hl7-3</code>. A SOAP envelope is XML, and reading one requires
      no HL7 knowledge beyond the names of a few elements — so a router can move v3 traffic without
      decoding it.
    </li>
  </ul>
  <p>
    Both crates depend on exactly one thing:
    <a href="/crates/hl7-2-xml-lite-helper/"><code>hl7-2-xml-lite-helper</code></a>, re-exported as
    <code>xml</code>, which has no dependencies of its own.
  </p>
</DocPage>
