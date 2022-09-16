use std::cmp::min;
use std::fs::File;
use std::fs;
use std::io::Write;
use std::path::Path;

use reqwest::Client;
use indicatif::{ProgressBar, ProgressStyle};
use futures_util::StreamExt;

use tokio::process::Command;
use nix::unistd::Uid;

pub async fn download_file(client: &Client, url: &str, path: &str) -> Result<(), String> {
    // Reqwest setup
    let res = client
        .get(url)
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", &url)))?;
    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", &url))?;
    
    // Indicatif setup
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("\t{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .progress_chars("#>-"));
    pb.set_message(&format!("Downloading {}", url));

    // download chunks
    if Path::new(path).exists() {
        fs::remove_file(path);
    }
    let mut file = File::create(path).or(Err(format!("Failed to create file '{}'", path)))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write_all(&chunk)
            .or(Err(format!("Error while writing to file")))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(&format!("Downloaded {} to {}", url, path));
    return Ok(());
}

#[tokio::main]
async fn main() {
    // Check if root
    if !Uid::effective().is_root() {
        println!("Please run with sudo! This overwrites the current outdated Discord files!");
        std::process::exit(1);
    }
    println!("Pulling latest discord tar.gz file!");
    let url = "https://discord.com/api/download?platform=linux&format=tar.gz";
    let client = Client::new();
    let path = "/tmp/discord.tar.gz";
    // Download newest tar
    download_file(&client,&url,&path).await.expect("Download failed."); 
    // un tar file into /tmp
    let tar_status = Command::new("tar") 
        .args(["-xvf", "/tmp/discord.tar.gz", "-C", "/tmp/"])
        .status()
        .await
        .expect("tar extract failed! Is tar installed?");
    println!("process finished with: {tar_status}");
    // cp tar content to /opt (default dir in arch linux)
    let cp_status = Command::new("cp")
        .args(["-r", "/tmp/Discord/*", "/opt/Discord/"])
        .status()
        .await
        .expect("cp new discord to old discord failed.");
    println!("Completed!");
}
