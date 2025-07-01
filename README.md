# plato-feed
A fetcher hook for the [Plato](https://github.com/baskerville/plato) document
reader that syncs your kobo with [Dropbox](https://dropbox.com) 

## Usage

1. Build a `plato-dropbox` binary and create a folder in Plato's bin directory for it (usually `/mnt/onboard/.adds/plato/bin/dropbox`)
2. Edit `Settings.toml` and place it alongside the binary. To get dropbox-token:
   - Open this [link](https://www.dropbox.com/oauth2/authorize?response_type=code&token_access_type=offline&client_id=qv00v86kfe5d2j2&code_challenge_method=plain&code_challenge=0000000000000000000000000000000000000000000&redirect_uri=https://plato-dropbox.truongnq.dev)
   - Paste the command in a terminal
   - Copy the line starting with dropbox-token = from your terminal
3. Add a hook to Plato's own `.adds/plato/Settings.toml` that looks like the following:
    ```toml
    [[libraries.hooks]]
    path = "Dropbox"
    program = "bin/dropbox/plato-dropbox"
    sort-method = "file-name"
    first-column = "title-and-author"
    second-column = "year"
    ```
4. Whenever the `Dropbox` folder is opened, this hook will check if there are any
articles that haven't been downloaded and will fetch them if need be.

## Building

### Docker
```shell
cargo install cross --git https://github.com/cross-rs/cross
cross build --release --target=arm-unknown-linux-musleabihf
```

### Without docker
```shell
rustup target add arm-unknown-linux-gnueabihf
cargo build --release --target=arm-unknown-linux-gnueabihf
```


## Features
Plato-dropbox currently supports only EPUB files. Only EPUB files placed directly in the application folder (Plato Fetcher) will be synced, files located in subfolders are not supported for synchronization.


# Acknowledgements
This hook is based on the work done for the following projects.

## [Plato article fetcher](https://github.com/baskerville/plato/blob/master/crates/fetcher/src/main.rs)
```
Plato -- Document reader for the Kobo e-ink devices.
Copyright (C) 2017 Bastien Dejean.

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as
published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program. If not, see <http://www.gnu.org/licenses/>.
```

## [plato-opds](https://github.com/videah/plato-opds)
```plato-opds -- OPDS syncing hook for the Plato document reader.
Copyright (C) 2023 videah

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as
published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program. If not, see <http://www.gnu.org/licenses/>.
```