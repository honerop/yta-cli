use std::path::{Path, PathBuf};

use tokio::{fs, process::Command};
use which::which;

use crate::error::Error;

pub async fn get_playlists() -> Result<Vec<String>, std::io::Error> {
    let playlist_dir = get_playlists_dir().await?;
    if !playlist_dir.exists() {
        fs::create_dir_all(playlist_dir).await?;
        Ok(Vec::new())
    } else {
        let playlists = list_subdir_names_async(playlist_dir).await?;
        Ok(playlists)
    }
}
pub async fn get_default_path() -> Result<PathBuf, std::io::Error> {
    let dir = dirs_next::home_dir()
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Couldnt find home dir",
        ))?
        .join(".yta-cli");
    fs::create_dir_all(&dir).await?;
    Ok(dir)
}

pub async fn create_playlist(playlist_name: &str) -> Result<(), std::io::Error> {
    let playlist_dir = get_playlists_dir().await?.join(playlist_name);
    fs::create_dir_all(playlist_dir).await?;
    Ok(())
}
pub async fn get_playlists_dir() -> Result<PathBuf, std::io::Error> {
    Ok(get_default_path().await?.join("playlists"))
}

#[derive(Clone)]
pub struct Paths {
    pub yt_dlp_path: PathBuf,
    pub ffmpeg_path: Option<PathBuf>,
}
pub async fn get_title_of_url(ytdlp_path: PathBuf, url: &str) -> Result<String, Error> {
    let output = Command::new(ytdlp_path)
        .arg("-e")
        .arg(url)
        .output()
        .await?
        .stdout;
    let str_output = String::from_utf8(output)?;

    Ok(str_output)
}
async fn list_subdir_names_async<P: AsRef<Path>>(dir: P) -> Result<Vec<String>, std::io::Error> {
    let mut names = Vec::new();
    let mut rd = fs::read_dir(dir).await?;
    while let Some(entry) = rd.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    Ok(names)
}

pub async fn get_programs_paths() -> Paths {
    let yt_dlp_path: PathBuf = match which("yt-dlp") {
        Ok(path) => path,
        Err(_) => {
            panic!("yt-dlp is not in PATH env variable")
        }
    };
    let ffmpeg_path: Option<PathBuf> = match which("ffmpeg") {
        Ok(path) => Some(path),
        Err(_) => {
            println!("ffmpeg is not in PATH env variable,Continuing without");
            None
        }
    };
    Paths {
        yt_dlp_path: yt_dlp_path,
        ffmpeg_path: ffmpeg_path,
    }
}
