use std::{collections::HashMap, net::Ipv4Addr};

use xml_builder::{XMLBuilder, XMLElement, XMLError, XMLVersion};

use crate::services::Service;

pub(crate) fn build_sonos_url(ip: Ipv4Addr, endpoint: &str) -> String {
    format!("http://{ip}:1400{endpoint}")
}

pub(crate) async fn get_res_text(res: reqwest::Response) -> Result<String, String> {
    Ok(res
        .text()
        .await
        .map_err(|err| format!("Error getting response body: {err}"))?)
}

pub(crate) fn handle_sonos_err_code(service: &Service, err_code: &str) -> String {
    match service {
        Service::AVTransport => match err_code {
            "701" => String::from("Transition unavailable"),
            "711" => String::from("Invalid seek target"),
            _ => format!("Sonos error code: {err_code}"),
        },
        _ => format!("Sonos error code: {err_code}"),
    }
}

// not checking for str validity since this function is internal to crate
pub(crate) fn generate_xml(
    action: &str,
    service: &str,
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

    let mut action = XMLElement::new(&format!("u:{}", action));
    action.add_attribute(
        "xmlns:u",
        &format!("urn:schemas-upnp-org:service:{}", service),
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

pub(crate) fn stringify_xml_err(err: XMLError) -> String {
    match err {
        XMLError::IOError(s) => s,
        XMLError::InsertError(s) => s,
    }
}
