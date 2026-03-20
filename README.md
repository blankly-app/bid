# Bid

Typed ULID-compatible 128-bit sortable unique identifiers.

Each ID is 20 bytes (160 bits) with an embedded type hash, so IDs for different entity types are inherently distinct. Encoded as 32 Crockford Base32 characters.

## Layout

```
Bytes 0..4   (32 bits)  FNV-1a hash of the type tag
Bytes 4..10  (48 bits)  Unix timestamp in milliseconds
Bytes 10..20 (80 bits)  Cryptographic random
```

IDs with the same tag sort chronologically. IDs with different tags cluster by type.

## Usage

```rust
use bid::Bid;

// Generate a tagged ID — the tag is hashed into the first 4 bytes
let id = Bid::tagged("usr").unwrap();

// Display with a human-readable prefix
let display = format!("usr-{id}");
// => "usr-0A1B2C3D..."

// Parse back — prefix is stripped automatically
let parsed: Bid = display.parse().unwrap();
assert_eq!(parsed, id);

// Untagged IDs (tag hash = 0)
let bare = Bid::new().unwrap();
```

## Accessors

```rust
let id = Bid::tagged("usr").unwrap();

id.tag_hash();    // u32 — FNV-1a hash of "usr"
id.timestamp_ms(); // u64 — creation time in ms
id.random();      // [u8; 10] — random component
id.as_bytes();    // &[u8; 20]
id.is_nil();      // false
```

## Serde

Enable the `serde` feature for JSON/binary serialization:

```toml
bid = { version = "0.1", features = ["serde"] }
```

Human-readable formats (JSON) serialize as the 32-char Base32 string. Binary formats serialize as raw bytes.

## Part of Blankly

Bid is a shared primitive used across the [Blankly](https://github.com/blankly-app/blankly) platform.

## License

MIT
