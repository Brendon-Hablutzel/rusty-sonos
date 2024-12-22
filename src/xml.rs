use std::{collections::HashMap, net::Ipv4Addr};

use crate::{
    errors::XMLError,
    responses::{CurrentTrack, PlaybackState, PlaybackStatus, QueueItem},
    services::Service,
    speaker::BasicSpeakerInfo,
};
use roxmltree::{Document, Node};
use xml_builder::{self, XMLBuilder, XMLElement, XMLVersion};

pub(crate) fn get_tag_by_name<'a>(
    parsed_xml: &'a Document,
    tag_name: &str,
) -> Result<roxmltree::Node<'a, 'a>, XMLError> {
    let tag = parsed_xml
        .descendants()
        .find(|n| n.has_tag_name(tag_name))
        .ok_or(XMLError::ElementNotFound(tag_name.to_string()))?;

    Ok(tag)
}

pub(crate) fn get_tag_by_name_node<'a>(
    parsed_xml: &'a Node,
    tag_name: &str,
) -> Result<roxmltree::Node<'a, 'a>, XMLError> {
    let tag = parsed_xml
        .descendants()
        .find(|n| n.has_tag_name(tag_name))
        .ok_or(XMLError::ElementNotFound(tag_name.to_string()))?;

    Ok(tag)
}

pub(crate) fn get_text(node: roxmltree::Node<'_, '_>) -> Result<String, XMLError> {
    node.text()
        .ok_or(XMLError::ElementNotFound(
            node.tag_name().name().to_string(),
        ))
        .map(|text| text.to_owned())
}

fn clean_response_xml(xml: String) -> String {
    xml.replace("<s:", "<")
        .replace("</s:", "</")
        .replace(
            r#" xmlns:s="http://schemas.xmlsoap.org/soap/envelope/""#,
            "",
        )
        .replace(
            r#" s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/""#,
            "",
        )
        .replace("<u:", "<")
        .replace("</u:", "</")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace(r#"xmlns:dc="http://purl.org/dc/elements/1.1/""#, "")
        .replace(
            r#" xmlns:upnp="urn:schemas-upnp-org:metadata-1-0/upnp/""#,
            "",
        )
        .replace(
            r#" xmlns:r="urn:schemas-rinconnetworks-com:metadata-1-0/""#,
            "",
        )
        .replace(
            r#" xmlns="urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/""#,
            "",
        )
        .replace("<dc:", "<")
        .replace("<upnp:", "<")
        .replace("<r:", "<")
        .replace("</dc:", "</")
        .replace("</upnp:", "</")
        .replace("</r:", "</")
}

pub(crate) fn parse_queue_xml(xml: String) -> Result<Vec<QueueItem>, XMLError> {
    let xml = clean_response_xml(xml);

    let parsed_xml = roxmltree::Document::parse(&xml).map_err(XMLError::from)?;

    let items: Result<Vec<QueueItem>, XMLError> = parsed_xml
        .descendants()
        .filter(|node| node.has_tag_name("item"))
        .map(|item| parse_queue_item(item))
        .collect();

    items
}

fn parse_queue_item(item: roxmltree::Node) -> Result<QueueItem, XMLError> {
    let res = get_tag_by_name_node(&item, "res")?;

    let title = get_tag_by_name_node(&item, "title")?
        .text()
        .map(str::to_string);

    let artist = get_tag_by_name_node(&item, "artist")?
        .text()
        .map(str::to_string);

    let duration = res.attribute("duration").map(str::to_string);

    let uri = get_text(res)?.to_owned();

    Ok(QueueItem {
        duration,
        uri,
        title,
        artist,
    })
}

pub(crate) fn parse_current_track_xml(xml: String) -> Result<CurrentTrack, XMLError> {
    let xml = clean_response_xml(xml);

    let parsed_xml = roxmltree::Document::parse(&xml)?;

    let duration = get_text(get_tag_by_name(&parsed_xml, "TrackDuration")?)?;

    let uri = get_text(get_tag_by_name(&parsed_xml, "TrackURI")?)?;

    let title = get_tag_by_name(&parsed_xml, "title")?
        .text()
        .map(str::to_string);

    let artist = get_tag_by_name(&parsed_xml, "creator")
        .ok()
        .and_then(|node| node.text())
        .map(str::to_string);

    let position = get_text(get_tag_by_name(&parsed_xml, "RelTime")?)?;

    Ok(CurrentTrack {
        position,
        duration,
        uri,
        title,
        artist,
    })
}

pub(crate) fn parse_getvolume_xml(xml: String) -> Result<u8, XMLError> {
    let xml = clean_response_xml(xml);

    let parsed_xml = roxmltree::Document::parse(&xml)?;

    let volume = get_text(get_tag_by_name(&parsed_xml, "CurrentVolume")?)?;

    volume
        .parse::<u8>()
        .map_err(|_| XMLError::UnexpectedValue(format!("invalid volume: {}", volume)))
}

pub(crate) fn parse_playback_status_xml(xml: String) -> Result<PlaybackStatus, XMLError> {
    let xml = clean_response_xml(xml);

    let parsed_xml = roxmltree::Document::parse(&xml)?;

    let playback_state = get_text(get_tag_by_name(&parsed_xml, "CurrentTransportState")?)?;
    let playback_state = PlaybackState::new(&playback_state).map_err(|_| {
        XMLError::UnexpectedValue(format!("invalid playback state: {}", playback_state))
    })?;

    let status = get_text(get_tag_by_name(&parsed_xml, "CurrentTransportStatus")?)?;

    Ok(PlaybackStatus {
        playback_state,
        status,
    })
}

pub(crate) fn get_error_code(xml: String) -> Result<String, XMLError> {
    let xml = clean_response_xml(xml);

    let parsed_xml = roxmltree::Document::parse(&xml)?;

    get_text(get_tag_by_name(&parsed_xml, "errorCode")?)
}

pub(crate) fn parse_description_xml(
    xml: String,
    ip_addr: Ipv4Addr,
) -> Result<BasicSpeakerInfo, XMLError> {
    let parsed_xml = roxmltree::Document::parse(&xml)?;

    let friendly_name = get_text(get_tag_by_name(&parsed_xml, "friendlyName")?)?;

    let room_name = get_text(get_tag_by_name(&parsed_xml, "roomName")?)?;

    let uuid = get_text(get_tag_by_name(&parsed_xml, "UDN")?)?.replace("uuid:", "");

    Ok(BasicSpeakerInfo {
        friendly_name,
        room_name,
        uuid,
        ip_addr,
    })
}

pub(crate) fn generate_xml(
    action_name: &str,
    service: &Service,
    arguments: HashMap<&str, &str>,
) -> Result<Vec<u8>, XMLError> {
    let mut xml = XMLBuilder::new()
        .version(XMLVersion::XML1_1)
        .encoding("UTF-8".into())
        .build();

    let mut envelope = XMLElement::new("s:Envelope");
    envelope.add_attribute("xmlns:s", "http://schemas.xmlsoap.org/soap/envelope/");
    envelope.add_attribute(
        "s:encodingStyle",
        "http://schemas.xmlsoap.org/soap/encoding/",
    );

    let mut body = XMLElement::new("s:Body");

    let mut action = XMLElement::new(&format!("u:{}", action_name));
    action.add_attribute(
        "xmlns:u",
        &format!("urn:schemas-upnp-org:service:{}", service.get_name()),
    );

    for (arg, value) in arguments {
        let mut xml_obj = XMLElement::new(arg);
        xml_obj.add_text(value.to_owned())?;
        action.add_child(xml_obj)?;
    }

    body.add_child(action)?;

    envelope.add_child(body)?;

    xml.set_root_element(envelope);

    let mut writer = Vec::new();
    xml.generate(&mut writer)?;
    Ok(writer)
}
