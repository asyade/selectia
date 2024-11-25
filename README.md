# Selectia [WIP]

Selectia provides a modern interface for managing your music library and organizing your music files.

## Features

### Library management

Selectia can load directories, scan audio files, index them and store the metadata in a database. It handle doublon seamlessly and handle removal of files that are no longer in the library as well.

### Tagging

The tagging feature is at the core of Selectia. It allows you to tag your music files with metadata such as artist, album, genre, bpm and more.
The tagging system is also used for search, filtering and recommendation.

#### Automatic Tagging and suggestions

When you add a new music file to your library, Selectia can automatically tag it using a combination of pattern matching and AI.
Selectia can also suggest tags based on the file name, the file path, the audio data and existing tags to allow fast and non repetitive tagging.

### Audio file analysis

Selectia can analyze audio files to extract metadata such as bpm, key, energy, danceability, etc...

### Audio editing

Selectia can edit audio files to remove silence, normalize volume, etc...
It can host VST plugins to apply custom effects to audio files as well.

### Audio file normalization

Selectia can normalize audio files to ensure consistent playback quality across your library.


### Audio file conversion

Selectia can convert audio files to different formats to ensure compatibility across your library.


### Upscaling

Selectia can upscale audio files to improve the playback quality.

### Playlists

<details>
    <summary>Developper guide</summary>

### Dependencies

- Node.js (>22 recommended)
- Yarn
- Rust nightly
- SQLite system library

### Running the app in dev mode

Run the app

> Make sure to have the database file `selectia/selectia.db` present otherwise run `scripts/    regenerate_db.sh` to create it.

```bash
export DATABASE_URL=sqlite://selectia/selectia.db
yarn tauri dev
```

</details>