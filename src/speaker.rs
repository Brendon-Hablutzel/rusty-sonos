//! Resources for connecting to and controlling speakers

use crate::{
    discovery::get_speaker_info,
    errors::{SonosError, SpeakerError},
    responses::{CurrentTrack, PlaybackStatus, QueueItem},
    services::Service,
    xml::{
        generate_xml, get_error_code, parse_current_track_xml, parse_getvolume_xml,
        parse_playback_status_xml, parse_queue_xml,
    },
};
use reqwest::{self, StatusCode};
use std::collections::HashMap;
use std::net::Ipv4Addr;

/// Represents typical speaker data
#[derive(Debug)]
pub struct BasicSpeakerInfo {
    /// The IP address of the speaker
    pub ip_addr: Ipv4Addr,
    /// Readable speaker name, usually in the form `IP - Model`
    pub friendly_name: String,
    /// The name of the room containing the speaker
    pub room_name: String,
    /// The unique ID of the speaker
    pub uuid: String,
}

impl PartialEq for BasicSpeakerInfo {
    fn eq(&self, other: &Self) -> bool {
        self.ip_addr == other.ip_addr
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

/// A sonos speaker
pub struct Speaker {
    ip_addr: Ipv4Addr,
    uuid: String,
    friendly_name: String,
    client: reqwest::Client,
}

impl Speaker {
    // can return error for:
    // - invalid ip
    // - HTTP error while sending
    /// Creates a new speaker object, if a speaker is found at the specified IP address
    pub async fn new(ip_addr: Ipv4Addr) -> Result<Self, SpeakerError> {
        let speaker = get_speaker_info(ip_addr).await?;

        let client = reqwest::Client::new();

        Ok(Speaker {
            ip_addr,
            uuid: speaker.uuid,
            friendly_name: speaker.friendly_name,
            client,
        })
    }

    /// Returns the ID of the speaker
    pub fn get_uuid(&self) -> String {
        self.uuid.to_owned()
    }

    /// Returns the IP address of the speaker
    pub fn get_ip_addr(&self) -> String {
        self.ip_addr.to_string()
    }

    /// Returns the friendly name of the speaker (typically in the form `IP - Model`)
    pub fn get_friendly_name(&self) -> String {
        self.friendly_name.to_owned()
    }

    async fn make_request(
        &self,
        service: Service,
        action_name: &str,
        arguments: HashMap<&str, &str>,
    ) -> Result<String, SpeakerError> {
        let url = format!("http://{}:1400{}", self.ip_addr, service.get_endpoint());

        let xml_body = generate_xml(&action_name, &service, arguments)?;

        let response = self
            .client
            .post(url)
            .body(xml_body)
            .header("Content-Type", "text/xml")
            .header(
                "SOAPACTION",
                format!(
                    "urn:schemas-upnp-org:service:{}#{}",
                    service.get_name(),
                    &action_name
                ),
            )
            .send()
            .await?;

        let status = response.status();
        let xml_response = response.text().await?;

        if let StatusCode::OK = status {
            Ok(xml_response)
        } else {
            let error_code = get_error_code(xml_response)?;

            Err(SpeakerError::from(SonosError::from_err_code(
                &error_code,
                &format!("HTTP status code: {}", status),
            )))
        }
    }

    /// Starts playback of the current track on the speaker
    pub async fn play(&self) -> Result<(), SpeakerError> {
        let action_name = "Play";
        let service = Service::AVTransport;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");
        arguments.insert("Speed", "1");

        let _ = self.make_request(service, action_name, arguments).await?;

        Ok(())
    }

    /// Pauses playback on the speaker
    pub async fn pause(&self) -> Result<(), SpeakerError> {
        let action_name = "Pause";
        let service = Service::AVTransport;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");

        let _ = self.make_request(service, action_name, arguments).await?;

        Ok(())
    }

    /// Returns information about the current track
    pub async fn get_current_track(&self) -> Result<CurrentTrack, SpeakerError> {
        let action_name = "GetPositionInfo";
        let service = Service::AVTransport;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");

        let xml_response = self.make_request(service, action_name, arguments).await?;

        let current_track = parse_current_track_xml(xml_response)?;

        Ok(current_track)
    }

    /// Sets the current track source to the given URI
    ///
    /// * `uri` - the URI of to the audio file to play
    pub async fn set_current_uri(&self, uri: &str) -> Result<(), SpeakerError> {
        let action_name = "SetAVTransportURI";
        let service = Service::AVTransport;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");
        arguments.insert("CurrentURI", uri);
        arguments.insert("CurrentURIMetaData", "");

        let _ = self.make_request(service, action_name, arguments).await?;

        Ok(())
    }

    /// Returns the current volume of the speaker
    pub async fn get_volume(&self) -> Result<u8, SpeakerError> {
        let action_name = "GetVolume";
        let service = Service::RenderingControl;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");
        arguments.insert("Channel", "Master");

        let xml_response = self.make_request(service, action_name, arguments).await?;

        let volume = parse_getvolume_xml(xml_response)?;

        Ok(volume)
    }

    /// Changes the volume of the speaker to the specified value
    ///
    /// * `new_volume` - the volume to set the speaker to, between 0 and 100 inclusive
    pub async fn set_volume(&self, new_volume: u8) -> Result<(), SpeakerError> {
        if new_volume > 100 {
            return Err(SpeakerError::InvalidInput(format!(
                "invalid volume: {}",
                new_volume
            )));
        };

        let new_volume = new_volume.to_string();

        let action_name = "SetVolume";
        let service = Service::RenderingControl;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");
        arguments.insert("Channel", "Master");
        arguments.insert("DesiredVolume", &new_volume);

        let _ = self.make_request(service, action_name, arguments).await?;

        Ok(())
    }

    /// Returns the current status of playback on the speaker (playing, paused, stopped, etc...)
    pub async fn get_playback_status(&self) -> Result<PlaybackStatus, SpeakerError> {
        let action_name = "GetTransportInfo";
        let service = Service::AVTransport;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");

        let xml_response = self.make_request(service, action_name, arguments).await?;

        parse_playback_status_xml(xml_response).map_err(SpeakerError::from)
    }

    /// Starts playing from the specified position in the current track
    ///
    /// * `new_position` - the position to start playing from, as hh:mm:ss
    pub async fn seek(&self, new_position: &str) -> Result<(), SpeakerError> {
        let action_name = "Seek";
        let service = Service::AVTransport;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");
        arguments.insert("Unit", "REL_TIME");
        arguments.insert("Target", new_position);

        let _ = self.make_request(service, action_name, arguments).await?;

        Ok(())
    }

    /// Returns all tracks in the queue
    pub async fn get_queue(&self) -> Result<Vec<QueueItem>, SpeakerError> {
        let action_name = "Browse";
        let service = Service::ContentDirectory;

        let mut arguments = HashMap::new();
        arguments.insert("ObjectID", "Q:0");
        arguments.insert("BrowseFlag", "BrowseDirectChildren");
        arguments.insert("Filter", "*");
        arguments.insert("StartingIndex", "0");
        arguments.insert("RequestedCount", "100");
        arguments.insert("SortCriteria", "");

        let xml_response = self.make_request(service, action_name, arguments).await?;

        parse_queue_xml(xml_response).map_err(SpeakerError::from)
    }

    /// Start playback from the queue (you must enter the queue before playing tracks from it)
    pub async fn enter_queue(&self) -> Result<(), SpeakerError> {
        let queue_uri = format!("x-rincon-queue:{}#0", &self.uuid);
        self.set_current_uri(&queue_uri).await?;

        Ok(())
    }

    /// Add a track to the end of the queue
    ///
    /// * `uri` - the URI of the track to add
    pub async fn add_track_to_queue(&self, uri: &str) -> Result<(), SpeakerError> {
        let action_name = "AddURIToQueue";
        let service = Service::AVTransport;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");
        arguments.insert("EnqueuedURI", uri);
        arguments.insert("EnqueuedURIMetaData", "");
        arguments.insert("DesiredFirstTrackNumberEnqueued", "0");
        arguments.insert("EnqueueAsNext", "0");

        let _ = self.make_request(service, action_name, arguments).await?;

        Ok(())
    }

    /// Skips to the next track in the queue, erroring if there are no tracks after the current one
    /// Note: this function will error if you use it before you have entered the queue
    pub async fn move_to_next_track(&self) -> Result<(), SpeakerError> {
        let action_name = "Next";
        let service = Service::AVTransport;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");

        let _ = self.make_request(service, action_name, arguments).await?;

        Ok(())
    }

    /// Goes back to the previous track in the queue, erroring if there are no tracks before the current one
    /// Note: this function will error if you use it before you have entered the queue
    pub async fn move_to_previous_track(&self) -> Result<(), SpeakerError> {
        let action_name = "Previous";
        let service = Service::AVTransport;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");

        let _ = self.make_request(service, action_name, arguments).await?;

        Ok(())
    }

    /// Clears all tracks from the queue
    pub async fn clear_queue(&self) -> Result<(), SpeakerError> {
        let action_name = "RemoveAllTracksFromQueue";
        let service = Service::AVTransport;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");

        let _ = self.make_request(service, action_name, arguments).await?;

        Ok(())
    }

    /// Cuts off connections from any third party services trying to use the speaker
    /// (Use this to stop playback from Spotify, for example)
    pub async fn end_external_control(&self) -> Result<(), SpeakerError> {
        let action_name = "EndDirectControlSession";
        let service = Service::AVTransport;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");

        let _ = self.make_request(service, action_name, arguments).await?;

        Ok(())
    }
}
