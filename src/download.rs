use std::path::PathBuf;

use tokio::{fs, process::Command};

use crate::{
    error::Error,
    queue::{Queue, QueueItem},
    utils::{Paths, get_playlists_dir, get_title_of_url},
};

pub async fn download_youtube_video_audio(
    paths: Paths,
    url: &str,
    output_path: PathBuf,
) -> Result<(), std::io::Error> {
    let mut cmd = Command::new(paths.yt_dlp_path);
    if let Some(path) = paths.ffmpeg_path {
        cmd.arg("--ffmpeg-location").arg(path);
    }
    cmd.arg("--quiet")
        .arg("-x")
        .arg("--audio-format")
        .arg("mp3")
        .arg("-o")
        .arg(output_path)
        .arg(url);

    cmd.status().await?;

    Ok(())
}
pub async fn download_youtube_playlist(
    paths: Paths,
    playlist_url: &str,
    playlist_name: &str,
    queue: &mut Queue,
) -> Result<(), Error> {
    let playlist_path = get_playlists_dir().await?.join(playlist_name);

    fs::create_dir_all(&playlist_path).await?;

    let output = Command::new(&paths.yt_dlp_path)
        .arg("--flat-playlist")
        .arg("-J")
        .arg(playlist_url)
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)?;

    let entries = parsed["entries"]
        .as_array()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Invalid playlist JSON"))?;

    for entry in entries.iter() {
        if let Some(video_id) = entry["id"].as_str() {
            let paths = paths.clone();
            let url = format!("https://www.youtube.com/watch?v={}", video_id);

            let title = get_title_of_url(paths.yt_dlp_path.clone(), &url).await?;
            let clean = sanitize_filename::sanitize(title.trim());
            let filename = format!("{}.mp3", clean);
            let output_path = playlist_path.join(&filename);

            queue.items.push(QueueItem {
                file_path: output_path.to_string_lossy().to_string(),
                name: clean,
            });

            download_youtube_video_audio(paths, &url, output_path).await?;
        }
    }

    Ok(())
}
