# unjdbc

Convert JDBC table output to regular JSON arrays.

## What it does

JDBC returns query results in a columnar format with a `schema` array and `datarows` array:

```json
{
  "schema": [
    {"name": "field1", "type": "string"},
    {"name": "field2", "type": "struct"}
  ],
  "datarows": [
    ["value1", {"nested": "object"}],
    ["value2", {"nested": "object2"}]
  ]
}
```

This tool converts it to plain NDJSON records:

```json
{"field1": "value1", "field2": {"nested": "object"}}
{"field1": "value2", "field2": {"nested": "object2"}}
```

## Usage

Build the project:
```bash
cargo build --release
```

Convert a JDBC file:
```bash
./target/release/unjdbc samples/big5.json
```

Or pipe from stdin:
```bash
cat samples/big5.json | ./target/release/unjdbc
```

You can also pipe to `jq` for further processing:
```bash
./target/release/unjdbc samples/big5.json | jq -s 'length'
```
