use core::error;
use std::{fs, io::Cursor, path::Path};

use flate2::read::GzDecoder;
use scraper::{Html, Selector};

use super::SourceEngineVersion;

pub struct SourcemodDownloader;

#[derive(Debug, Clone, PartialEq)]
pub enum SourcemodBranch {
    Stable,
    Dev,
}

const SOURCEMOD_VERSIONS_URL: &str = "https://sm.alliedmods.net/smdrop";

impl SourcemodDownloader {
    pub async fn download(
        path: impl AsRef<Path>,
        branch: &SourcemodBranch,
        source_version: &SourceEngineVersion,
    ) -> Result<(), Box<dyn error::Error>> {
        let version = get_latest_sourcemod_version(branch, source_version).await?;

        println!("Inside Sourcemod");

        let path = path.as_ref();

        fs::create_dir_all(path)?;

        let latest_sourcemod_archive_name_url = format!(
            "{}/{version}/sourcemod-latest-linux",
            SOURCEMOD_VERSIONS_URL
        );

        let sourcemod_version_name = reqwest::get(latest_sourcemod_archive_name_url)
            .await?
            .text()
            .await?;

        let sourcemod_download_url = format!(
            "{}/{version}/{sourcemod_version_name}",
            SOURCEMOD_VERSIONS_URL
        );

        let sourcemod_archive_contents =
            reqwest::get(sourcemod_download_url).await?.bytes().await?;

        let cursor = Cursor::new(sourcemod_archive_contents);

        let tar = GzDecoder::new(cursor);

        let mut archive = tar::Archive::new(tar);

        archive.unpack(path.to_path_buf().join("tf/"))?;

        println!("Done Sourcemod");

        Ok(())
    }
}

/// Oh God, this is so annoying.
async fn get_latest_sourcemod_version(
    branch: &SourcemodBranch,
    source_version: &SourceEngineVersion,
) -> Result<String, Box<dyn error::Error>> {
    let page_contents = reqwest::get(SOURCEMOD_VERSIONS_URL).await?.text().await?;

    let html = Html::parse_fragment(&page_contents);

    let a_selector = Selector::parse("a")?;

    let mut stable = 0u32;
    let mut dev = 0u32;

    for element in html.select(&a_selector).skip(1) {
        println!("{}", element.inner_html().trim_end_matches("/"));

        let string = element.inner_html();

        let (major, minor) = {
            let mut split = string.trim_end_matches("/").trim().split(".");

            (
                match split.nth(0).unwrap().parse::<u32>()? {
                    1 => SourceEngineVersion::Source1,
                    2 => SourceEngineVersion::Source2,
                    _ => panic!(),
                },
                split.nth(0).unwrap().parse::<u32>()?,
            )
        };

        if &major != source_version {
            continue;
        }

        if minor > stable {
            stable = dev;
        }

        if minor > dev {
            dev = minor;
        }
    }

    println!("Stable version: {stable}");
    println!("Dev version: {dev}");

    let version: u32 = source_version.clone().into();

    match branch {
        SourcemodBranch::Stable => Ok(format!("{version}.{stable}")),
        SourcemodBranch::Dev => Ok(format!("{version}.{dev}")),
    }
}
