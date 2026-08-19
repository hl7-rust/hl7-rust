//! Reading a SOAP envelope, and writing one.
//!
//! An envelope carries exactly one business payload in its body. That is
//! not a SOAP rule — SOAP permits several — but it is the rule every HL7
//! SOAP interface actually operates on, and enforcing it turns an ambiguous
//! request into a clear rejection rather than a silent choice of the first
//! child.
//!
//! Every failure here is a [`Fault`], because every one of them is
//! something the receiver has to tell the sender.

use crate::fault::{Fault, SOAP_NS};
use crate::xml::{self, Element};

/// A parsed SOAP envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    root: Element,
}

/// Parse a SOAP envelope.
///
/// ```
/// let envelope = hl7_2_soap::parse(
///     r#"<soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/">
///          <soapenv:Body><ADT_A05><MSH><MSH.10>1</MSH.10></MSH></ADT_A05></soapenv:Body>
///        </soapenv:Envelope>"#,
/// )?;
/// assert_eq!(envelope.payload()?.local_name(), "ADT_A05");
/// # Ok::<(), hl7_2_soap::Fault>(())
/// ```
///
/// # Errors
///
/// A `Client` fault (HTTP 400) when the input is not well-formed XML, or
/// when its root element is not an `Envelope`. Both are the sender's to
/// fix, which is why neither is retryable.
pub fn parse(xml_text: &str) -> Result<Envelope, Fault> {
    let root = xml::parse(xml_text).map_err(|_| Fault::client("Malformed SOAP XML request."))?;
    if root.local_name() != "Envelope" {
        return Err(Fault::client("SOAP Envelope element is missing."));
    }
    Ok(Envelope { root })
}

impl Envelope {
    /// The envelope element itself.
    #[must_use]
    pub fn root(&self) -> &Element {
        &self.root
    }

    /// The `Header`, if the sender included one.
    #[must_use]
    pub fn header(&self) -> Option<&Element> {
        self.root.child("Header")
    }

    /// The `Body`.
    ///
    /// # Errors
    ///
    /// A `Client` fault when the envelope has no `Body`.
    pub fn body(&self) -> Result<&Element, Fault> {
        self.root
            .child("Body")
            .ok_or_else(|| Fault::client("SOAP Body element is missing."))
    }

    /// The single business payload element inside the body.
    ///
    /// A body with no children, or with more than one, is a fault: a
    /// receiver that picked the first would process something the sender
    /// did not necessarily mean to send.
    ///
    /// # Errors
    ///
    /// A `Client` fault when there is no `Body`, or when the body holds
    /// anything other than exactly one element.
    pub fn payload(&self) -> Result<&Element, Fault> {
        let body = self.body()?;
        match body.children.as_slice() {
            [only] => Ok(only),
            _ => Err(Fault::client(
                "SOAP Body must contain exactly one business payload element.",
            )),
        }
    }
}

/// Wrap an already-serialized XML payload in a request envelope.
///
/// The payload is inserted as XML, not escaped, because it *is* markup —
/// this is the carriage an HL7 v2.xml interface uses, where the body holds
/// the message's own elements. See [`crate::message::wrap_er7`] for the
/// other convention.
#[must_use]
pub fn wrap_xml(payload_xml: &str) -> String {
    format!(
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            r#"<soapenv:Envelope xmlns:soapenv="{}">"#,
            "<soapenv:Header/>",
            "<soapenv:Body>{}</soapenv:Body>",
            "</soapenv:Envelope>",
        ),
        SOAP_NS, payload_xml
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENVELOPE: &str = r#"<?xml version="1.0"?>
        <soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/">
          <soapenv:Header/>
          <soapenv:Body>
            <ADT_A05 xmlns="urn:hl7-org:v2xml"><MSH><MSH.10>CTRL1</MSH.10></MSH></ADT_A05>
          </soapenv:Body>
        </soapenv:Envelope>"#;

    #[test]
    fn reads_the_payload_out_of_the_body() {
        let envelope = parse(ENVELOPE).unwrap();
        assert_eq!(envelope.payload().unwrap().local_name(), "ADT_A05");
        assert!(envelope.header().is_some());
    }

    #[test]
    fn a_body_with_no_single_payload_is_a_fault() {
        let none = parse(r#"<Envelope><Body></Body></Envelope>"#).unwrap();
        assert_eq!(none.payload().unwrap_err().status, 400);

        let two = parse(r#"<Envelope><Body><A/><B/></Body></Envelope>"#).unwrap();
        assert!(
            two.payload()
                .unwrap_err()
                .reason
                .contains("exactly one business payload")
        );
    }

    #[test]
    fn a_missing_body_is_a_fault() {
        let envelope = parse(r#"<Envelope><Header/></Envelope>"#).unwrap();
        assert!(envelope.body().unwrap_err().reason.contains("Body"));
    }

    #[test]
    fn something_that_is_not_an_envelope_is_a_fault() {
        assert!(
            parse("<NotAnEnvelope/>")
                .unwrap_err()
                .reason
                .contains("Envelope")
        );
        assert!(
            parse("not xml at all")
                .unwrap_err()
                .reason
                .contains("Malformed")
        );
    }

    #[test]
    fn a_wrapped_payload_round_trips() {
        let envelope = wrap_xml("<ADT_A05><MSH><MSH.10>9</MSH.10></MSH></ADT_A05>");
        let parsed = parse(&envelope).unwrap();
        assert_eq!(parsed.payload().unwrap().local_name(), "ADT_A05");
    }
}
