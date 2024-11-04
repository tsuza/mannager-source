use std::{fs, io::Cursor, path::Path};

use flate2::read::GzDecoder;
use scraper::{Html, Selector};

use super::{get_arg_game_name, Error, ExtractError, SourceAppIDs, SourceEngineVersion};

pub struct MetamodDownloader;

const METAMOD_VERSIONS_URL: &str = "https://mms.alliedmods.net/mmsdrop";

#[cfg(target_os = "windows")]
const TARGET_OS: &str = "windows";

#[cfg(target_os = "linux")]
const TARGET_OS: &str = "linux";

#[derive(Debug, Clone)]
pub enum MetamodBranch {
    Stable,
    Dev,
}

impl MetamodDownloader {
    pub async fn download(
        path: impl AsRef<Path>,
        game: &SourceAppIDs,
        branch: &MetamodBranch,
        source_version: &SourceEngineVersion,
    ) -> Result<(), Error> {
        let version = get_latest_metamod_version(branch, source_version).await?;

        let path = path.as_ref();

        fs::create_dir_all(path)?;

        let latest_metamod_archive_name_url =
            format!("{METAMOD_VERSIONS_URL}/{version}/mmsource-latest-{TARGET_OS}");

        let metamod_version_name = reqwest::get(latest_metamod_archive_name_url)
            .await?
            .text()
            .await?;

        let metamod_download_url =
            format!("{}/{version}/{metamod_version_name}", METAMOD_VERSIONS_URL);

        let metamod_archive_contents = reqwest::get(metamod_download_url).await?.bytes().await?;

        let cursor = Cursor::new(metamod_archive_contents);

        #[cfg(target_os = "linux")]
        {
            let tar = GzDecoder::new(cursor);

            let mut archive = tar::Archive::new(tar);

            archive
                .unpack(
                    path.to_path_buf()
                        .join(format!("{}/", get_arg_game_name(game))),
                ) // get_arg_game_name
                .map_err(|err| ExtractError::TarError(err))?;
        }

        #[cfg(target_os = "windows")]
        {
            let mut zip =
                zip::ZipArchive::new(cursor).map_err(|err| ExtractError::ZipError(err))?;

            zip.extract(path)
                .map_err(|err| ExtractError::ZipError(err))?;
        }

        Ok(())
    }
}

/// Oh God, this is so annoying.
async fn get_latest_metamod_version(
    branch: &MetamodBranch,
    source_version: &SourceEngineVersion,
) -> Result<String, Error> {
    let page_contents = reqwest::get(METAMOD_VERSIONS_URL).await?.text().await?;

    let html = Html::parse_fragment(&page_contents);

    let a_selector = Selector::parse("a").map_err(|_| Error::UnableToFindLatestVersionError)?;

    let mut stable = 0u32;
    let mut dev = 0u32;

    for element in html.select(&a_selector).skip(5) {
        let string = element.inner_html();

        let mut split = string.trim_end_matches("/").trim().split(".");

        let major = match split.next().and_then(|next| next.parse::<u32>().ok()) {
            Some(1) => SourceEngineVersion::Source1,
            Some(2) => SourceEngineVersion::Source2,
            _ => continue,
        };

        let Some(Ok(minor)) = split.next().map(|s| s.parse::<u32>()) else {
            continue;
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

    let version: u32 = source_version.clone().into();

    match branch {
        MetamodBranch::Stable => Ok(format!("{version}.{stable}")),
        MetamodBranch::Dev => Ok(format!("{version}.{dev}")),
    }
}
