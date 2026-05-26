# unjdbc

Convert JDBC table output to regular JSON objects.

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

## On Performance

This is a personal exercise in trying to make the conversion run as fast as possible.
The core logic is trivial to do in Serde (and a past version of the tool did),
but I wanted to see how far I could go.
This tool is purpose-built to do exactly this conversion.
It also requires that `schema` comes before `datarows`, which is true of all JDBC-emitting services I work with.

The current version is based on a custom purpose-built parser based on [Logos](https://github.com/maciejhirsz/logos).
While it catches many types of errors, those errors aren't particularly descriptive, and many cases aren't caught.
(In particular, invalid objects in datarows are likely to be copied verbatim.)
This allows it to run faster than `wc`.

Some notes on the past and possible future of this tool are in `NOTES.md`.
These were written before the current version so the tenses might be off.

For a more robust version, check out the `sonic` or `serde` historic tags.

## On AI

No AI code is in the present mainline.

The initial commit was largely AI as I needed a tool quickly for an immediate work task.
It's been theseus'd out as I took over and started optimizing for fun.
