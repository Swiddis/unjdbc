# Benchmarks

This directory contains Criterion benchmarks for the unjdbc tool.

## Running Benchmarks

Run all benchmarks:
```bash
cargo bench
```

Run specific benchmark group:
```bash
cargo bench --bench jdbc_benchmarks parse
cargo bench --bench jdbc_benchmarks transform_row
cargo bench --bench jdbc_benchmarks end_to_end
```

## Benchmark Groups

### `parse`
Measures JSON deserialization performance with varying dataset sizes (10, 500, 10000 rows).
Tests the `parse_jdbc_json` function.

### `extract_field_names`
Measures schema field extraction performance with varying column counts (5, 10, 20, 50 columns).
Tests the `extract_field_names` function.

### `transform_row`
Measures single row transformation performance with varying column counts (5, 10, 20, 50 columns).
Tests the `transform_row` function - the core transformation logic.

### `transform_datarows`
Measures bulk row transformation and serialization with varying dataset sizes (10, 500, 10000 rows).
Tests the `transform_datarows` function.

### `end_to_end`
Measures the complete pipeline: parse → extract schema → transform all rows.
This represents the full tool performance without I/O.

## Results Location

Benchmark results are saved in `target/criterion/`. HTML reports are available at:
```
target/criterion/report/index.html
```

## Comparison

To compare against a baseline:
```bash
# Save current performance as baseline
cargo bench -- --save-baseline main

# Make changes...

# Compare against baseline
cargo bench -- --baseline main
```
