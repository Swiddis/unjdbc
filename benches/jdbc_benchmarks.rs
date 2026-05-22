use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::io::Cursor;
use unjdbc::convert_jdbc;

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

fn bench_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");

    for size in [10, 500, 10000].iter() {
        let input = generate_jdbc_json(*size, 5);
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &input, |b, input| {
            b.iter(|| {
                let mut reader = Cursor::new(black_box(input.as_bytes()));
                let mut output = Vec::new();
                convert_jdbc(&mut reader, &mut output).unwrap();
                black_box(output);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_end_to_end);
criterion_main!(benches);
