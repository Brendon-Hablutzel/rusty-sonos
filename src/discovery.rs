//! Resources for learning about speakers on the current network

use std::{
    net::{IpAddr, Ipv4Addr, UdpSocket},
    time::{Duration, Instant},
};

use reqwest::StatusCode;

use crate::{
    errors::{SonosError, SpeakerError, UDPError},
    speaker::BasicSpeakerInfo,
    xml::{get_error_code, parse_description_xml},
};

const DISCOVERY_REQUEST_BODY: &str = "M-SEARCH * HTTP/1.1
HOST: 239.255.255.250:1900
MAN: ssdp:discover
MX: 1
ST: urn:schemas-upnp-org:device:ZonePlayer:1";

const DESCRIPTION_ENDPOINT: &str = "/xml/device_description.xml";

/// Returns basic information about a speaker, if one is found at the given IP address
/// * `ip_addr` - the IP of the speaker to query for information
pub async fn get_speaker_info(ip_addr: Ipv4Addr) -> Result<BasicSpeakerInfo, SpeakerError> {
    let url = format!(
        "http://{}:1400{}",
        DESCRIPTION_ENDPOINT,
        ip_addr.to_string()
    );

    let response = reqwest::get(&url).await?;

    let status = response.status();
    let xml_response = response.text().await?;

    if let StatusCode::OK = status {
        let speaker_info = parse_description_xml(xml_response, ip_addr)?;

        Ok(speaker_info)
    } else {
        let error_code = get_error_code(xml_response)?;

        Err(SpeakerError::from(SonosError::from_err_code(
            &error_code,
            &format!("HTTP status code: {}", status),
        )))
    }
}

/// Returns devices discovered on the current network within a given amount of time
/// * `search_secs` - the number of seconds for which the function will accept responses from speakers (the function will return in about this many seconds)
/// * `read_timeout` - the maximum amount of time for which the function will try and read data from a given response
pub async fn discover_devices(
    search_secs: u64,
    read_timeout: u64,
) -> Result<Vec<BasicSpeakerInfo>, UDPError> {
    let socket: UdpSocket = UdpSocket::bind("0.0.0.0:0")?;

    socket.set_broadcast(true)?;

    socket.set_read_timeout(Some(Duration::from_secs(read_timeout)))?;

    socket.send_to(DISCOVERY_REQUEST_BODY.as_bytes(), "239.255.255.250:1900")?;

    socket.send_to(DISCOVERY_REQUEST_BODY.as_bytes(), "255.255.255.255:1900")?;

    let start_time = Instant::now();

    // this buffer is large enough to hold typical speaker response
    let mut buf = [0; 1024];

    let mut discovered_speakers = Vec::new();

    loop {
        if start_time.elapsed().as_secs() > search_secs {
            break;
        }

        if let Ok((_, addr)) = socket.recv_from(&mut buf) {
            let ip_addr = addr.ip();

            if let IpAddr::V4(ip_addr) = ip_addr {
                if let Ok(info) = get_speaker_info(ip_addr).await {
                    if !discovered_speakers.contains(&info) {
                        discovered_speakers.push(info);
                    }
                }
            }
        }
    }

    Ok(discovered_speakers)
}
