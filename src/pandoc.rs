use std::{
    fs::{self, File},
    io::{self, Cursor},
};

use crate::{Error, Result, b64};
use tempfile::Builder;
use tokio::process::Command;

pub async fn save_to_file_and_convert(
    original_file_name: &str,
    converting_from: &str,
    b64_encoded: &str,
) -> Result<String> {
    let tmp_dir = Builder::new().prefix("greco").tempdir()?;
    // `cd` into the temporary directory
    std::env::set_current_dir(tmp_dir.path())?;

    // Save to file
    {
        let mut dest_file = File::create(original_file_name)?;
        let decoded_buffer = b64::decode(b64_encoded)?;

        io::copy(&mut Cursor::new(decoded_buffer), &mut dest_file)?;
    }

    std::mem::forget(tmp_dir);

    // Run `pandoc` to convert the files
    convert_file(&original_file_name, converting_from).await
}

async fn convert_file(origin: &str, converting_from: &str) -> Result<String> {
    tracing::info!("Will call pandoc on {origin}");
    let converted = format!("{origin}.md");

    let arguments = [
        // The file to read from
        origin,
        // Convert from whatever extension we got
        "-f",
        converting_from,
        // Convert into GitHub-flavored Markdown
        "-t",
        "gfm",
        // Where we'll be saving into
        "-o",
        &converted,
    ];

    let exitted_successfully = Command::new("pandoc")
        .args(arguments)
        .spawn()?
        .wait()
        .await?
        .success();

    if !exitted_successfully {
        return Err(Error::Pandoc);
    }

    fs::read_to_string(converted).map_err(Into::into)
}
