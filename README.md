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

# 🔧 Linux Audio Backend Requirements

This project uses rodio
 for audio playback, which relies on the cpal
 library to interface with the system's audio backend.

On Linux, cpal requires development headers for at least one supported audio backend to compile. The most common option is ALSA.

To build this project on Linux, you need to install the ALSA development libraries:

✅ Install on Debian/Ubuntu:
sudo apt install libasound2-dev

✅ Install on Fedora:
sudo dnf install alsa-lib-devel


If you're using another distribution, install the equivalent of the ALSA development package.
🚀 Install
cargo install yta-cli


