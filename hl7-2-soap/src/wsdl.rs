//! Describing the endpoint to client tooling.
//!
//! A WSDL is what `SoapUI`, .NET's `svcutil` and Java's `wsimport` read to
//! generate a client. Serving it from the endpoint itself, with the address
//! filled in from the request that asked for it, means the document is
//! always right for the environment it came from — no hand-edited copy per
//! environment, and no support call about a client pointed at dev.
//!
//! The payload is declared as `xsd:anyType` rather than importing the HL7
//! schemas. Those schemas use relative `xsd:include` paths, which break the
//! moment the WSDL is saved somewhere else — which is exactly what happens
//! when someone imports it into a client tool. The real structural check
//! belongs on the server, against the schemas it holds; the WSDL's job is
//! to describe the *contract*, and the contract is "a SOAP envelope, at
//! this address, carrying one HL7 message".

use crate::fault::SOAP_NS;
use crate::message::{ER7_ELEMENT, OPERATION};
use crate::response::ACK_ELEMENT;
use crate::xml;

/// The namespace the generated service is defined in, when the caller does
/// not name one.
///
/// A target namespace identifies *your* service, not this crate, so a
/// deployment with existing clients should pass its own to
/// [`for_address_in`] — the generated clients are bound to whatever it was
/// when they were generated, and changing it breaks them. This default
/// exists so the common case needs no decision, not because it is right for
/// everyone.
pub const TARGET_NS: &str = "urn:hl7-2-soap:service";

/// The WSDL for an endpoint at `address`.
///
/// `address` is the full URL clients should POST to, as the serving
/// application knows it — typically rebuilt from the request's host header
/// so it matches however the endpoint was reached.
///
/// ```
/// let wsdl = hl7_2_soap::wsdl::for_address("https://hub.example.nhs.uk/soap");
/// assert!(wsdl.contains(r#"location="https://hub.example.nhs.uk/soap""#));
/// ```
#[must_use]
pub fn for_address(address: &str) -> String {
    for_address_in(address, TARGET_NS)
}

/// The WSDL for an endpoint at `address`, in a target namespace of your
/// own.
///
/// Use this when the service already has clients: they are bound to the
/// namespace they were generated against, and serving a different one
/// silently stops matching them.
///
/// ```
/// let wsdl = hl7_2_soap::wsdl::for_address_in(
///     "https://hub.example.nhs.uk/soap",
///     "urn:example:hl7-soap-server",
/// );
/// assert!(wsdl.contains(r#"targetNamespace="urn:example:hl7-soap-server""#));
/// ```
#[must_use]
pub fn for_address_in(address: &str, target_namespace: &str) -> String {
    TEMPLATE
        .replace("@TARGET_NS@", &xml::escape(target_namespace))
        .replace("@OPERATION@", OPERATION)
        .replace("@ER7_ELEMENT@", ER7_ELEMENT)
        .replace("@ACK_ELEMENT@", ACK_ELEMENT)
        .replace("@ADDRESS@", &xml::escape(address))
}

/// The document, with `@NAME@` where a value goes.
///
/// Substitution rather than `format!` because the template is XML: it is
/// read alongside the documents it describes far more often than it is
/// edited, and doubling every brace to please a format string would make
/// it worse at the thing it is mostly for.
// `r###"..."###`: the template contains `namespace="##other"`, and both
// `"#` and `"##` appear inside it, so the delimiter has to be longer than
// either.
const TEMPLATE: &str = r###"<?xml version="1.0" encoding="UTF-8"?>
<definitions name="Hl7SoapServer"
             targetNamespace="@TARGET_NS@"
             xmlns:tns="@TARGET_NS@"
             xmlns:soap="http://schemas.xmlsoap.org/wsdl/soap/"
             xmlns:xsd="http://www.w3.org/2001/XMLSchema"
             xmlns="http://schemas.xmlsoap.org/wsdl/">

  <types>
    <xsd:schema targetNamespace="@TARGET_NS@" elementFormDefault="qualified">
      <!-- The payload is deliberately unconstrained here; the server
           validates it against the HL7 schemas it holds. -->
      <xsd:element name="@OPERATION@">
        <xsd:complexType>
          <xsd:sequence>
            <xsd:element name="@ER7_ELEMENT@" type="xsd:anyType" minOccurs="0"/>
            <xsd:any namespace="##other" processContents="lax"
                     minOccurs="0" maxOccurs="unbounded"/>
          </xsd:sequence>
        </xsd:complexType>
      </xsd:element>
      <xsd:element name="@ACK_ELEMENT@">
        <xsd:complexType>
          <xsd:sequence>
            <xsd:element name="Status" type="xsd:string"/>
            <xsd:element name="MessageControlId" type="xsd:string" minOccurs="0"/>
          </xsd:sequence>
        </xsd:complexType>
      </xsd:element>
    </xsd:schema>
  </types>

  <message name="@OPERATION@Request">
    <part name="parameters" element="tns:@OPERATION@"/>
  </message>
  <message name="@OPERATION@Response">
    <part name="parameters" element="tns:@ACK_ELEMENT@"/>
  </message>

  <portType name="Hl7SoapServerPortType">
    <operation name="@OPERATION@">
      <input message="tns:@OPERATION@Request"/>
      <output message="tns:@OPERATION@Response"/>
    </operation>
  </portType>

  <binding name="Hl7SoapServerBinding" type="tns:Hl7SoapServerPortType">
    <soap:binding style="document" transport="http://schemas.xmlsoap.org/soap/http"/>
    <operation name="@OPERATION@">
      <soap:operation soapAction="@TARGET_NS@/@OPERATION@"/>
      <input><soap:body use="literal"/></input>
      <output><soap:body use="literal"/></output>
    </operation>
  </binding>

  <service name="Hl7SoapServer">
    <port name="Hl7SoapServerPort" binding="tns:Hl7SoapServerBinding">
      <soap:address location="@ADDRESS@"/>
    </port>
  </service>
</definitions>
"###;

/// The SOAP envelope namespace the generated contract is bound to, for a
/// caller that wants to state it alongside.
#[must_use]
pub fn envelope_namespace() -> &'static str {
    SOAP_NS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_address_is_the_one_that_was_asked_for() {
        let wsdl = for_address("http://localhost:8080/soap");
        assert!(wsdl.contains(r#"location="http://localhost:8080/soap""#));
        let other = for_address("https://prod.example.nhs.uk/soap");
        assert!(other.contains(r#"location="https://prod.example.nhs.uk/soap""#));
    }

    #[test]
    fn an_address_cannot_break_the_document() {
        let wsdl = for_address(r#"http://x/"><evil/>"#);
        assert!(!wsdl.contains("<evil/>"));
        assert!(xml::parse(&wsdl).is_ok());
    }

    #[test]
    fn it_is_well_formed_and_describes_the_operation() {
        let wsdl = for_address("http://localhost/soap");
        let root = xml::parse(&wsdl).unwrap();
        assert_eq!(root.local_name(), "definitions");
        assert!(root.find("portType").is_some());
        assert!(root.find("binding").is_some());
        assert!(root.find("service").is_some());
        // The operation and response names match what the crate actually
        // sends and accepts, so the contract cannot drift from the code.
        assert!(wsdl.contains(&format!(r#"<operation name="{OPERATION}">"#)));
        assert!(wsdl.contains(&format!(r#"element="tns:{ACK_ELEMENT}""#)));
    }
}
