use reqwest::{self};
use snafu::ResultExt;
use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::process::{Child, ChildStdout, Command};

use super::{
    ArchiveExtractionSnafu, DirectoryCreationSnafu, DownloadRequestSnafu, Error, SpawnProcessSnafu,
    ZipSnafu,
};

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
        let mut process = Command::new(&self.depotdownloader_path);

        process
            .args(["-app", &appid.to_string()])
            .args(["-dir", path])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        #[cfg(target_os = "windows")]
        process.creation_flags(0x08000000);

        let mut process = process.spawn().context(SpawnProcessSnafu)?;

        let stdout = process.stdout.take();

        self.process = Some(process);

        Ok(stdout)
    }
}

async fn download_file(path: &PathBuf) -> Result<(), Error> {
    fs::create_dir_all(path).context(DirectoryCreationSnafu)?;

    let steamcmd_contents = reqwest::get(DEPOTDOWNLOADER_URL)
        .await
        .context(DownloadRequestSnafu)?
        .bytes()
        .await
        .context(DownloadRequestSnafu)?;

    let cursor = Cursor::new(steamcmd_contents);

    let mut zip = zip::ZipArchive::new(cursor)
        .context(ZipSnafu)
        .context(ArchiveExtractionSnafu)?;

    zip.extract(path)
        .context(ZipSnafu)
        .context(ArchiveExtractionSnafu)?;

    Ok(())
}

impl Drop for DepotDownloader {
    fn drop(&mut self) {
        let _ = self.process.as_mut().unwrap().start_kill();
    }
}
