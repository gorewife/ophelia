# Roadmap

Ophelia is a lightweight, cross-platform download manager with a modern UI and a plugin system that lets anyone extend its functionality

## Core

- [ ] Download queue with pause, resume, cancel
- [ ] Concurrent downloads with configurable limits
- [ ] Progress tracking (speed, ETA, bytes transferred)
- [ ] Storage usage tracking
- [ ] System notifications on completion

## UI

- [ ] Download list with status indicators
- [ ] Active / Finished / Paused filters
- [ ] Add download modal (URL input, destination picker)
- [ ] Settings panel

## File management

- [ ] **Auto-sort** — move completed downloads into subfolders by file type (video, audio, image, document, archive)
- [ ] **Auto-rename** — clean up filenames post-download: strip URL encoding, remove hashes/UUIDs, use file metadata where available (ID3 tags, EXIF, video container info)

## Plugins

- [ ] **yt-dlp** — seamless integration for video/audio downloads with format selection and metadata extraction
- [ ] **Browser extension** — send URLs directly from the browser to Ophelia

## Plugin API

- [ ] Public plugin trait for third-party plugins
- [ ] Plugin discovery and loading
- [ ] Plugin settings UI

## Platform

- [ ] macOS
- [ ] Linux
- [ ] Windows
