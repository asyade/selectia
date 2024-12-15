# Selectia [WIP]
Selectia is a music player and library manager for DJs.
It feature exeperimental features to help manage and play tracks in advanced ways.

## Features

### Precise track events detection
Commonon features such a tempo and beat grid analysis are still WIP but with promising results.
With the current technis it also possible to detect events within a track such as beats, bars, drops, etc...
As the audio analysis is done by spliting the audio into stems (i.e vocals, drums, bass, etc...) we should be able to implement some interesting features.
- Generate midi notes based on audio stems (i.e generate a midi file for drums, bass, vocals etc)
- Apply FX based on audio stems (i.e apply a reverb to vocals only)

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


## Development

### Build dependencies

- Node.js (>22 recommended)
- Yarn
- Rust nightly
- SQLite system library
- Tensorflow >2.x
  Note: Tensorflow library must be reachable from node install directory to work in dev mode
        prebuilt tensorflow.dll can be found in `packages/selectia-app/src-tauri/resources/tensorflow.dll`

<details>
    <summary>Running the app in dev mode</summary>

Install the frontend dependencies.

```bash
cd packages/selectia-app
yarn install
```

And finally run the app (this will automatically build the Rust part and watch for changes in both the Rust and Typescript parts).

```bash
cd packages/selectia-app
yarn tauri dev
```

</details>

### Rust/Typescript bindings

The Rust/Typescript bindings are generated using [ts-rs](https://github.com/Aleph-Alpha/ts-rs).
You can regenerate them by running `cargo test -p selectia-app` which will
generate the `src-tauri/bindings` folder.
