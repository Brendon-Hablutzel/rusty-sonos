//! The primary struct used for interacting with Sonos speakers

use crate::{
    parsing::{
        get_error_code, get_tag_by_name, get_text, parse_current_track_xml, parse_getvolume_xml,
        parse_playback_status_xml, parse_queue_xml,
    },
    responses::{CurrentTrack, PlaybackStatus, QueueItem},
    services::Service,
    utils::{
        build_sonos_url, generate_xml, get_res_text, handle_sonos_err_code, stringify_xml_err,
    },
};
use reqwest::{self, StatusCode};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::str;
use std::str::FromStr;

const DESCRIPTION_ENDPOINT: &str = "/xml/device_description.xml";

/// A sonos speaker
pub struct Speaker {
    ip_addr: Ipv4Addr,
    uid: String,
    friendly_name: String,
    client: reqwest::Client,
}

impl Speaker {
    /// Creates a new speaker object, if a speaker is found at the specified IP address
    pub async fn new(ip: &str) -> Result<Self, String> {
        let ip_addr = Ipv4Addr::from_str(ip).map_err(|_| "Invalid IP address".to_owned())?;

        let client = reqwest::Client::new();

        let speaker_description_response = client
            .get(build_sonos_url(ip_addr, DESCRIPTION_ENDPOINT))
            .send()
            .await
            .map_err(|err| format!("Request to device failed: {err}"))?;

        // parse the above request for speaker details such as zone, name, and uid
        let status = speaker_description_response.status();

        if let reqwest::StatusCode::OK = status {
            let speaker_description_xml = get_res_text(speaker_description_response).await?;

            let parsed_xml = roxmltree::Document::parse(&speaker_description_xml)
                .map_err(|err| format!("Error parsing xml: {err}"))?;

            let uid = get_text(get_tag_by_name(&parsed_xml, "UDN")?, "No uid found")?
                .replace("uuid:", "");

            let friendly_name = get_text(
                get_tag_by_name(&parsed_xml, "friendlyName")?,
                "No friendly name found",
            )?;

            Ok(Speaker {
                ip_addr,
                uid,
                friendly_name,
                client,
            })
        } else {
            Err(format!("Device returned unsuccessful response: {}", status))
        }
    }

    /// Returns the ID of the speaker
    pub fn get_uid(&self) -> String {
        self.uid.to_owned()
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
    ) -> Result<String, String> {
        let url = build_sonos_url(self.ip_addr, service.get_endpoint());

        let xml_body =
            generate_xml(&action_name, service.get_name(), arguments).map_err(stringify_xml_err)?;

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
            .await
            .map_err(|err| format!("Error sending request: {err}"))?;

        let status = response.status();
        let body = get_res_text(response).await?;

        match status {
            StatusCode::OK => Ok(body),
            status_code => {
                match get_error_code(body) {
                    Ok(sonos_err_code) => {
                        let details = handle_sonos_err_code(&service, &sonos_err_code);
                        Err(format!("Speaker responded with {status_code}: {details}"))
                    },
                    Err(err) => Err(format!("Speaker responded with {status_code}. A more specific error code could not be found: {err}"))
                }
            }
        }
    }

    /// Starts playback of the current track on the speaker
    pub async fn play(&self) -> Result<(), String> {
        let action_name = "Play";
        let service = Service::AVTransport;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");
        arguments.insert("Speed", "1");

        let _ = self.make_request(service, action_name, arguments).await;

        Ok(())
    }

    /// Pauses playback on the speaker
    pub async fn pause(&self) -> Result<(), String> {
        let action_name = "Pause";
        let service = Service::AVTransport;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");

        let _ = self.make_request(service, action_name, arguments).await?;

        Ok(())
    }

    /// Returns information about the current track
    pub async fn get_current_track(&self) -> Result<CurrentTrack, String> {
        let action_name = "GetPositionInfo";
        let service = Service::AVTransport;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");

        let xml_response = self.make_request(service, action_name, arguments).await?;

        parse_current_track_xml(xml_response)
    }

    /// Sets the current track source to the given URI
    ///
    /// * `uri` - the URI of to the audio file to play
    pub async fn set_current_uri(&self, uri: &str) -> Result<(), String> {
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
    pub async fn get_volume(&self) -> Result<u8, String> {
        let action_name = "GetVolume";
        let service = Service::RenderingControl;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");
        arguments.insert("Channel", "Master");

        let xml_response = self.make_request(service, action_name, arguments).await?;

        parse_getvolume_xml(xml_response)
    }

    /// Changes the volume of the speaker to the specified value
    ///
    /// * `new_volume` - the volume to set the speaker to, between 0 and 100 inclusive
    pub async fn set_volume(&self, new_volume: u8) -> Result<(), String> {
        if new_volume > 100 {
            return Err("Invalid volume".to_owned());
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
    pub async fn get_playback_status(&self) -> Result<PlaybackStatus, String> {
        let action_name = "GetTransportInfo";
        let service = Service::AVTransport;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");

        let xml_response = self.make_request(service, action_name, arguments).await?;

        parse_playback_status_xml(xml_response)
    }

    /// Starts playing from the specified position in the current track
    ///
    /// * `new_position` - the position to start playing from, as hh:mm:ss
    pub async fn seek(&self, new_position: &str) -> Result<(), String> {
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
    pub async fn get_queue(&self) -> Result<Vec<QueueItem>, String> {
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

        parse_queue_xml(xml_response)
    }

    /// Start playback from the queue (you must enter the queue before playing tracks from it)
    pub async fn enter_queue(&self) -> Result<(), String> {
        let queue_uri = format!("x-rincon-queue:{}#0", &self.uid);
        self.set_current_uri(&queue_uri).await?;

        Ok(())
    }

    /// Add a track to the end of the queue
    ///
    /// * `uri` - the URI of the track to add
    pub async fn add_track_to_queue(&self, uri: &str) -> Result<(), String> {
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
    pub async fn move_to_next_track(&self) -> Result<(), String> {
        let action_name = "Next";
        let service = Service::AVTransport;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");

        let _ = self.make_request(service, action_name, arguments).await?;

        Ok(())
    }

    /// Goes back to the previous track in the queue, erroring if there are no tracks before the current one
    /// Note: this function will error if you use it before you have entered the queue
    pub async fn move_to_previous_track(&self) -> Result<(), String> {
        let action_name = "Previous";
        let service = Service::AVTransport;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");

        let _ = self.make_request(service, action_name, arguments).await?;

        Ok(())
    }

    /// Clears all tracks from the queue
    pub async fn clear_queue(&self) -> Result<(), String> {
        let action_name = "RemoveAllTracksFromQueue";
        let service = Service::AVTransport;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");

        let _ = self.make_request(service, action_name, arguments).await?;

        Ok(())
    }

    /// Cuts off connections from any third party services trying to use the speaker
    /// (Use this to stop playback from Spotify, for example)
    pub async fn end_external_control(&self) -> Result<(), String> {
        let action_name = "EndDirectControlSession";
        let service = Service::AVTransport;

        let mut arguments = HashMap::new();
        arguments.insert("InstanceID", "0");

        let _ = self.make_request(service, action_name, arguments).await?;

        Ok(())
    }
}
