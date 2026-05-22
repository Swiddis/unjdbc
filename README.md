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

Install the project:
```bash
cargo install --path .
```

Convert a JDBC file:
```bash
unjdbc samples/big5.json
```

Or pipe from stdin:
```bash
cat samples/big5.json | unjdbc
```

You can also pipe to `jq` for further processing:
```bash
unjdbc samples/big5.json | jq -s 'length'
```
