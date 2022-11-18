# QMstats
QMstats (quick machine stats) is a user friendly library to interact with wmi. The WMI connection and execution is handled on a separate thread.
The stats supported at the moment are `used memory` (and total memory), `tempreture` and `cpu util` (thou cpu util might change in the future).
My goal is to expand this in the future and add more usful information.

## Usage
To add this branch as a crate to your project, simply add the following line to your `Cargo.toml`

``` qmstats = { git = "https://github.com/AlbinDalbert/qmstats.git", branch = "v0.2.0-beta-read-only"}```

## Example
to be added...
