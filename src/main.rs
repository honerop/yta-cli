use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use reedline::{DefaultPromptSegment, ExternalPrinter};

use std::{path::PathBuf, sync::Arc};
use tokio::sync::{
    Mutex,
    mpsc::{self, Sender},
};

use crate::{
    download::{download_youtube_playlist, download_youtube_video_audio},
    playlist::{PlaylistControl, play_playlist},
    queue::{Queue, handle_getting_queue, handle_removing_audio, handle_renaming_audio},
    search::search_youtube,
    utils::{
        Paths, create_playlist, get_default_path, get_playlists, get_programs_paths,
        get_title_of_url,
    },
};

mod download;
mod error;
mod playlist;
mod queue;
mod search;
mod utils;

#[derive(Parser, Debug)]
#[command(
    name = "yta-cli",
    version = "1.0",
    about = "CLI REPL tool for creating and playing playlists using yt-dlp"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    ///Download URls to playlist
    Download {
        #[arg(help = "URLs of the videos")]
        urls: Vec<String>,

        #[arg(short, long)]
        playlist_name: String,
    },
    ///Download YouTube playlist to local playlist
    DownloadPlaylist {
        #[arg(help = "URL of the playlist")]
        url: String,

        #[arg(short, long, help = "Name of playlist you want to add audio")]
        playlist_name: String,
    },
    ///Search for youtube video
    Search {
        #[arg(help = "Query for youtube video")]
        query: Vec<String>,

        #[arg(short, long, default_value_t = 1, help = "How many results you want")]
        number_of_results: u16,
    },

    ///Plays playlist
    Play {
        #[arg(help = "Name of playlist you want to play")]
        name: String,
    },
    ///Creates playlist
    Create {
        #[arg(help = "Name of playlist you want to create")]
        name: String,
    },

    ///Downloads previous search result
    DownloadResult {
        #[arg(help = "name of playlist you want to add audio")]
        name: String,

        #[arg(
            short,
            long,
            default_value_t = 1,
            help = "index of previous search result"
        )]
        result_index: u16,
    },
    ///Displays all playlists
    GetPlaylists,
    ///Exit program
    Exit,
    ///Change current audio to previous
    Previous,
    ///Pause current playing audio
    Pause,
    ///Resumes current playing audio
    Resume,
    ///Skips current playing audio
    Skip,
    ///Skips current audio by some seconds
    SkipBy {
        #[arg(help = "How many seconds you want to skip by")]
        seconds: u64,
    },
    ///Renames audio in some playlist
    Rename {
        playlist_name: String,
        #[arg(long, short, help = "Current name of audio")]
        current_name: String,
        #[arg(long, short, help = "Target name of audio")]
        target_name: String,
    },
    ///Remove audio from playlist
    RemoveAudio {
        #[arg(help = "Playlist from which you want to delete audio")]
        playlist_name: String,
        #[arg(long, short, help = "Name of audio")]
        name: String,
    },
    ///Get queue of playlist
    GetQueue { playlist_name: String },
}
#[tokio::main]
async fn main() {
    println!("Welcome to yta-cli CLI REPL. Type `help` or `exit` to quit.");

    let paths = get_programs_paths().await;
    let mut control_playlist: Option<Sender<PlaylistControl>> = None;
    let last_searched_ids: Arc<Mutex<Option<Vec<String>>>> = Arc::new(Mutex::new(None));

    use reedline::{DefaultPrompt, Reedline, Signal};
    let printer = ExternalPrinter::default();
    let mut line_editor = Reedline::create().with_external_printer(printer.clone());
    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic("yta-cli".into()),
        DefaultPromptSegment::Empty,
    );

    loop {
        let input = match line_editor.read_line(&prompt) {
            Ok(Signal::Success(buffer)) => buffer,
            Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                println!("\nAborted!");
                break;
            }
            _ => {
                break;
            }
        };

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        let args = shell_words::split(trimmed);
        if args.is_err() {
            println!("Failed to parse input.");
            continue;
        }
        let default_path = get_default_path().await.unwrap();

        let cli_args = args.unwrap();
        match Cli::command()
            .no_binary_name(true)
            .try_get_matches_from(cli_args)
        {
            Ok(matches) => {
                let cli = Cli::from_arg_matches(&matches).unwrap(); // safe unwrap
                match cli.command {
                    Commands::Previous => {
                        if let Some(tx) = &control_playlist {
                            handle_sending_playlist_control(tx, PlaylistControl::Previous).await;
                        } else {
                            println!("Currently no playlist is playing");
                        }
                    }
                    Commands::Rename {
                        current_name,
                        target_name,
                        playlist_name,
                    } => {
                        if let Err(e) =
                            handle_renaming_audio(&playlist_name, target_name, &current_name).await
                        {
                            println!("Error while renaming audio: {e}");
                        }
                    }
                    Commands::RemoveAudio {
                        playlist_name,
                        name,
                    } => {
                        if let Err(e) = handle_removing_audio(&name, &playlist_name).await {
                            println!("Error while removing audio: {e}");
                        }
                    }
                    Commands::GetQueue { playlist_name } => {
                        match handle_getting_queue(&playlist_name).await {
                            Ok(v) => println!("queue: {:?}", v),
                            Err(e) => println!("Error while trying to get queue: {e}"),
                        }
                    }
                    Commands::DownloadPlaylist { url, playlist_name } => {
                        let paths = paths.clone();
                        let printer = printer.clone();
                        tokio::spawn(async move {
                            handle_download_playlist(paths, url, playlist_name, printer).await;
                        });
                    }

                    Commands::DownloadResult { result_index, name } => {
                        let arc_clone = Arc::clone(&last_searched_ids);
                        let paths = paths.clone();
                        let printer = printer.clone();
                        tokio::spawn(async move {
                            handle_download_last_search_result(
                                result_index,
                                name,
                                paths,
                                arc_clone,
                                printer,
                            )
                            .await;
                        });
                    }

                    Commands::GetPlaylists => match get_playlists().await {
                        Ok(playlists) => {
                            for playlist in playlists {
                                println!("{}", playlist);
                            }
                        }
                        Err(e) => {
                            println!("Error while trying to get playlists: {e}")
                        }
                    },
                    Commands::Skip => {
                        if let Some(tx) = &control_playlist {
                            handle_sending_playlist_control(tx, PlaylistControl::Skip).await;
                        } else {
                            println!("Currently no playlist is skippable");
                        }
                    }
                    Commands::Pause => {
                        if let Some(tx) = &control_playlist {
                            handle_sending_playlist_control(tx, PlaylistControl::Pause).await;
                        } else {
                            println!("Currently no playlist is playing");
                        }
                    }

                    Commands::Resume => {
                        if let Some(tx) = &control_playlist {
                            handle_sending_playlist_control(tx, PlaylistControl::Play).await;
                        } else {
                            println!("Currently no playlist is paused");
                        }
                    }
                    Commands::Download {
                        urls,
                        playlist_name,
                    } => {
                        let paths = paths.clone();
                        let printer = printer.clone();
                        tokio::spawn(async move {
                            handle_download(urls, playlist_name, &paths, &default_path, printer)
                                .await;
                        });
                    }
                    Commands::Search {
                        query,
                        number_of_results,
                    } => {
                        let arc_clone = Arc::clone(&last_searched_ids);
                        let printer = printer.clone();
                        tokio::spawn(async move {
                            if let Ok(results) =
                                search_youtube(&query.join(" "), number_of_results as usize).await
                            {
                                *arc_clone.lock().await =
                                    Some(results.iter().map(|x| x.id.clone()).collect());
                                for result in results {
                                    let _ =
                                        printer.sender().send(format!("Video: {}", result.title));
                                }
                            }
                        });
                    }
                    Commands::Create { name } => {
                        tokio::spawn(async move {
                            if let Ok(_) = create_playlist(&name).await {
                                println!("Successfully created playlist of name {}", name);
                            }
                        });
                    }
                    Commands::Play { name } => {
                        let (tx, rx) = mpsc::channel(10);
                        control_playlist = Some(tx);
                        tokio::spawn(async move {
                            if let Err(e) = play_playlist(&name, rx).await {
                                println!("Error when trying to play playlist: {e}")
                            }
                        });
                    }
                    Commands::SkipBy { seconds } => {
                        if let Some(tx) = &control_playlist {
                            handle_sending_playlist_control(tx, PlaylistControl::SkipBy(seconds))
                                .await;
                        } else {
                            println!("Currently no playlist is skippable");
                        }
                    }
                    Commands::Exit => {
                        println!("Goodbye!");
                        break;
                    }
                }
            }
            Err(err) => {
                use clap::error::ErrorKind;
                let msg = match err.kind() {
                    ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => err.to_string(),
                    _ => format!("Error: {err}"),
                };

                println!("{}", msg);
            }
        }
    }
}
async fn handle_sending_playlist_control(
    tx: &Sender<PlaylistControl>,
    playlist_control: PlaylistControl,
) {
    let control_str = display_playlist_control(&playlist_control);
    if let Err(e) = tx.send(playlist_control).await {
        println!("Error while trying to: {control_str},error: {e}")
    }
}
fn display_playlist_control(playlist_control: &PlaylistControl) -> String {
    match playlist_control {
        PlaylistControl::Previous => "Previous".into(),
        PlaylistControl::Skip => "Skip".into(),
        PlaylistControl::Pause => "Pause".into(),
        PlaylistControl::Play => "Play".into(),
        PlaylistControl::SkipBy(v) => format!("Skip By {}", v),
    }
}

async fn handle_download(
    urls: Vec<String>,
    playlist_name: String,
    paths: &Paths,
    default_path: &PathBuf,
    printer: ExternalPrinter<String>,
) {
    let sender = printer.sender();
    let mut queue = match Queue::from_queue_json(&playlist_name).await {
        Ok(q) => q,
        Err(e) => {
            println!("Failed to load queue: {e}");
            return;
        }
    };

    for url in urls {
        let paths = paths.clone();
        let title = match get_title_of_url(paths.yt_dlp_path.clone(), &url).await {
            Ok(t) => t,
            Err(e) => {
                let _ = sender.send(format!("Failed to get title for URL: {e}"));
                let _ = sender.send(format!("Failed to get title for URL: {e}"));
                continue;
            }
        };

        let filename = format!("{}.mp3", sanitize_filename::sanitize(title.trim()));
        let output_path = default_path
            .join("playlists")
            .join(&playlist_name)
            .join(&filename);

        match download_youtube_video_audio(paths.clone(), &url, output_path.clone()).await {
            Ok(_) => {
                queue.items.push(queue::QueueItem {
                    file_path: output_path.to_string_lossy().to_string(),
                    name: title,
                });
                if let Err(e) = queue.to_json(&playlist_name).await {
                    let _ = sender.send(format!(
                        "Failed to save json of playlist: {playlist_name},error: {e}"
                    ));
                    return;
                }
                let _ = sender.send(format!("Downloaded: {filename}"));
            }
            Err(e) => {
                let _ = sender.send(format!("Download failed: {e}"));
            }
        }
    }
}

pub async fn handle_download_last_search_result(
    result_index: u16,
    name: String,
    paths: Paths,
    last_searched_ids: Arc<Mutex<Option<Vec<String>>>>,
    printer: ExternalPrinter<String>,
) {
    let sender = printer.sender();

    let ids_lock = last_searched_ids.lock().await;
    let Some(ids) = &*ids_lock else {
        let _ = sender.send("No previous search results found.".into());
        return;
    };

    let selected_idx = (result_index as usize).saturating_sub(1);
    if selected_idx >= ids.len() {
        let _ = sender.send(format!(
            "Invalid index. Please choose between 1 and {}.",
            ids.len()
        ));
        return;
    }

    let selected_id = ids[selected_idx].clone();
    let title = match get_title_of_url(paths.yt_dlp_path.clone(), &selected_id).await {
        Ok(t) => t,
        Err(e) => {
            let _ = sender.send(format!("Failed to fetch video title: {e}"));
            return;
        }
    };

    let default_path = match get_default_path().await {
        Ok(p) => p,
        Err(e) => {
            let _ = sender.send(format!("Failed to get default path: {e}"));
            return;
        }
    };

    let sanitezed = sanitize_filename::sanitize(title.trim());
    let filename = format!("{}.mp3", sanitezed);
    let output_path = default_path.join("playlists").join(&name).join(&filename);

    let mut queue = match Queue::from_queue_json(&name).await {
        Ok(q) => q,
        Err(e) => {
            let _ = sender.send(format!("Failed to load playlist queue: {e}"));
            return;
        }
    };

    if let Err(e) =
        download_youtube_video_audio(paths.clone(), &selected_id, output_path.clone()).await
    {
        let _ = sender.send(format!("Download failed: {e}"));
    } else {
        queue.items.push(queue::QueueItem {
            file_path: output_path.to_string_lossy().to_string(),
            name: sanitezed,
        });

        if let Err(e) = queue.to_json(&name).await {
            let _ = sender.send(format!("Failed to save queue: {e}"));
        } else {
            let _ = sender.send(format!("Download complete: {filename}"));
        }
    }
}
pub async fn handle_download_playlist(
    paths: Paths,
    url: String,
    playlist_name: String,
    printer: ExternalPrinter<String>,
) {
    let sender = printer.sender();
    let mut queue = match Queue::from_queue_json(&playlist_name).await {
        Ok(q) => q,
        Err(e) => {
            let _ = sender.send(format!("Failed to load playlist queue: {e}"));
            return;
        }
    };

    match download_youtube_playlist(paths, &url, &playlist_name, &mut queue).await {
        Ok(_) => {
            if let Err(e) = queue.to_json(&playlist_name).await {
                let _ = sender.send(format!("Failed to save updated queue: {e}"));
            } else {
                let _ = sender.send(format!("Successfully downloaded playlist: {playlist_name}"));
            }
        }
        Err(e) => {
            let _ = sender.send(format!("Failed to download YouTube playlist: {e}"));
        }
    }
}
