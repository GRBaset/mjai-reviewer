# convlog

Crate convlog provides methods to transform mahjong logs from tenhou.net/6 (JSON) and tenhou.net/0 (XML)
format into mjai format.

Additionally, it provides a program that reads /0 logs from a SQLite DB (`tenhou-log` format) and converts them to mjai, gzipped.

## Building
The `--release` flag is important for performance reasons.
```
cargo build --release
```

## Basic usage
From `mjai-reviewer` root folder. Make sure the output folder exists first.
```
target/release/convlog [path to DB] [path to output folder]
```
