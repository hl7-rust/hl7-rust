//! Criterion benchmarks for the reader, over the two document shapes this
//! crate exists to read: a v2.xml message and a SOAP envelope.
//!
//! Run with `cargo bench -p hl7-2-xml-lite-helper`, and compare against a
//! baseline with `-- --save-baseline before`.

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

/// A v2.xml message with 200 observations, the shape a converter reads.
fn v2xml() -> String {
    let mut text = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<ORU_R01 xmlns=\"urn:hl7-org:v2xml\">\n",
    );
    text.push_str("  <MSH><MSH.1>|</MSH.1><MSH.2>^~\\&amp;</MSH.2></MSH>\n");
    for i in 1..=200 {
        text.push_str(&format!(
            "  <OBX><OBX.1>{i}</OBX.1><OBX.3><CE.1>2093-3</CE.1>\
             <CE.2>Cholesterol &amp; esters</CE.2></OBX.3><OBX.5>187</OBX.5></OBX>\n"
        ));
    }
    text.push_str("</ORU_R01>\n");
    text
}

/// A prefixed SOAP envelope: the other document shape, and the one that
/// exercises the prefix-stripping accessors rather than the tag scanner.
fn soap() -> String {
    "<?xml version=\"1.0\"?>\
     <soapenv:Envelope xmlns:soapenv=\"http://schemas.xmlsoap.org/soap/envelope/\">\
     <soapenv:Header><wsa:To>https://example.invalid/hl7</wsa:To></soapenv:Header>\
     <soapenv:Body><hl7:sendMessage xmlns:hl7=\"urn:hl7-org:v2xml\">\
     <hl7:payload>MSH|^~\\&amp;|LAB|ACME</hl7:payload>\
     </hl7:sendMessage></soapenv:Body></soapenv:Envelope>"
        .to_string()
}

fn bench_parse(c: &mut Criterion) {
    let v2 = v2xml();
    let envelope = soap();
    let mut group = c.benchmark_group("parse");
    group.throughput(Throughput::Bytes(v2.len() as u64));
    group.bench_function("v2xml", |b| {
        b.iter(|| hl7_2_xml_lite_helper::parse(black_box(&v2)).unwrap());
    });
    group.throughput(Throughput::Bytes(envelope.len() as u64));
    group.bench_function("soap_envelope", |b| {
        b.iter(|| hl7_2_xml_lite_helper::parse(black_box(&envelope)).unwrap());
    });
    group.finish();
}

fn bench_lookup(c: &mut Criterion) {
    let envelope = hl7_2_xml_lite_helper::parse(&soap()).unwrap();
    let v2 = hl7_2_xml_lite_helper::parse(&v2xml()).unwrap();
    let mut group = c.benchmark_group("lookup");
    group.bench_function("find_by_local_name", |b| {
        b.iter(|| black_box(&envelope).find("payload").unwrap());
    });
    group.bench_function("last_child_scan", |b| {
        b.iter(|| black_box(&v2).children.last().unwrap().local_name());
    });
    group.finish();
}

criterion_group!(benches, bench_parse, bench_lookup);
criterion_main!(benches);
