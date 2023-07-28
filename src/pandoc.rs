use std::{fs::{File, self}, io};

use crate::{Result, Error};
use tempfile::Builder;
use tokio::process::Command;
use url::Url;

fn destination_file(url: &Url) -> String {
    tracing::info!("{url}");
    url.path_segments()
        .and_then(|segments| segments.last())
        .filter(|file| file.is_empty())
        .unwrap_or("tmp.file")
        .to_owned()
}

pub async fn save_to_file_and_convert(download_url: &str, converting_from: &str, client: &reqwest::Client) -> Result<String> {
    let tmp_dir = Builder::new().prefix("greco").tempdir()?;
    // `cd` into the temporary directory
    std::env::set_current_dir(tmp_dir.path())?;

    let response = client.get(download_url).send().await?;

    // Where the response contents will be saved to
    let dest = destination_file(response.url());
    
    // Save to file
    {
        let mut dest_file = File::create(&dest)?;
        let downloaded_buffer = response.text().await?;

        io::copy(&mut downloaded_buffer.as_bytes(), &mut dest_file)?;
    }

    std::mem::forget(tmp_dir);

    // Run `pandoc` to convert the files
    convert_file(&dest, converting_from).await
}

async fn convert_file(origin: &str, converting_from: &str) -> Result<String> {
    tracing::info!("Will call pandoc on {origin}");
    let converted = format!("{origin}.md");

    let arguments = [
        // The file to read from
        origin, 
        // Convert from whatever extension we got
        "-f", converting_from,
        // Convert into GitHub-flavored Markdown
        "-t", "gfm",
        // Where we'll be saving into
        "-o", &converted
    ];

    let exitted_successfully = Command::new("pandoc").args(arguments).spawn()?.wait().await?.success();

    if !exitted_successfully {
        return Err(Error::Pandoc);
    }

    tracing::info!("{}", fs::read_to_string(&converted).unwrap());

    fs::read_to_string(converted).map_err(Into::into)
}