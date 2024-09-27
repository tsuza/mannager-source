use core::error;
use reqwest::{self};
use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    process::Stdio,
};
use thiserror::Error;
use tokio::process::{Child, ChildStdout, Command};

pub struct DepotDownloader {
    pub depotdownloader_path: PathBuf,
    process: Option<Child>,
}

#[cfg(target_os = "linux")]
const DEPOTDOWNLOADER_URL: &str = "https://github.com/SteamRE/DepotDownloader/releases/latest/download/DepotDownloader-linux-x64.zip";

#[cfg(target_os = "windows")]
const DEPOTDOWNLOADER_URL: &str = "https://github.com/SteamRE/DepotDownloader/releases/latest/download/DepotDownloader-windows-x64.zip";

impl DepotDownloader {
    pub async fn new(path: impl AsRef<Path>) -> Result<Self, Box<dyn error::Error>> {
        let path = path.as_ref().to_path_buf();

        if !path.try_exists()? {
            download_file(&path).await?;
        }

        let executable_path = path.join("DepotDownloader");

        Ok(Self {
            depotdownloader_path: executable_path.into(),
            process: None,
        })
    }

    pub async fn download_app(
        &mut self,
        path: &str,
        appid: u32,
    ) -> Result<Option<ChildStdout>, Box<dyn error::Error>> {
        let args = format!("-app {appid} -dir {path}");

        let mut process = Command::new(&self.depotdownloader_path)
            .args(args.split_whitespace())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = process.stdout.take();

        self.process = Some(process);

        Ok(stdout)
    }
}

async fn download_file(path: &PathBuf) -> Result<(), Box<dyn error::Error>> {
    fs::create_dir_all(path)?;

    let steamcmd_contents = reqwest::get(DEPOTDOWNLOADER_URL).await?.bytes().await?;

    let cursor = Cursor::new(steamcmd_contents);

    let mut zip = zip::ZipArchive::new(cursor)?;

    zip.extract(path)?;

    Ok(())
}

impl Drop for DepotDownloader {
    fn drop(&mut self) {
        let _ = self.process.as_mut().unwrap().start_kill();
    }
}
#[derive(Error, Debug)]
pub enum Error {}
