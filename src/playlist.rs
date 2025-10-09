use std::{sync::Arc, time::Duration};

use rodio::{Decoder, OutputStreamBuilder, Sink, Source, source::SeekError};
use tokio::{
    fs,
    sync::{Mutex, mpsc::Receiver},
};

use crate::queue::Queue;

pub async fn play_playlist(
    playlist_name: &str,
    rx: Receiver<PlaylistControl>,
) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        std::env::set_var("RUST_LOG", "off");
    }

    let queue = Queue::from_queue_json(playlist_name).await?;

    let stream_handle = OutputStreamBuilder::open_default_stream()?;
    let sink = rodio::Sink::connect_new(&stream_handle.mixer());
    let sink_arc = Arc::new(Mutex::new(sink));
    let sink_to_task = Arc::clone(&sink_arc);
    let task1 = tokio::spawn(async move { control_playlist(rx, sink_to_task).await });
    for item in queue.items {
        let file_path = item.file_path;

        let audio_bytes = fs::read(file_path).await?;
        let cursor = std::io::Cursor::new(audio_bytes);
        let source = Decoder::try_from(cursor)?.stoppable();
        sink_arc.lock().await.append(source);
    }

    let _ = task1.await?;
    Ok(())
}
pub enum PlaylistControl {
    Play,
    Skip,
    Pause,
    SkipBy(u64),
}

async fn control_playlist(
    mut rx: Receiver<PlaylistControl>,
    sink: Arc<Mutex<Sink>>,
) -> Result<(), SeekError> {
    while let Some(v) = rx.recv().await {
        match v {
            PlaylistControl::Pause => sink.lock().await.pause(),
            PlaylistControl::Skip => sink.lock().await.skip_one(),
            PlaylistControl::Play => sink.lock().await.play(),
            PlaylistControl::SkipBy(v) => {
                let locked = sink.lock().await;
                let current_duration = locked.get_pos();
                let next_duration = current_duration + Duration::from_secs(v);
                locked.try_seek(next_duration)?;
            }
        }
    }
    Ok(())
}
