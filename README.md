🎵 yta-cli

A YouTube-powered CLI REPL music tool built in Rust using yt-dlp and ffmpeg. Create playlists, download audio, search, and play music — all from your terminal.

✨ Features

✅ Download individual YouTube videos as audio

✅ Download entire YouTube playlists

✅ Save and organize audio files in named playlists

✅ Search YouTube videos from the CLI

✅ Play playlists with controls: pause, resume, skip, skip by seconds

✅ REPL interface (interactive prompt)

✅ Persistent playlist queue stored in JSON

📦 Requirements

yt-dlp
 (must be in PATH)

ffmpeg
 (optional but recommended for better audio handling)

Install both via your package manager:

# Debian / Ubuntu
sudo apt install yt-dlp ffmpeg

# macOS (Homebrew)
brew install yt-dlp ffmpeg

🚀 Install
cargo install yta-cli

🧠 Commands
Command	Description
create <name>	Create a new playlist
download <url> ...	Download audio from YouTube URLs
download-playlist	Download entire YouTube playlist
play <name>	Play audio from a playlist
pause / resume	Pause / resume audio playback
skip / skip-by 	Skip current song / skip forward N seconds
search <query>	Search YouTube and show results
download-result	Download from the last search result
get-playlists	List all playlists
exit	Exit the CLI
