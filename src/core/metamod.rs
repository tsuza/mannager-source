use core::error;
use std::{fs, io::Cursor, path::Path};

use flate2::read::GzDecoder;
use scraper::{Html, Selector};

use super::SourceEngineVersion;

pub struct MetamodDownloader;

const METAMOD_VERSIONS_URL: &str = "https://mms.alliedmods.net/mmsdrop";

#[derive(Debug, Clone)]
pub enum MetamodBranch {
    Stable,
    Dev,
}

impl MetamodDownloader {
    pub async fn download(
        path: impl AsRef<Path>,
        branch: &MetamodBranch,
        source_version: &SourceEngineVersion,
    ) -> Result<(), Box<dyn error::Error>> {
        println!("Inside Metamod");

        let version = get_latest_metamod_version(branch, source_version).await?;

        let path = path.as_ref();

        fs::create_dir_all(path)?;

        let latest_metamod_archive_name_url =
            format!("{}/{version}/mmsource-latest-linux", METAMOD_VERSIONS_URL);

        let metamod_version_name = reqwest::get(latest_metamod_archive_name_url)
            .await?
            .text()
            .await?;

        let metamod_download_url =
            format!("{}/{version}/{metamod_version_name}", METAMOD_VERSIONS_URL);

        let metamod_archive_contents = reqwest::get(metamod_download_url).await?.bytes().await?;

        let cursor = Cursor::new(metamod_archive_contents);

        let tar = GzDecoder::new(cursor);

        let mut archive = tar::Archive::new(tar);

        archive.unpack(path.to_path_buf().join("tf/"))?;

        println!("Done Metamod");

        Ok(())
    }
}

/// Oh God, this is so annoying.
async fn get_latest_metamod_version(
    branch: &MetamodBranch,
    source_version: &SourceEngineVersion,
) -> Result<String, Box<dyn error::Error>> {
    let page_contents = reqwest::get(METAMOD_VERSIONS_URL).await?.text().await?;

    let html = Html::parse_fragment(&page_contents);

    let a_selector = Selector::parse("a")?;

    let mut stable = 0u32;
    let mut dev = 0u32;

    for element in html.select(&a_selector).skip(5) {
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
        MetamodBranch::Stable => Ok(format!("{version}.{stable}")),
        MetamodBranch::Dev => Ok(format!("{version}.{dev}")),
    }
}
