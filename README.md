# Selectia [WIP]

Selectia provides a modern interface for managing your music library and
organizing your music files.

## Features

### Library management

Selectia can load directories, scan audio files, index them and store the
metadata in a database. It handle doublon seamlessly and handle removal of files
that are no longer in the library as well.

### Tagging

The tagging feature is at the core of Selectia. It allows you to tag your music
files with metadata such as artist, album, genre, bpm and more. The tagging
system is also used for search, filtering and recommendation.

#### Automatic Tagging and suggestions

When you add a new music file to your library, Selectia can automatically tag it
using a combination of pattern matching and AI. Selectia can also suggest tags
based on the file name, the file path, the audio data and existing tags to allow
fast and non repetitive tagging.

### Audio file analysis

Selectia can analyze audio files to extract metadata such as bpm, key, energy,
danceability, etc...

### Audio editing

Selectia can edit audio files to remove silence, normalize volume, etc... It can
host VST plugins to apply custom effects to audio files as well.

### Audio file normalization

Selectia can normalize audio files to ensure consistent playback quality across
your library.

### Audio file conversion

Selectia can convert audio files to different formats to ensure compatibility
across your library.

### Upscaling

Selectia can upscale audio files to improve the playback quality.

### Playlists

## Development

### Build dependencies

- Node.js (>22 recommended)
- Yarn
- Rust nightly
- SQLite system library

<details>
    <summary>Running the app in dev mode</summary>
First you want to make sure that the default database file is present and up to
date (which is the case on release).

> You can run `scripts/regenerate_db.sh` to regenerate the database from the
> latest schema.

Once done you want to update your environment to allow SQLX to find the database
which is used for schema validation.

```bash
# DATABASE_URL is relative to the root of the project
export DATABASE_URL=sqlite://selectia.db
```

> Its a good idea to ensure that your IDE/rust analyzer is aware of the
> `DATABASE_URL` environment variable.

Once done you can install the frontend dependencies.

```bash
cd selectia-app
yarn install
```

And finally run the app.

```bash
cd selectia-app
yarn tauri dev
```

</details>
