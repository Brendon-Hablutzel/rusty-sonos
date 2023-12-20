use roxmltree;

use crate::responses::{CurrentTrack, PlaybackState, PlaybackStatus, QueueItem};

fn clean_response_string(xml: String) -> String {
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

pub(crate) fn get_tag_by_name<'a>(
    parsed_xml: &'a roxmltree::Document,
    tag_name: &str,
) -> Result<roxmltree::Node<'a, 'a>, String> {
    let tag = parsed_xml
        .descendants()
        .find(|n| n.has_tag_name(tag_name))
        .ok_or(format!("'{tag_name}' tag not found"))?;
    Ok(tag)
}

pub(crate) fn get_text<'a>(tag: roxmltree::Node, err: &'a str) -> Result<String, &'a str> {
    Ok(tag.text().ok_or(err)?.to_owned())
}

pub(crate) fn parse_queue_xml(xml: String) -> Result<Vec<QueueItem>, String> {
    let xml = clean_response_string(xml);

    let parsed =
        roxmltree::Document::parse(&xml).map_err(|err| format!("Error parsing xml: {err}"))?;

    let items: Result<Vec<QueueItem>, String> = parsed
        .descendants()
        .filter(|n| n.has_tag_name("item"))
        .map(|item| parse_queue_item(item))
        .collect();
    let items = items?;
    Ok(items)
}

fn parse_queue_item(item: roxmltree::Node) -> Result<QueueItem, String> {
    let res = item
        .descendants()
        .find(|n| n.has_tag_name("res"))
        .ok_or("'res' tag not found")?;

    let title = item
        .descendants()
        .find(|n| n.has_tag_name("title"))
        .map(|n| get_text(n, "Error getting title"))
        .transpose()?;

    let artist = item
        .descendants()
        .find(|n| n.has_tag_name("albumArtist"))
        .map(|n| get_text(n, "Error getting artist"))
        .transpose()?;

    let duration = res.attribute("duration").map(|n| n.to_owned());

    let uri = get_text(res, "No URI found")?;

    Ok(QueueItem {
        duration,
        uri,
        title,
        artist,
    })
}

pub(crate) fn parse_current_track_xml(xml: String) -> Result<CurrentTrack, String> {
    let xml = clean_response_string(xml);

    let parsed_xml =
        roxmltree::Document::parse(&xml).map_err(|err| format!("Error parsing xml: {err}"))?;

    let duration = get_text(
        get_tag_by_name(&parsed_xml, "TrackDuration")?,
        "No duration found",
    )?;

    let uri = get_text(get_tag_by_name(&parsed_xml, "TrackURI")?, "No track found")?;

    let title = get_tag_by_name(&parsed_xml, "title")
        .ok()
        .map(|n| get_text(n, "Error getting title"))
        .transpose()?;

    let artist = get_tag_by_name(&parsed_xml, "creator")
        .ok()
        .map(|n| get_text(n, "Error getting artist"))
        .transpose()?;

    let position = get_text(
        get_tag_by_name(&parsed_xml, "RelTime")?,
        "No position found",
    )?;

    Ok(CurrentTrack {
        position,
        duration,
        uri,
        title,
        artist,
    })
}

pub(crate) fn parse_getvolume_xml(xml: String) -> Result<u8, String> {
    let xml = clean_response_string(xml);

    let parsed_xml =
        roxmltree::Document::parse(&xml).map_err(|err| format!("Error parsing xml: {err}"))?;

    let volume = get_text(
        get_tag_by_name(&parsed_xml, "CurrentVolume")?,
        "No volume found",
    )?
    .parse::<u8>()
    .map_err(|_| "Unable to parse volume into number".to_owned())?;

    Ok(volume)
}

pub(crate) fn parse_playback_status_xml(xml: String) -> Result<PlaybackStatus, String> {
    let xml = clean_response_string(xml);
    let parsed_xml =
        roxmltree::Document::parse(&xml).map_err(|err| format!("Error parsing xml: {err}"))?;

    let playback_state = get_text(
        get_tag_by_name(&parsed_xml, "CurrentTransportState")?,
        "No state found",
    )?;
    let playback_state = PlaybackState::new(&playback_state)?;

    let status = get_text(
        get_tag_by_name(&parsed_xml, "CurrentTransportStatus")?,
        "No status found",
    )?;

    Ok(PlaybackStatus {
        playback_state,
        status,
    })
}

pub(crate) fn get_error_code(xml: String) -> Result<String, String> {
    let xml = clean_response_string(xml);
    let parsed =
        roxmltree::Document::parse(&xml).map_err(|err| format!("Error parsing xml: {err}"))?;
    let error_code = get_text(
        get_tag_by_name(&parsed, "errorCode")?,
        "Could not find error code",
    )?;
    Ok(error_code)
}
