use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
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

    for rows in [10, 500, 10000].iter() {
        for columns in [5, 20, 100].iter() {
            let input = generate_jdbc_json(*rows, *columns);
            group.throughput(Throughput::Elements((*rows * *columns) as u64));
            group.bench_with_input(
                BenchmarkId::new("rows_columns", format!("{}x{}", rows, columns)),
                &input,
                |b, input| {
                    b.iter(|| {
                        let mut output = Vec::new();
                        convert_jdbc(input, &mut output).unwrap();
                        black_box(output);
                    });
                },
            );
        }
    }

    group.finish();
}

criterion_group!(benches, bench_end_to_end);
criterion_main!(benches);
