use rodio::{Decoder, OutputStreamBuilder, Sink, Source, source::SeekError};
use std::{io::Cursor, sync::Arc, time::Duration};
use tokio::{
    fs,
    sync::{Mutex, Notify, mpsc::Receiver},
    time::Instant,
};

use crate::queue::Queue;

pub enum PlaylistControl {
    Play,
    Pause,
    Skip,
    Previous,
    SkipBy(u64),
}
#[derive(Clone, Debug)]
enum PlaybackState {
    Playing,
    Paused,
}

pub async fn play_playlist(
    playlist_name: &str,
    rx: Receiver<PlaylistControl>,
) -> Result<(), Box<dyn std::error::Error>> {
    let queue = Queue::from_queue_json(playlist_name).await?;
    let current_index = Arc::new(Mutex::new(0usize));
    let stream_handle = OutputStreamBuilder::open_default_stream()?;
    let sink = rodio::Sink::connect_new(&stream_handle.mixer());

    let sink = Arc::new(Mutex::new(sink));
    let notify = Arc::new(Notify::new());

    let index_clone = Arc::clone(&current_index);
    let sink_clone = Arc::clone(&sink);
    let notify_clone = Arc::clone(&notify);

    let state = Arc::new(Mutex::new(PlaybackState::Playing));
    let state_clone = Arc::clone(&state);

    tokio::spawn(async move {
        let _ = control_playlist(rx, index_clone, notify_clone, state_clone, sink_clone).await;
    });

    loop {
        let idx = *current_index.lock().await;
        if idx >= queue.items.len() {
            break;
        }

        let audio_bytes = fs::read(&queue.items[idx].file_path).await?;
        let cursor = Cursor::new(audio_bytes);
        let source = Decoder::try_from(cursor)?.stoppable();
        let duration = source.total_duration().unwrap_or(Duration::from_secs(5));

        {
            let sink_lock = sink.lock().await;
            sink_lock.clear();
            sink_lock.append(source);
            sink_lock.play();
        }

        let mut elapsed = Duration::ZERO;
        let mut remaining = duration;
        let mut started = Instant::now();

        loop {
            tokio::select! {
                _ = tokio::time::sleep(remaining) => {
                    *current_index.lock().await += 1;
                    break;
                }
                _ = notify.notified() => {
                    let current_state = state.lock().await.clone();

                    match current_state {
                        PlaybackState::Paused => {
                            elapsed += started.elapsed();
                            {
                                sink.lock().await.pause();
                            }

                            loop {
                                notify.notified().await;
                                if let PlaybackState::Playing = *state.lock().await {
                                    break;
                                }
                            }

                            {
                                sink.lock().await.play();
                            }

                            remaining = duration.checked_sub(elapsed).unwrap_or(Duration::from_secs(1));
                            started = Instant::now();
                            continue;
                        }
                        PlaybackState::Playing => {
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
async fn control_playlist(
    mut rx: Receiver<PlaylistControl>,
    current_index: Arc<Mutex<usize>>,
    notify: Arc<Notify>,
    state: Arc<Mutex<PlaybackState>>,
    sink: Arc<Mutex<Sink>>,
) -> Result<(), SeekError> {
    while let Some(msg) = rx.recv().await {
        match msg {
            PlaylistControl::Pause => {
                *state.lock().await = PlaybackState::Paused;
                notify.notify_one();
            }
            PlaylistControl::Play => {
                *state.lock().await = PlaybackState::Playing;
                notify.notify_one();
            }
            PlaylistControl::Skip => {
                *current_index.lock().await += 1;
                notify.notify_one();
            }
            PlaylistControl::Previous => {
                let mut idx = current_index.lock().await;
                if *idx > 0 {
                    *idx -= 1;
                }
                notify.notify_one();
            }

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
