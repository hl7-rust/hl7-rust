//! Criterion benchmarks: how long one conversion takes, over a short
//! message and a 200-observation result.
//!
//! The inputs are built here rather than pulled from the forward sibling
//! crate, so that measuring this crate does not mean depending on that one:
//! the short document is the crate's own golden sample, and the long one is
//! the same shape repeated. Both are synthetic, never real patient data.
//!
//! Run with `cargo bench -p hl7-2-from-xml-into-er7`, and compare against a
//! baseline with `-- --save-baseline before`.

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

/// The crate's own golden sample: a short ORM with group nesting.
fn small_xml() -> &'static str {
    include_str!("../samples/orm_o01.xml")
}

/// A result with 200 observations, in the shape the forward crate emits:
/// grouped, typed component names, one element per repetition.
fn large_xml() -> String {
    let mut text = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<ORU_R01 xmlns=\"urn:hl7-org:v2xml\">\n",
    );
    text.push_str(
        "  <MSH>\n    <MSH.1>|</MSH.1>\n    <MSH.2>^~\\&amp;</MSH.2>\n\
         \x20   <MSH.9><MSG.1>ORU</MSG.1><MSG.2>R01</MSG.2></MSH.9>\n  </MSH>\n",
    );
    text.push_str(
        "  <PID>\n    <PID.1>1</PID.1>\n\
         \x20   <PID.3><CX.1>444333222</CX.1><CX.4><HD.1>ACME</HD.1></CX.4></PID.3>\n\
         \x20   <PID.5><XPN.1><FN.1>EVERYWOMAN</FN.1></XPN.1><XPN.2>EVE</XPN.2></PID.5>\n  </PID>\n",
    );
    for i in 1..=200 {
        text.push_str(&format!(
            "  <ORU_R01.OBSERVATION>\n    <OBX>\n      <OBX.1>{i}</OBX.1>\n\
             \x20     <OBX.3><CE.1>2093-3</CE.1><CE.2>Cholesterol &amp; esters</CE.2></OBX.3>\n\
             \x20     <OBX.5>187</OBX.5>\n    </OBX>\n\
             \x20   <NTE>\n      <NTE.1>{i}</NTE.1>\n\
             \x20     <NTE.3>Fasting sample, drawn \\.br\\ processed</NTE.3>\n    </NTE>\n\
             \x20 </ORU_R01.OBSERVATION>\n"
        ));
    }
    text.push_str("</ORU_R01>\n");
    text
}

fn bench_convert(c: &mut Criterion) {
    let small = small_xml();
    let large = large_xml();
    let mut group = c.benchmark_group("xml_into_er7");
    group.throughput(Throughput::Bytes(small.len() as u64));
    group.bench_function("small", |b| {
        b.iter(|| hl7_2_from_xml_into_er7::convert(black_box(small)).unwrap());
    });
    group.throughput(Throughput::Bytes(large.len() as u64));
    group.bench_function("large", |b| {
        b.iter(|| hl7_2_from_xml_into_er7::convert(black_box(&large)).unwrap());
    });
    // `parse` stops at the value tree; the difference from `large` is what
    // rendering the ER7 text costs.
    group.bench_function("large_parse_only", |b| {
        b.iter(|| hl7_2_from_xml_into_er7::parse(black_box(&large)).unwrap());
    });
    group.finish();
}

criterion_group!(benches, bench_convert);
criterion_main!(benches);
