[![Travis (.com)](https://img.shields.io/travis/com/Lucretiel/lazy_format.svg?logo=travis)](https://travis-ci.com/Lucretiel/lazy_format/)
[![Crates.io](https://img.shields.io/crates/v/lazy_format.svg?logo=rust&logoColor=white)](https://crates.io/crates/lazy_format)
[![docs.rs](https://docs.rs/lazy_format/badge.svg)](https://docs.rs/lazy_format)
[![license](https://img.shields.io/github/license/Lucretiel/lazy_format.svg)](https://crates.io/crates/lazy_format/)

# lazy_format

A `[no_std]` library for lazily formatting things. Because allocating temporary strings is bad for your health.

```rust
use std::io;

use lazy_format::lazy_format;
use joinery::JoinableIterator;

fn main() {
	let result = (0..10)
		.map(|value| lazy_format!("\t'{}'", value))
		.join_with(",\n")
		.to_string();

	assert_eq!(result,
"	'0'
	'1'
	'2'
	'3'
	'4'
	'5'
	'6'
	'7'
	'8'
	'9'")
}
```

The above example is the key motivating example: when building up some kind of object you wish to write or format, there's no reason to allocate intermediary strings (which is what `format!` does). Instead, `lazy_format!` captures its arguments and returns an opaque struct with a `Display` implementation, so that the actual formatting can happen directly into its final destination buffer (such as a file or string).
