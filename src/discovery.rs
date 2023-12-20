//! Resources for learning about speakers on the current network

use std::{
    net::{Ipv4Addr, UdpSocket},
    str::FromStr,
    time::{Duration, Instant},
};

use crate::{
    parsing::{get_tag_by_name, get_text},
    utils::get_res_text,
};

const DISCOVERY_REQUEST_BODY: &str = "M-SEARCH * HTTP/1.1
HOST: 239.255.255.250:1900
MAN: ssdp:discover
MX: 1
ST: urn:schemas-upnp-org:device:ZonePlayer:1";

/// Represents typical speaker data
#[derive(Debug)]
pub struct BasicSpeakerInfo {
    /// The IP address of the speaker
    pub ip_addr: Ipv4Addr,
    /// Readable speaker name, usually in the form `IP - Model`
    pub friendly_name: String,
    /// The name of the room containing the speaker
    pub room_name: String,
}

impl PartialEq for BasicSpeakerInfo {
    fn eq(&self, other: &Self) -> bool {
        self.ip_addr == other.ip_addr
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

/// Returns basic information about a speaker, if one is found at the given IP address
///
/// * `ip_addr` - the IP of the speaker to query for information
pub async fn get_speaker_info(ip_addr: &str) -> Result<BasicSpeakerInfo, String> {
    let url = format!(
        "http://{}:1400/xml/device_description.xml",
        ip_addr.to_string()
    );

    let res = reqwest::get(&url).await.map_err(|err| err.to_string())?;

    let xml_response = get_res_text(res).await?;

    let parsed_xml = roxmltree::Document::parse(&xml_response).map_err(|err| err.to_string())?;

    let friendly_name = get_text(
        get_tag_by_name(&parsed_xml, "friendlyName")?,
        "No friendly name found",
    )?;

    let room_name = get_text(
        get_tag_by_name(&parsed_xml, "roomName")?,
        "No room name found",
    )?;

    Ok(BasicSpeakerInfo {
        ip_addr: Ipv4Addr::from_str(ip_addr)
            .expect("If a speaker exists, then its IP address should be valid"),
        friendly_name,
        room_name,
    })
}

/// Returns devices discovered on the current network within a given amount of time
/// * `search_secs` - the number of seconds for which the function will accept responses from speakers (the function will return in about this many seconds)
/// * `read_timeout` - the maximum amount of time for which the function will try and read data from a given response
pub async fn discover_devices(
    search_secs: u64,
    read_timeout: u64,
) -> Result<Vec<BasicSpeakerInfo>, String> {
    let socket: UdpSocket =
        UdpSocket::bind("0.0.0.0:0").expect("Should be able to create a UDP socket");

    socket
        .set_broadcast(true)
        .expect("Should be able to enable broadcast");

    socket
        .set_read_timeout(Some(Duration::from_secs(read_timeout)))
        .expect("Should be able to set socket read timeout");

    socket
        .send_to(DISCOVERY_REQUEST_BODY.as_bytes(), "239.255.255.250:1900")
        .map_err(|err| err.to_string())?;

    socket
        .send_to(DISCOVERY_REQUEST_BODY.as_bytes(), "255.255.255.255:1900")
        .map_err(|err| err.to_string())?;

    let start_time = Instant::now();

    // this buffer is large enough to hold typical speaker response
    let mut buf = [0; 1024];

    let mut discovered_speakers = Vec::new();

    loop {
        if start_time.elapsed().as_secs() > search_secs {
            break;
        }

        if let Ok((_, addr)) = socket.recv_from(&mut buf) {
            let addr = addr.to_string().replace(&format!(":{}", addr.port()), "");

            if let Ok(info) = get_speaker_info(&addr).await {
                if !discovered_speakers.contains(&info) {
                    discovered_speakers.push(info);
                }
            }
        }
    }

    Ok(discovered_speakers)
}
