use reqwest::header;
use reqwest::blocking::Client;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut headers = header::HeaderMap::new();
    //headers.insert("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:104.0) Gecko/20100101 Firefox/104.0".parse().unwrap());
    //headers.insert("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8".parse().unwrap());
    //headers.insert("Accept-Language", "en-US,en;q=0.5".parse().unwrap());
    //headers.insert("Accept-Encoding", "gzip, deflate, br".parse().unwrap());
    //headers.insert("Connection", "keep-alive".parse().unwrap());

    let client = Client::new();
    let res = client.get("https://discord.com/api/download?platform=linux&format=tar.gz")
        .headers(headers)
        .send()?
        .text()?;
    //res

    Ok(())
}
