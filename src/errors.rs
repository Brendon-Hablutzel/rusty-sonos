//! Crate errors

/// An XML-related error
pub enum XMLError {
    /// Parsing error
    ParseError(roxmltree::Error),
    /// Element not found in XML
    ElementNotFound(String),
    /// An unexpected value was found while processing XML
    UnexpectedValue(String), // this string contains the label and value for the unexpected value
    /// An error occurred while building XML
    XMLBuilderError(xml_builder::XMLError),
}

impl std::fmt::Display for XMLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XMLError::ParseError(source) => write!(f, "error parsing XML: {}", source),
            XMLError::ElementNotFound(details) => write!(f, "element not found {}", details),
            XMLError::UnexpectedValue(details) => write!(f, "unexpected value: {}", details),
            XMLError::XMLBuilderError(source) => write!(f, "error building XML: {}", source),
        }
    }
}

impl From<xml_builder::XMLError> for XMLError {
    fn from(error: xml_builder::XMLError) -> Self {
        XMLError::XMLBuilderError(error)
    }
}

impl From<roxmltree::Error> for XMLError {
    fn from(error: roxmltree::Error) -> Self {
        XMLError::ParseError(error)
    }
}

/// Errors that may be returned from speaker methods
pub enum SpeakerError {
    /// An error that occurred while making a request to the speaker
    RequestError(reqwest::Error),
    /// An error that occurred while processing the response from the speaker
    ResponseError(reqwest::Error),
    /// An XML-related error
    XMLError(XMLError),
    /// Bad input for a speaker action, with the string containing additional details
    InvalidInput(String),
    /// A speaker-specific error
    SonosError(SonosError),
}

/// Speaker-specific errors
pub enum SonosError {
    /// Not able to change transition, ex. pausing when playback is already paused
    TransitionUnavailable,
    /// Invalid target for operations such as seek (ex. an invalid duration) or next (using next at the end of the queue, or while not in a queue)
    InvalidSeekTarget,
    /// Some other Sonos error, with the string containing additional data
    Unknown(String),
}

impl SonosError {
    pub(crate) fn from_err_code(err_code: &str, additional_details: &str) -> Self {
        match err_code {
            "701" => SonosError::TransitionUnavailable,
            "711" => SonosError::InvalidSeekTarget,
            _ => SonosError::Unknown(String::from(additional_details)),
        }
    }
}

impl std::fmt::Display for SonosError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SonosError::TransitionUnavailable => write!(f, "transition unavailable"),
            SonosError::InvalidSeekTarget => write!(f, "invalid seek target"),
            SonosError::Unknown(s) => write!(f, "other Sonos error: {}", s),
        }
    }
}

impl From<reqwest::Error> for SpeakerError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_builder()
            || error.is_connect()
            || error.is_redirect()
            || error.is_timeout()
            || error.is_request()
        {
            SpeakerError::RequestError(error)
        } else {
            SpeakerError::ResponseError(error)
        }
    }
}

impl From<XMLError> for SpeakerError {
    fn from(error: XMLError) -> Self {
        SpeakerError::XMLError(error)
    }
}

impl From<SonosError> for SpeakerError {
    fn from(error: SonosError) -> Self {
        SpeakerError::SonosError(error)
    }
}

impl std::fmt::Display for SpeakerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpeakerError::InvalidInput(details) => write!(f, "invalid input: {}", details),
            SpeakerError::RequestError(source) => write!(f, "request error: {}", source),
            SpeakerError::ResponseError(source) => write!(f, "response error: {}", source),
            SpeakerError::SonosError(source) => write!(f, "Sonos speaker error: {}", source),
            SpeakerError::XMLError(source) => write!(f, "XML error: {}", source),
        }
    }
}
