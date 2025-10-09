use serde::Deserialize;
use tokio::process::Command;

#[derive(Debug, Deserialize)]
pub struct Video {
    pub title: String,
    pub id: String,
}
pub async fn search_youtube(query: &str, count: usize) -> Result<Vec<Video>, std::io::Error> {
    let search_arg = format!("ytsearch{}:{}", count, query);

    let output = Command::new("yt-dlp")
        .args(&[
            "--skip-download",
            "-j", // JSON output
            &search_arg,
        ])
        .output()
        .await?;

    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "yt-dlp command failed",
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let results: Vec<Video> = stdout
        .lines()
        .filter_map(|line| serde_json::from_str::<Video>(line).ok())
        .collect();

    Ok(results)
}
