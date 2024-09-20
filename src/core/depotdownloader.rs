use core::error;
use decompress::{self, ExtractOpts, ExtractOptsBuilder};
use reqwest::{self};
use std::{
    fs,
    path::{self, PathBuf},
    process::Stdio,
};
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
    pub async fn new(path: &str) -> Result<Self, Box<dyn error::Error>> {
        if !path::Path::new(&path).try_exists()? {
            download_file(path).await?;
        }

        let executable_path = format!("{path}/DepotDownloader");

        /*
        let mut steamcmd_process = Command::new(executable_path)
            .args(["+login", "anonymous"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        */

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

async fn download_file(path: &str) -> Result<(), Box<dyn error::Error>> {
    fs::create_dir_all(path)?;

    let steamcmd_contents = reqwest::get(DEPOTDOWNLOADER_URL).await?.bytes().await?;

    tokio::fs::write(
        format!("{path}/depotdownloader_compressed.zip"),
        steamcmd_contents,
    )
    .await?;

    decompress::decompress(
        format!("{path}/depotdownloader_compressed.zip"),
        format!("{path}"),
        &ExtractOptsBuilder::default().build()?,
    )?;

    Ok(())
}

impl Drop for DepotDownloader {
    fn drop(&mut self) {
        self.process.as_mut().unwrap().kill();
    }
}
