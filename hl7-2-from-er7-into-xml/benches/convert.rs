//! Criterion benchmarks: how long one conversion takes, over a short
//! message and a 200-observation result.
//!
//! The inputs are synthetic, never real patient data. Run with
//! `cargo bench -p hl7-2-from-er7-into-xml`, and compare against a
//! baseline with `-- --save-baseline before`.

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

/// A short ADT, the shape most interfaces move in bulk.
fn small_er7() -> String {
    "MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814080000||ADT^A08|MSG00042|P|2.5\r\
     EVN|A08|20260814080000\r\
     PID|1||444333222^^^ACME&1.2.3.4&ISO^MR||EVERYWOMAN^EVE^E||19620320|F\r\
     PV1|1|O|OP^^^ACME"
        .to_string()
}

/// A lab result with 200 observations: the shape that decides whether a
/// converter is fast enough for a day's traffic.
fn large_er7() -> String {
    let mut text =
        String::from("MSH|^~\\&|LAB|ACME|EHR|CLINIC|20260814080000||ORU^R01|MSG00042|P|2.5\r");
    text.push_str("PID|1||444333222^^^ACME&1.2.3.4&ISO^MR||EVERYWOMAN^EVE^E||19620320|F\r");
    for i in 1..=200 {
        text.push_str(&format!(
            "OBR|{i}|ORD{i}||24331-1^Lipid Panel^LN\r\
             OBX|{i}|NM|2093-3^Cholesterol^LN||187|mg/dL|<200|N|||F\r\
             NTE|{i}||Fasting sample, drawn \\T\\ processed at ACME\r"
        ));
    }
    text.pop();
    text
}

fn bench_convert(c: &mut Criterion) {
    let small = small_er7();
    let large = large_er7();
    let mut group = c.benchmark_group("er7_into_xml");
    group.throughput(Throughput::Bytes(small.len() as u64));
    group.bench_function("small", |b| {
        b.iter(|| hl7_2_from_er7_into_xml::convert(black_box(&small)).unwrap());
    });
    group.throughput(Throughput::Bytes(large.len() as u64));
    group.bench_function("large", |b| {
        b.iter(|| hl7_2_from_er7_into_xml::convert(black_box(&large)).unwrap());
    });
    group.finish();
}

criterion_group!(benches, bench_convert);
criterion_main!(benches);
