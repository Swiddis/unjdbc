use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use sonic_rs::JsonContainerTrait;
use std::hint::black_box;
use unjdbc::{extract_field_names, parse_jdbc_json, transform_datarows, transform_row};

fn generate_jdbc_json(num_rows: usize, num_cols: usize) -> String {
    let mut schema = Vec::new();
    for i in 0..num_cols {
        schema.push(format!(r#"{{"name":"field_{}","type":"string"}}"#, i));
    }

    let mut datarows = Vec::new();
    for row in 0..num_rows {
        let mut values = Vec::new();
        for col in 0..num_cols {
            values.push(format!(r#""value_{}_{}""#, row, col));
        }
        datarows.push(format!("[{}]", values.join(",")));
    }

    format!(
        r#"{{"schema":[{}],"datarows":[{}]}}"#,
        schema.join(","),
        datarows.join(",")
    )
}

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    for size in [10, 100, 1000].iter() {
        let input = generate_jdbc_json(*size, 5);
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &input, |b, input| {
            b.iter(|| parse_jdbc_json(black_box(input)).unwrap());
        });
    }

    group.finish();
}

fn bench_extract_field_names(c: &mut Criterion) {
    let mut group = c.benchmark_group("extract_field_names");

    for num_cols in [5, 10, 20, 50].iter() {
        let input = generate_jdbc_json(100, *num_cols);
        let parsed = parse_jdbc_json(&input).unwrap();
        let schema = parsed
            .as_object()
            .unwrap()
            .get(&"schema")
            .unwrap()
            .as_array()
            .unwrap();

        group.throughput(Throughput::Elements(*num_cols as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_cols),
            schema,
            |b, schema| {
                b.iter(|| extract_field_names(black_box(schema)));
            },
        );
    }

    group.finish();
}

fn bench_transform_row(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_row");

    for num_cols in [5, 10, 20, 50].iter() {
        let input = generate_jdbc_json(1, *num_cols);
        let parsed = parse_jdbc_json(&input).unwrap();
        let obj = parsed.as_object().unwrap();
        let schema = obj.get(&"schema").unwrap().as_array().unwrap();
        let field_names = extract_field_names(schema);
        let datarows = obj.get(&"datarows").unwrap().as_array().unwrap();
        let row = datarows[0].as_array().unwrap();

        group.throughput(Throughput::Elements(*num_cols as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(num_cols),
            &(row, &field_names),
            |b, (row, field_names)| {
                b.iter(|| transform_row(black_box(row), black_box(field_names)));
            },
        );
    }

    group.finish();
}

fn bench_transform_datarows(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_datarows");

    for size in [10, 100, 1000].iter() {
        let input = generate_jdbc_json(*size, 5);
        let parsed = parse_jdbc_json(&input).unwrap();
        let obj = parsed.as_object().unwrap();
        let schema = obj.get(&"schema").unwrap().as_array().unwrap();
        let field_names = extract_field_names(schema);
        let datarows = obj.get(&"datarows").unwrap().as_array().unwrap();

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &(datarows, &field_names),
            |b, (datarows, field_names)| {
                b.iter(|| transform_datarows(black_box(datarows), black_box(field_names)));
            },
        );
    }

    group.finish();
}

fn bench_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");

    for size in [10, 100, 1000].iter() {
        let input = generate_jdbc_json(*size, 5);
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &input, |b, input| {
            b.iter(|| {
                let parsed = parse_jdbc_json(black_box(input)).unwrap();
                let obj = parsed.as_object().unwrap();
                let schema = obj.get(&"schema").unwrap().as_array().unwrap();
                let field_names = extract_field_names(schema);
                let datarows = obj.get(&"datarows").unwrap().as_array().unwrap();
                transform_datarows(datarows, &field_names)
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_parse,
    bench_extract_field_names,
    bench_transform_row,
    bench_transform_datarows,
    bench_end_to_end
);
criterion_main!(benches);
