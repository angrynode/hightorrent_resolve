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
//! To use as the binary crate, enabling the `cli` feature is required, like so:
//!
//! ```ignore
//! cargo run --features cli -- --timeout 10 hello.magnet
//! ```

use anyhow::{Result, bail};
use async_tempfile::TempDir;
use hightorrent::MagnetLink;
use librqbit::*;
use tokio::time::timeout;

use std::time::Duration;

pub async fn resolve(magnet: &MagnetLink) -> Result<ListOnlyResponse> {
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
            Ok(resp)
        }
        _ => {
            bail!("RQBIT BUG!");
        }
    }
}

pub async fn resolve_with_timeout(
    magnet: &MagnetLink,
    duration: Duration,
) -> Option<Result<ListOnlyResponse>> {
    timeout(duration, resolve(magnet)).await.ok()
}
