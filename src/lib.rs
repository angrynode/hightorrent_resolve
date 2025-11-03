//! Resolve magnet links to torrents (WIP)

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
