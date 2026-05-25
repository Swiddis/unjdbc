## Problem Statement

JDBC JSON is a format that looks like this:

```json
{
	"schema": [{"name": "name", "type": "string"}, {"name": "age", "type": "int"}],
	"datarows": [["Marie", 25], ["John", 24]]
}
```

This is easy enough to read when you have two columns and enough rows to fit on one screen. When dealing with many columns, counting is hard. When dealing with many rows, you probably won't see the columns before they're pushed out of your buffer. The goal is to write a script to convert that into this:

```json
{"name": "Marie", "age": 25}
{"name": "John", "age": 24}
```

This is trivial to do with any scripting language: load a map from indices to schema names, then for each datarow, zip the values with the names and emit that object.

I've been thinking a lot about how to make this as fast as possible, for fun and profit.

Additional notes/constraints:
- We're going to assume `schema` comes before `datarows` in the object. All JDBC-emitting servers I care about follow this convention. If this is ever not true, I will be very sad.
- While not technically necessary for jq, for many ndjson parsers and to be spec-compliant with ndjson, the output json objects must be on one line each. I could also emit a regular json array instead and ignore all formatting but this is not philosophically satisfying (and makes the jq expressions marginally more annoying to work with).
- I would additionally _like_ to be able to format the objects consistently in terms of whitespace, but I'm not sure how serious I am about that.
- The content of the `type` doesn't matter, we're just copying the array values directly. The array values may be sub-arrays or objects or nulls or any other valid JSON values. (I think technically the servers don't ever serialize with scientific notation but failing to handle that would not be philosophically satisfying.)

## The story so far

V0.0.1: the trivial code in Python, it was perfectly adequate for all my real use cases, I even had it outputting pretty colorized json records when the terminal was interactive :p

V0.1.0: Claude rewrote my original Python in Rust with Serde, it was decent -- about 0.06 GB/s (or 13 MB in about 180 ms). But Serde fundamentally needs a JSON object to be parsed all at once and can only do operations once the entire object is deserialized in memory (including all array leaf values). It has facilities for streaming many small objects, but notice that the JDBC object is technically a single very wide object. This always made me a little philosophically uncomfortable processing 10+ MB objects, despite having memory to spare.

V0.2.0: I modified the Rust to use [sonic-rs](https://github.com/cloudwego/sonic-rs) instead of serde_json. Sonic is a cute little JSON parsing library, it's SIMD accelerated. Critically, it also has facilities to lazy-load values from text buffers: you can write Sonic code that first loads the Schema object and produces your map, then incrementally goes through all the array entries and serializes. This still requires loading the entire input string into memory, but we don't have the full parsing overhead on top of the plain IO and alloc cost.

I will be surprised if I can actually get faster than 0.2, sonic is really fast (simd and such) and the IO cost is not that severe, I'm effectively going to need to parse JSON at the speed of `malloc` to beat this. It also is the last version that would be able to handle the `schema` and `datarows` keys being out of order.

V0.3.0: (Current) I ditched Sonic and started writing a custom incremental parser-writer built on top of [Logos](https://github.com/maciejhirsz/logos). The idea is to just go through the tokens with minimal bracket tracking and copy the relevant values directly into the output buffer without any array-value parsing at all. But this _still_ requires loading the full input, which is a constraint that's necessary to make lifetime math work right.

(future: To my surprise, it was faster! By a factor of about 50%)

V0.4.0: Where I might go. If I want to have a true streaming parser, I effectively need to ditch Rust's lifetime system and work directly on input and output buffers, which means either unsafe Rust or C. (Or Zig, my beloved, but it's going through a tough time right now.) The goal is to design a lexer-parser that can emit the formatted JSON output as it goes without needing to load the whole input stream in memory, and without parsing the inner datarow entry contents beyond bracket counting.

## But like... Self-invalidating buffers are hard, yo

The Rust libraries are kinda onto something. You can imagine your input buffer has a parsing position and a codegen position:

```
[["whatever_key": "a long string tha...
                  ^ codegen        ^ parsing
```

If parsing runs off the buffer and we aren't in a position to codegen, we need to make sure at the end of whatever clear/flush/resize ops we have two valid pointers and haven't dropped any content. This is doable with a lot of pointer arithmetic. It is not really doable in safe Rust, to my knowledge.

So that's where I currently am at: Trying to figure out the data structures for the input and output here. Is the input a ring buffer? What is the output? What happens if we try to load a string that's more than 8 kilobytes long? What _exactly_ are the stdin and stdout buffers and how do they work? Should we even have separate codegen and parsing? (Maybe we can codegen token-by-token like the Rust version, but then this is only performant if output buffering is sane).

Fun stuff :D
