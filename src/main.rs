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
        fs::remove_file(path).expect("Removal of {path} failed.");
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
    println!("Extract tar file...");
    Command::new("tar") 
        .args(["-xvf", "/tmp/discord.tar.gz", "-C", "/tmp/"])
        .output()
        .await
        .expect("tar extract failed! Is tar installed?");
    println!("Extracted!");
    // cp tar content to /opt (default dir in arch linux)
    println!("Checking if /opt/discord exists...");
    if Path::new("/opt/discord/").exists() {
        println!("IT DOES! Deleting...");
        fs::remove_dir_all("/opt/discord/").expect("Removal of /opt/discord failed");
    }
    println!("Copying new discord files to original folder...");
    Command::new("sudo")
        .args(["cp","-R", "/tmp/Discord/", "/opt/discord/"])
        .output()
        .await
        .expect("cp new discord to old discord failed.");
    println!("\tCopied!");
    println!("Making discord binary executable...");
    Command::new("sudo")
        .args(["chmod","+x","/opt/discord/Discord"])
        .output()
        .await
        .expect("Failed to set discord as an executable");
    println!("\tSuccess!");
    println!("Linking /opt/discord to /usr/share ...");
    Command::new("sudo")
        .args(["ln", "-s", "/opt/discord", "/usr/share/discord"])
        .output()
        .await
        .expect("ln /opt/discord to /usr/share/discord failed");
    println!("\tLinked!");
    println!("Cleaning up...");
    fs::remove_dir_all("/tmp/Discord").expect("Removal of /tmp/discord failed");
    fs::remove_file("/tmp/discord.tar.gz").expect("Removal of /tmp/discord.tar.gz failed");
    println!("Removed /tmp files!");
    println!("Completed!");
}
