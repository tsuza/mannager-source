use reqwest::{self};
use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::process::{Child, ChildStdout, Command};

use super::{Error, ExtractError};

pub struct DepotDownloader {
    pub depotdownloader_path: PathBuf,
    process: Option<Child>,
}

#[cfg(target_os = "linux")]
const DEPOTDOWNLOADER_URL: &str = "https://github.com/SteamRE/DepotDownloader/releases/latest/download/DepotDownloader-linux-x64.zip";

#[cfg(target_os = "windows")]
const DEPOTDOWNLOADER_URL: &str = "https://github.com/SteamRE/DepotDownloader/releases/latest/download/DepotDownloader-windows-x64.zip";

impl DepotDownloader {
    pub async fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref().to_path_buf();

        let executable_path = path.join("DepotDownloader");

        if !executable_path.try_exists().unwrap_or(false) {
            download_file(&path).await?;
        }

        Ok(Self {
            depotdownloader_path: executable_path,
            process: None,
        })
    }

    pub async fn download_app(
        &mut self,
        path: &str,
        appid: u32,
    ) -> Result<Option<ChildStdout>, Error> {
        let args = format!("-app {appid} -dir {path}");

        let mut process = Command::new(&self.depotdownloader_path)
            .args(args.split_whitespace())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| Error::SpawnProcessError(err.to_string()))?;

        let stdout = process.stdout.take();

        self.process = Some(process);

        Ok(stdout)
    }
}

async fn download_file(path: &PathBuf) -> Result<(), Error> {
    fs::create_dir_all(path).map_err(|err| Error::DirectoryCreationError(err.to_string()))?;

    let steamcmd_contents = reqwest::get(DEPOTDOWNLOADER_URL).await?.bytes().await?;

    let cursor = Cursor::new(steamcmd_contents);

    let mut zip = zip::ZipArchive::new(cursor).map_err(|err| ExtractError::ZipError(err))?;

    zip.extract(path)
        .map_err(|err| ExtractError::ZipError(err))?;

    Ok(())
}

impl Drop for DepotDownloader {
    fn drop(&mut self) {
        let _ = self.process.as_mut().unwrap().start_kill();
    }
}
