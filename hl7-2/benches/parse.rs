//! Criterion benchmarks: what the five things this crate is asked to do cost,
//! over a short ADT and a 200-observation lab result.
//!
//! The inputs are synthetic, never real patient data — see the workspace's
//! `spec/phi/index.md`. Run with `cargo bench -p hl7-2`, and compare against a
//! baseline with `-- --save-baseline before`.
//!
//! The five are deliberately separate rather than one end-to-end number,
//! because they scale differently and an integration usually does only some of
//! them: parsing is per message, `get` is per field read, the tree and
//! validation are per message but only when asked for, and rendering is per
//! message written back out. A single "messages per second" figure hides which
//! of those a given interface is actually paying for.

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

/// A lab result with 200 observations: the shape that decides whether a parser
/// is fast enough for a day's traffic.
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

/// Parsing: text in, `Message` out. Every other number here assumes this one
/// has already been paid.
fn bench_parse(c: &mut Criterion) {
    let small = small_er7();
    let large = large_er7();
    let mut group = c.benchmark_group("parse");
    group.throughput(Throughput::Bytes(small.len() as u64));
    group.bench_function("small", |b| {
        b.iter(|| hl7_2::parse(black_box(&small)).unwrap());
    });
    group.throughput(Throughput::Bytes(large.len() as u64));
    group.bench_function("large", |b| {
        b.iter(|| hl7_2::parse(black_box(&large)).unwrap());
    });
    group.finish();
}

/// Reading one field by path, which is what an interface that only wants the
/// MRN and the accession number does thousands of times a second.
fn bench_get(c: &mut Criterion) {
    let small = hl7_2::parse(&small_er7()).unwrap();
    let large = hl7_2::parse(&large_er7()).unwrap();
    let mut group = c.benchmark_group("get");
    group.bench_function("small_pid5_1", |b| {
        b.iter(|| small.get(black_box("PID-5.1")).unwrap());
    });
    // A late repetition, so the cost of walking to it is in the number rather
    // than hidden by hitting the first segment.
    group.bench_function("large_obx200_5", |b| {
        b.iter(|| large.get(black_box("OBX[200]-5")).unwrap());
    });
    group.finish();
}

/// The generic tree: every value in the message, named. Materializes the whole
/// message, so it is the most expensive read and the one to avoid in a hot
/// path that wants two fields.
fn bench_tree(c: &mut Criterion) {
    let small = hl7_2::parse(&small_er7()).unwrap();
    let large = hl7_2::parse(&large_er7()).unwrap();
    let mut group = c.benchmark_group("tree");
    group.bench_function("small", |b| b.iter(|| black_box(&small).tree()));
    group.bench_function("large", |b| b.iter(|| black_box(&large).tree()));
    group.finish();
}

/// Validation against the message's own dictionary.
fn bench_validate(c: &mut Criterion) {
    let small = hl7_2::parse(&small_er7()).unwrap();
    let large = hl7_2::parse(&large_er7()).unwrap();
    let mut group = c.benchmark_group("validate");
    group.bench_function("small", |b| b.iter(|| black_box(&small).validate()));
    group.bench_function("large", |b| b.iter(|| black_box(&large).validate()));
    group.finish();
}

/// Rendering back to ER7 — the second half of the round trip that has to come
/// out byte for byte.
fn bench_render(c: &mut Criterion) {
    let small = hl7_2::parse(&small_er7()).unwrap();
    let large = hl7_2::parse(&large_er7()).unwrap();
    let mut group = c.benchmark_group("render");
    group.throughput(Throughput::Bytes(small.to_er7().len() as u64));
    group.bench_function("small", |b| b.iter(|| black_box(&small).to_er7()));
    group.throughput(Throughput::Bytes(large.to_er7().len() as u64));
    group.bench_function("large", |b| b.iter(|| black_box(&large).to_er7()));
    group.finish();
}

criterion_group!(
    benches,
    bench_parse,
    bench_get,
    bench_tree,
    bench_validate,
    bench_render
);
criterion_main!(benches);
