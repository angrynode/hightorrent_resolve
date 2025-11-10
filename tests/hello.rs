#[cfg(feature = "test_common")]
use hightorrent::MagnetLink;
#[cfg(feature = "test_common")]
use hightorrent_resolve::*;
#[cfg(feature = "test_common")]
use librqbit::*;
#[cfg(feature = "test_common")]
use std::time::Duration;
#[cfg(feature = "test_common")]
use tokio::sync::{oneshot, oneshot::Sender};
#[cfg(feature = "test_common")]
use tokio::task::JoinHandle;
#[cfg(feature = "test_common")]
use tokio::time::timeout;

/// Seed a `hello` file containing `world` string.
///
/// This is used by the `tests/hello/` torrent and magnets.
///
/// Returns a tokio handle to later abort the task, and expects:
///
/// - the name of the torrent variant (eg `tests/hello/hello_udp_ipv4.torrent`),
///   to test the appropriate trackers according to scenario
/// - a channel sender, to let the test runner know when the seeder is ready
#[cfg(feature = "test_common")]
async fn spawn_hello_seeder(variant: &'static str, tx: Sender<()>) -> JoinHandle<()> {
    let task_handle = tokio::task::spawn(async move {
        // Temporary directory for the seeder session
        let tmpdir = async_tempfile::TempDir::new().await.unwrap();

        // Place expected content in there
        tokio::fs::write(tmpdir.join("hello"), "world")
            .await
            .unwrap();

        let session = Session::new_with_opts(
            tmpdir.to_path_buf(),
            SessionOptions {
                listen: Some(ListenerOptions::default()),
                disable_dht: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let torrent_handle = session
            .add_torrent(
                AddTorrent::from_local_filename(variant).unwrap(),
                Some(AddTorrentOptions {
                    // See https://github.com/ikatson/rqbit/issues/509
                    overwrite: true,
                    ..Default::default()
                }),
            )
            .await
            .unwrap()
            .into_handle()
            .unwrap();
        torrent_handle.wait_until_initialized().await.unwrap();
        torrent_handle.wait_until_completed().await.unwrap();
        if !torrent_handle.stats().finished {
            panic!("rqbit should find that hello contains world.");
        }

        // We are now ready to seed, so we inform the tests that they can start
        tx.send(()).unwrap();
    });

    task_handle
}

#[tokio::test]
#[cfg(feature = "test_udp")]
async fn udp_ipv4() {
    // Start the seeder and wait or it to be ready, with hard timeout
    let (tx, rx) = oneshot::channel();
    let task_handle = spawn_hello_seeder("tests/hello/hello_udp_ipv4.torrent", tx).await;
    timeout(Duration::from_secs(10), rx).await.unwrap().unwrap();

    // Start new foreground session fetching metainfo
    let magnet = MagnetLink::new(
        &tokio::fs::read_to_string("tests/hello/hello_udp_ipv4.magnet")
            .await
            .unwrap(),
    )
    .unwrap();
    let res = resolve_with_timeout(&magnet, Duration::from_secs(10)).await;
    if res.is_none() {
        panic!("TIMEOUT!");
    }
    let res = res.unwrap();
    assert!(res.is_ok());

    task_handle.abort();
    // May be a JoinError because the task was aborted, we don't care
    let _ = task_handle.await;
}

// This test is currently disabled due to parsing of ipv6 literals, see
// https://github.com/angrynode/hightorrent/issues/23
#[tokio::test]
#[cfg(feature = "test_udp")]
async fn udp_ipv6() {
    // Start the seeder and wait or it to be ready, with hard timeout
    let (tx, rx) = oneshot::channel();
    let task_handle = spawn_hello_seeder("tests/hello/hello_udp_ipv6.torrent", tx).await;
    timeout(Duration::from_secs(10), rx).await.unwrap().unwrap();

    // Start new foreground session fetching metainfo
    let magnet = MagnetLink::new(
        &tokio::fs::read_to_string("tests/hello/hello_udp_ipv6.magnet")
            .await
            .unwrap(),
    )
    .unwrap();
    let res = resolve_with_timeout(&magnet, Duration::from_secs(10)).await;
    if res.is_none() {
        panic!("TIMEOUT!");
    }
    let res = res.unwrap();
    assert!(res.is_ok());

    task_handle.abort();
    // May be a JoinError because the task was aborted, we don't care
    let _ = task_handle.await;
}

#[tokio::test]
#[cfg(feature = "test_http")]
async fn http_ipv4() {
    // Start the seeder and wait or it to be ready, with hard timeout
    let (tx, rx) = oneshot::channel();
    let task_handle = spawn_hello_seeder("tests/hello/hello_http_ipv4.torrent", tx).await;
    timeout(Duration::from_secs(10), rx).await.unwrap().unwrap();

    // Start new foreground session fetching metainfo
    let magnet = MagnetLink::new(
        &tokio::fs::read_to_string("tests/hello/hello_http_ipv4.magnet")
            .await
            .unwrap(),
    )
    .unwrap();
    let res = resolve_with_timeout(&magnet, Duration::from_secs(10)).await;
    if res.is_none() {
        panic!("TIMEOUT!");
    }
    let res = res.unwrap();
    assert!(res.is_ok());

    task_handle.abort();
    // May be a JoinError because the task was aborted, we don't care
    let _ = task_handle.await;
}

// This test is currently disabled due to parsing of ipv6 literals, see
// https://github.com/angrynode/hightorrent/issues/23
#[tokio::test]
#[cfg(feature = "test_http")]
async fn http_ipv6() {
    // Start the seeder and wait or it to be ready, with hard timeout
    let (tx, rx) = oneshot::channel();
    let task_handle = spawn_hello_seeder("tests/hello/hello_http_ipv6.torrent", tx).await;
    timeout(Duration::from_secs(10), rx).await.unwrap().unwrap();

    // Start new foreground session fetching metainfo
    let magnet = MagnetLink::new(
        &tokio::fs::read_to_string("tests/hello/hello_http_ipv6.magnet")
            .await
            .unwrap(),
    )
    .unwrap();
    let res = resolve_with_timeout(&magnet, Duration::from_secs(10)).await;
    if res.is_none() {
        panic!("TIMEOUT!");
    }
    let res = res.unwrap();
    assert!(res.is_ok());

    task_handle.abort();
    // May be a JoinError because the task was aborted, we don't care
    let _ = task_handle.await;
}
