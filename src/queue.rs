use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tokio::{fs, io::AsyncReadExt, io::AsyncWriteExt};

use crate::utils::get_playlists_dir;

#[derive(Deserialize, Serialize, Clone)]
pub struct Queue {
    pub items: Vec<QueueItem>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct QueueItem {
    pub file_path: String,
    pub name: String,
}
impl Queue {
    pub async fn from_queue_json(playlist_name: &str) -> Result<Self, std::io::Error> {
        let target_path = get_queue_path(playlist_name).await?;
        let mut file = match fs::File::open(target_path).await {
            Ok(v) => v,
            Err(_) => {
                let queue = Queue { items: vec![] };
                queue.to_json(playlist_name).await?;
                return Ok(queue);
            }
        };
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await?;
        Ok(serde_json::from_slice(&buf)?)
    }
    pub async fn to_json(&self, playlist_name: &str) -> Result<(), std::io::Error> {
        let target_path = get_queue_path(playlist_name).await?;
        let mut file = fs::File::create(target_path).await?;
        let bytes = serde_json::to_vec_pretty(self)?;
        file.write(&bytes).await?;
        Ok(())
    }
}

pub async fn handle_renaming_audio(
    playlist_name: &str,
    new_name: String,
    prev_name: &str,
) -> Result<(), std::io::Error> {
    let target_path = get_queue_path(playlist_name).await?;
    let mut file = fs::File::open(&target_path).await?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).await?;
    let mut serialized: Queue = serde_json::from_slice(&buf)?;
    let serialized_items = &mut serialized.items;
    if let Some(pos) = serialized_items
        .iter_mut()
        .position(|x| x.name == prev_name)
    {
        serialized_items[pos].name = new_name;
    }
    let mut file = fs::File::create(target_path).await?;
    file.write_all(&serde_json::to_vec_pretty(&serialized)?)
        .await?;
    Ok(())
}
pub async fn handle_removing_audio(
    audio_name: &str,
    playlist_name: &str,
) -> Result<(), std::io::Error> {
    let target_path = get_queue_path(playlist_name).await?;
    let mut file = fs::File::open(&target_path).await?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).await?;
    let mut serialized: Queue = serde_json::from_slice(&buf)?;
    let serialized_items = &mut serialized.items;
    if let Some(pos) = serialized_items
        .iter_mut()
        .position(|x| x.name == audio_name)
    {
        serialized_items.remove(pos);
    }
    let mut file = fs::File::create(target_path).await?;
    file.write_all(&serde_json::to_vec_pretty(&serialized)?)
        .await?;
    Ok(())
}
pub async fn handle_getting_queue(playlist_name: &str) -> Result<Vec<String>, std::io::Error> {
    let target_path = get_queue_path(playlist_name).await?;
    let mut file = fs::File::open(&target_path).await?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).await?;
    let serialized: Queue = serde_json::from_slice(&buf)?;
    let audio_names = serialized.items.into_iter().map(|v| v.name).collect();
    Ok(audio_names)
}
async fn get_queue_path(playlist_name: &str) -> Result<PathBuf, std::io::Error> {
    Ok(get_playlists_dir()
        .await?
        .join(playlist_name)
        .join("queue.json"))
}
