pub enum Service {
    AVTransport,
    ContentDirectory,
    RenderingControl,
}

impl Service {
    pub fn get_name(&self) -> &'static str {
        match self {
            Service::AVTransport => "AVTransport:1",
            Service::ContentDirectory => "ContentDirectory:1",
            Service::RenderingControl => "RenderingControl:1",
        }
    }

    pub fn get_endpoint(&self) -> &'static str {
        match self {
            Service::AVTransport => "/MediaRenderer/AVTransport/Control",
            Service::ContentDirectory => "/MediaServer/ContentDirectory/Control",
            Service::RenderingControl => "/MediaRenderer/RenderingControl/Control",
        }
    }
}
