//! Criterion benchmarks: how long one conversion takes, over a short
//! message and a 200-observation result.
//!
//! The inputs are built here rather than pulled from the forward sibling
//! crate, so that measuring this crate does not mean depending on that one:
//! the short document is the crate's own golden sample, and the long one is
//! the same shape repeated. Both are synthetic, never real patient data.
//!
//! Run with `cargo bench -p hl7-2-from-json-into-er7`, and compare against a
//! baseline with `-- --save-baseline before`.

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

/// The crate's own golden sample: a short ORM with group nesting.
fn small_json() -> &'static str {
    include_str!("../samples/orm_o01.json")
}

/// A result with 200 observations, in the shape the forward crate emits:
/// grouped, typed component keys, an array per repeating group.
fn large_json() -> String {
    let mut text = String::from(
        "{\"ORU_R01\": {\
         \"MSH\": {\"MSH.1\": \"|\", \"MSH.2\": \"^~\\\\&\", \
         \"MSH.9\": {\"MSG.1\": \"ORU\", \"MSG.2\": \"R01\"}}, \
         \"PID\": {\"PID.1\": \"1\", \
         \"PID.3\": {\"CX.1\": \"444333222\", \"CX.4\": {\"HD.1\": \"ACME\"}}, \
         \"PID.5\": {\"XPN.1\": {\"FN.1\": \"EVERYWOMAN\"}, \"XPN.2\": \"EVE\"}}, \
         \"ORU_R01.OBSERVATION\": [",
    );
    for i in 1..=200 {
        if i > 1 {
            text.push_str(", ");
        }
        text.push_str(&format!(
            "{{\"OBX\": {{\"OBX.1\": \"{i}\", \
             \"OBX.3\": {{\"CE.1\": \"2093-3\", \"CE.2\": \"Cholesterol & esters\"}}, \
             \"OBX.5\": \"187\"}}, \
             \"NTE\": {{\"NTE.1\": \"{i}\", \
             \"NTE.3\": \"Fasting sample, drawn \\\\.br\\\\ processed\"}}}}"
        ));
    }
    text.push_str("]}}");
    text
}

fn bench_convert(c: &mut Criterion) {
    let small = small_json();
    let large = large_json();
    let mut group = c.benchmark_group("json_into_er7");
    group.throughput(Throughput::Bytes(small.len() as u64));
    group.bench_function("small", |b| {
        b.iter(|| hl7_2_from_json_into_er7::convert(black_box(small)).unwrap());
    });
    group.throughput(Throughput::Bytes(large.len() as u64));
    group.bench_function("large", |b| {
        b.iter(|| hl7_2_from_json_into_er7::convert(black_box(&large)).unwrap());
    });
    // `parse` stops at the value tree; the difference from `large` is what
    // rendering the ER7 text costs.
    group.bench_function("large_parse_only", |b| {
        b.iter(|| hl7_2_from_json_into_er7::parse(black_box(&large)).unwrap());
    });
    group.finish();
}

criterion_group!(benches, bench_convert);
criterion_main!(benches);
