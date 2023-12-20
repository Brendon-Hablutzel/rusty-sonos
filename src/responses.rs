//! Structs and enums used while parsing speaker data

use std::fmt;

/// The track currently being played
#[derive(Debug)]
pub struct CurrentTrack {
    /// The current time of the track, in hh:mm:ss
    pub position: String,
    /// The total length of the track, in hh:mm:ss
    pub duration: String,
    /// The source URI of the track
    pub uri: String,
    /// The title of the track
    pub title: Option<String>,
    /// The artist/creator of the track
    pub artist: Option<String>,
}

/// The current playback state of the speaker
#[derive(Debug)]
pub enum PlaybackState {
    /// Playback is stopped
    Stopped,
    /// The track is currently playing
    Playing,
    /// The track is paused
    Paused,
    /// The track is transitioning between playback states
    Transitioning,
}

impl PlaybackState {
    pub(crate) fn new(state_str: &str) -> Result<Self, String> {
        match state_str {
            "STOPPED" => Ok(Self::Stopped),
            "PLAYING" => Ok(Self::Playing),
            "PAUSED_PLAYBACK" => Ok(Self::Paused),
            "TRANSITIONING" => Ok(Self::Transitioning),
            _ => Err("Invalid state".to_owned()),
        }
    }
}

impl fmt::Display for PlaybackState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            PlaybackState::Stopped => "Stopped",
            PlaybackState::Playing => "Playing",
            PlaybackState::Paused => "Paused",
            PlaybackState::Transitioning => "Transitioning",
        };
        write!(f, "{output}")
    }
}

/// Information about playback on the speaker
#[derive(Debug)]
pub struct PlaybackStatus {
    /// The current playback state on the speaker (playing, paused, etc...)
    pub playback_state: PlaybackState,
    /// Uncertain purpose
    pub status: String,
}

/// A track in the queue
#[derive(Debug)]
pub struct QueueItem {
    /// The length of the track, as hh:mm:ss
    pub duration: Option<String>,
    /// The source URI of the track
    pub uri: String,
    /// The title of the track
    pub title: Option<String>,
    /// The artist/creator of the track
    pub artist: Option<String>,
}
