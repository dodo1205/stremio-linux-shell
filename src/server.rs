use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{self, Child, Command},
    thread,
};

use anyhow::{Context, Ok};
use serde::Deserialize;
use tracing::debug;
use url::Url;

use crate::constants::{SERVER_DOWNLOAD_ENDPOINT, SERVER_UPDATER_ENDPOINT};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ServerUpdaterResponse {
    latest_version: String,
}

pub struct Server {
    file_path: PathBuf,
    version_path: PathBuf,
    process: Option<Child>,
}

impl Server {
    pub fn new(data_path: &Path) -> Self {
        let file_path = Path::new(data_path).join("server.js");
        let version_path = Path::new(data_path).join("server_version");

        Self {
            file_path,
            version_path,
            process: None,
        }
    }

    pub fn setup(&self) -> anyhow::Result<()> {
        let latest_version = reqwest::blocking::get(SERVER_UPDATER_ENDPOINT)?
            .json::<ServerUpdaterResponse>()?
            .latest_version;

        let should_download = fs::read_to_string(&self.version_path)
            .map_or(true, |current_version| current_version != latest_version);

        if should_download {
            let download_url = Url::parse(
                SERVER_DOWNLOAD_ENDPOINT
                    .replace("VERSION", &latest_version)
                    .as_str(),
            )?;

            let latest_file = reqwest::blocking::get(download_url)?
                .bytes()
                .context("Failed to fetch server file")?;

            fs::write(&self.file_path, latest_file).context("Failed to write server file")?;
            fs::write(&self.version_path, &latest_version)
                .context("Failed to write version file")?;
        }

        Ok(())
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        let mut child = Command::new("node")
            .arg(self.file_path.as_os_str())
            .stdout(process::Stdio::piped())
            .spawn()
            .context("Failed to start server")?;

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            thread::spawn(move || {
                while let Some(Result::Ok(line)) = lines.next() {
                    debug!(target: "server", "{}", line);
                }
            });
        }

        self.process = Some(child);

        Ok(())
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(mut process) = self.process.take() {
            process.kill().context("Failed to kill server process")?;
        }

        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop().expect("Failed to stop server");
    }
}
