# Rusty-Sonos
[![crates.io](https://img.shields.io/crates/v/rusty-sonos.svg)](https://crates.io/crates/rusty-sonos)
[![Documentation](https://docs.rs/rusty-sonos/badge.svg)](https://docs.rs/rusty-sonos)
[![MIT licensed](https://img.shields.io/crates/l/rusty-sonos.svg)](./LICENSE)

A library for discovering and interacting with Sonos speakers, written in Rust.

# Features

The primary functionality of this library is to provide a wrapper for speaker discovery and speaker actions.

## Discovery

To discover all speakers on the current network, use `discover_devices()`. This will return basic information about speakers (including IP addresses) about any speakers that were found. Internally, this uses the [SSDP](https://en.wikipedia.org/wiki/Simple_Service_Discovery_Protocol) protocol.

To get information about a specific speaker, given its IP, use `get_speaker_info()`.

## Speaker Interaction

Interaction with speakers is done through a single struct, `Speaker`, which has methods for all the features that are currently implemented. To use `Speaker`, you must know the speaker's IP address (refer to the discovery section for how to find this):

```rust
use rusty_sonos::speaker::Speaker;
use std::net::Ipv4Addr;

let ip_addr = Ipv4Addr::from_str("192.168.1.0").unwrap();

let speaker = Speaker::new(ip_addr).await.unwrap();

speaker.play().await.unwrap(); // plays the current track
```

A non-exhaustive list and description of speaker methods is provided below:
- `play`: starts or resumes playback of the current track
- `pause`: pauses playback of the current track
- `get_current_track`: returns information about the current track
- `set_current_uri`: sets the current track from a URI
- `get_volume`: returns the current volume
- `set_volume`: sets the volume to the given value
- `get_playback_status`: gets the playback status (playing, paused, etc.)
- `seek`: starts playback from the provided time in the track (hh:mm:ss)
- `get_queue`: returns the tracks currently in the queue
- `enter_queue`: enters the queue
- `add_track_to_queue`: adds a track to the queue
- `move_to_next_track`: skips to the next track
- `move_to_previous_track`: moves to the previous track
- `clear_queue`: removes all tracks from the queue
- `end_external_control`: ends control of the speaker by external services, such as Spotify

# Notes

Generally, the speakers' API is [UPnP](https://en.wikipedia.org/wiki/Universal_Plug_and_Play)-based

[These unofficial docs](https://sonos.svrooij.io/) were used to build this library. They contain information about various services, endpoints, and responses.