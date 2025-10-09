use serde::{Deserialize, Serialize};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::utils::get_default_path;

#[derive(Deserialize, Serialize)]
pub struct Queue {
    pub items: Vec<QueueItem>,
}

#[derive(Deserialize, Serialize)]
pub struct QueueItem {
    pub file_path: String,
}
impl Queue {
    pub async fn from_queue_json(playlist_name: &str) -> Result<Self, std::io::Error> {
        let target_path = get_default_path()
            .await?
            .join("playlists")
            .join(playlist_name)
            .join("queue.json");
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
        Ok(serde_json::from_slice(&buf).unwrap())
    }
    pub async fn to_json(&self, playlist_name: &str) -> Result<(), std::io::Error> {
        let default_path = get_default_path().await?;
        let target_path = default_path
            .join("playlists")
            .join(playlist_name)
            .join("queue.json");
        let mut file = fs::File::create(target_path).await?;
        let bytes = serde_json::to_vec_pretty(self)?;
        file.write(&bytes).await?;
        Ok(())
    }
}
