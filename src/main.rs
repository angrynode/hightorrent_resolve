use anyhow::{bail, Context, Result};
use clap::Parser;
use hightorrent::MagnetLink;
use librqbit::*;
use temp_dir::TempDir;

use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum Output {
    Stdout,
    Path(PathBuf),
}

impl FromStr for Output {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        if s == "-" {
            Ok(Self::Stdout)
        } else {
            Ok(Self::Path(PathBuf::from(s)))
        }
    }
}

impl std::fmt::Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdout => write!(f, "STDOUT"),
            Self::Path(path) => write!(f, "{}", path.display()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MagnetFileOrLink {
    // Cannot store librqbit Magnet because it's not Clone/Debug
    // Also it doesn't have easy methods to get printable infohash
    link: MagnetLink,
}

impl Deref for MagnetFileOrLink {
    type Target = MagnetLink;
    
    fn deref(&self) -> &Self::Target {
        &self.link
    }
}

impl FromStr for MagnetFileOrLink {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.starts_with("magnet:") {
            // Sanity check
            // _ = MagnetLink::new(s)?;
            // return Ok(Self { link: s.to_string() })
            return Ok(Self { link: MagnetLink::new(s)? });
        }

        let path = PathBuf::from(s);
        if !path.is_file() {
            bail!("No such magnet file: {s}");
        }

        let magnet = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file {s}"))?;
        // Sanity check
        // _ = MagnetLink::parse(&magnet)?;
        // Ok(Self { link: magnet.to_string() })
        Ok(Self { link: MagnetLink::new(&magnet)? })
    }
}

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short, long, default_value_t=Output::Stdout)]
    output: Output,
    magnet: MagnetFileOrLink,
}

#[tokio::main(flavor="current_thread")]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Cli::parse();
    log::info!("Searching metadata for magnet {}", args.magnet.hash());
    
    let tmpdir = TempDir::new()?;
    log::info!("Using tmpdir {}", tmpdir.path().display());
    
    let session = Session::new_with_opts(
        tmpdir.path().to_path_buf(),
        SessionOptions {
            listen: Some(ListenerOptions::default()),
            ..Default::default()
        }
    ).await?;
    let resp = session.add_torrent(
        AddTorrent::from_url(args.magnet.deref().to_string()),
        Some(AddTorrentOptions {
            list_only: true,
            ..Default::default()
        })
    ).await?;

    match resp {
        AddTorrentResponse::ListOnly(resp) => {
            log::info!("Found metainfo for torrent {:?}", resp.info_hash);
        }
        _ => {
            bail!("RQBIT BUG!");
        }
    }

    Ok(())
}
