//! Resolve magnet links to torrents (WIP)

use anyhow::{Context, Result, bail};
use clap::Parser;
use hightorrent::MagnetLink;
use hightorrent_resolve::resolve_with_timeout;

use std::ops::Deref;
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;
use std::time::Duration;

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
            return Ok(Self {
                link: MagnetLink::new(s)?,
            });
        }

        let path = PathBuf::from(s);
        if !path.is_file() {
            bail!("No such magnet file: {s}");
        }

        let magnet =
            std::fs::read_to_string(path).with_context(|| format!("Failed to read file {s}"))?;
        Ok(Self {
            link: MagnetLink::new(&magnet)?,
        })
    }
}

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short, long, default_value_t=Output::Stdout)]
    output: Output,
    #[arg(short, long, default_value_t = 30)]
    timeout: u64,
    magnet: MagnetFileOrLink,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Cli::parse();
    log::info!("Searching metadata for magnet {}", args.magnet.hash());

    if let Some(res) = resolve_with_timeout(&args.magnet, Duration::from_secs(args.timeout)).await {
        let _resp = res?;
    } else {
        log::error!("Timeout");
        exit(1);
    }

    Ok(())
}
