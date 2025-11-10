//! hightorrent_resolve is a very high-level crate to turn magnet links into
//! torrent files.
//!
//! **NOTE:** hightorrent_resolve currently only supports Bittorrent v1 and hybrid magnets (no Bittorrent v2 support),
//! because upstream librqbit does not fully support Bittorrent v2 (see [librqbit#70](https://github.com/ikatson/rqbit/issues/70)).
//!
//! It is entirely possible to use upstream [librqbit](https://docs.rs/librqbit) directly to perform this operation.
//! hightorrent_resolve is provided simply as a ease-of-life wrapper around that API, and integrating with
//! [hightorrent](https://docs.rs/hightorrent) types directly.
//!
//! To use the command-line, enabling the `cli` feature is required, like so:
//!
//! ```ignore
//! cargo run --features cli -- --timeout 10 hello.magnet
//! ```
//!
//! ## Features
//!
//! The following torrents are supported:
//!
//! - [x] v1 torrents (tested)
//! - [ ] Hybrid torrents (not currently tested)
//! - [ ] v2 torrents (not currently planned)
//!
//! The following metadata sources are supported:
//!
//! - [x] UDP trackers (tested)
//! - [x] HTTP trackers (tested)
//! - [x] DHT (not currently tested)
//!
//! ## Running tests
//!
//! To run tests, you need the [aquatic tracker](https://github.com/greatest-ape/aquatic) installed:
//!
//! - run a UDP instance on port 30000: `aquatic_udp -c tests/aquatic-udp-config.toml`
//! - run a HTTP instance on port 30000: `aquatic_http -c tests/aquatic-http-config.toml`
//!
//! Then you can run tests like so: `cargo test --features test_udp,test_http`
//!
//! ## Why use rqbit and all its dependencies?
//!
//! At first i thought i'd write my own resolver from scratch. But the steps are more involved than i thought:
//!
//! - implement a UDP/HTTP tracker client to find peers
//! - (optionally) implement a DHT client to find peers
//! - foreach peer:
//!   - send a Bittorrent handshake ([BEP-0003](https://www.bittorrent.org/beps/bep_0003.html))
//!   - send an extension handshake ([BEP-0010](https://www.bittorrent.org/beps/bep_0010.html)) with `ut_metadata` capability advertised ([BEP-0009](https://www.bittorrent.org/beps/bep_0009.html))
//!   - send the peer metadata request (BEP-0009)
//!
//! That would be a fun adventure, but making it correct, fast and robust is more work than i wanted. Since [rqbit](https://github.com/ikatson/rqbit) already implements all of that (and more) as part of [librqbit](https://docs.rs/librqbit), it would be a shame not to use it. Thanks for their amazing work!
//!
//! ## Alternatives
//!
//! If you know of any other alternative, feel free to open a pull request!
//!
//! - [demagnetize](https://github.com/jwodder/demagnetize-rs) is only a command-line tool (no library), and i believe it only does sequential requests so it's much slower (outside of tests which only use one tracker/peer) than librqbit
//! - [imdl](https://github.com/casey/intermodal) is only a command-line tool (no library), and does not support DHT
//!
//! ## License
//!
//! This project is GNU aGPL v3. You are free to use this program for any purpose, but if you it as part of a service provided to someone else than yourself, you need to make any modification and derived work available under the same license.
//!
//! ## Contributing
//!
//! Contrbutions welcome.
use anyhow::{Result, bail};
use async_tempfile::TempDir;
use hightorrent::{MagnetLink, TorrentFile};
use librqbit::*;
use tokio::time::timeout;

use std::time::Duration;

pub async fn resolve(magnet: &MagnetLink) -> Result<TorrentFile> {
    let tmpdir = TempDir::new().await?;
    log::debug!("Using tmpdir {}", tmpdir.dir_path().display());

    let session = Session::new_with_opts(
        tmpdir.dir_path().to_path_buf(),
        SessionOptions {
            listen: Some(ListenerOptions::default()),
            ..Default::default()
        },
    )
    .await?;
    let resp = session
        .add_torrent(
            AddTorrent::from_url(magnet.to_string()),
            Some(AddTorrentOptions {
                list_only: true,
                ..Default::default()
            }),
        )
        .await?;

    match resp {
        AddTorrentResponse::ListOnly(resp) => {
            log::debug!("Found metainfo for torrent {:?}", resp.info_hash);
            Ok(TorrentFile::from_slice(&resp.torrent_bytes)?)
        }
        _ => {
            bail!("RQBIT BUG!");
        }
    }
}

pub async fn resolve_with_timeout(
    magnet: &MagnetLink,
    duration: Duration,
) -> Option<Result<TorrentFile>> {
    timeout(duration, resolve(magnet)).await.ok()
}
