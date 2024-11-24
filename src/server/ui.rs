use flate2::read::GzDecoder;
use std::io::Cursor;
use std::process::Stdio;
use tar::Archive;
use tempfile::TempDir;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tracing::info;
use tracing::trace;
use tracing::warn;

pub struct EmbeddedUi {
    // Must hold the temp dir otherwise it will be deleted
    _temp_dir: TempDir,
    pub port: u16,
    process: tokio::process::Child,
}

impl Drop for EmbeddedUi {
    fn drop(&mut self) {
        if let Err(_kind) = self.process.start_kill() {
            info!("Ui was already dead on port {}", self.port);
        } else {
            info!("Ui killed on port {}", self.port);
        }
    }
}

pub fn start_embedded_ui(server_port: &str) -> Result<EmbeddedUi, anyhow::Error> {
    let temp_dir = TempDir::new()?;
    let site_archive = include_bytes!(concat!(env!("OUT_DIR"), "/site.tar.gz"));
    let mut cursor = Cursor::new(site_archive);
    let tar = GzDecoder::new(&mut cursor);
    let mut archive = Archive::new(tar);
    trace!(
        "Unpacking site archive to file://{}",
        temp_dir.path().display()
    );
    archive.unpack(temp_dir.path())?;

    // TODO(doug): Pick default port or random
    //let port = { TcpListener::bind(("127.0.0.1", 0))?.local_addr()?.port() };
    let port = 3000;
    trace!("Starting embedded UI server on port {}", port);
    // TODO(doug): Fix
    const NODE: &str = "/home/doug/.local/share/pnpm/node";
    let mut process = Command::new(NODE)
        .current_dir(temp_dir.path())
        .arg("site/index.js")
        .env(
            "PUBLIC_GRAPHQL_ENDPOINT",
            format!("http://localhost:{}/graphql", server_port),
        )
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = process.stdout.take().expect("Failed to get stdout");
    let mut stdout = BufReader::new(stdout).lines();
    tokio::spawn(async move {
        while let Some(line) = stdout.next_line().await? {
            info!("{}", line);
        }
        Ok::<_, anyhow::Error>(())
    });
    let stderr = process.stderr.take().expect("Failed to get stdout");
    let mut stderr = BufReader::new(stderr).lines();
    tokio::spawn(async move {
        while let Some(line) = stderr.next_line().await? {
            warn!("{}", line);
        }
        Ok::<_, anyhow::Error>(())
    });
    Ok(EmbeddedUi {
        _temp_dir: temp_dir,
        port,
        process,
    })
}
