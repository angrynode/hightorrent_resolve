# hightorrent_resolve

<!-- cargo-rdme start -->

hightorrent_resolve is a very high-level crate to turn magnet links into
torrent files.

**NOTE:** hightorrent_resolve currently only supports Bittorrent v1 and hybrid magnets (no Bittorrent v2 support),
because upstream librqbit does not fully support Bittorrent v2 (see [librqbit#70](https://github.com/ikatson/rqbit/issues/70)).

It is entirely possible to use upstream [librqbit](https://docs.rs/librqbit) directly to perform this operation.
hightorrent_resolve is provided simply as a ease-of-life wrapper around that API, and integrating with
[hightorrent](https://docs.rs/hightorrent) types directly.

To use as the binary crate, enabling the `cli` feature is required, like so:

```rust
cargo run --features cli -- --timeout 10 hello.magnet
```

<!-- cargo-rdme end -->
