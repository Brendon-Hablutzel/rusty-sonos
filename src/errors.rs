//! Crate error types

/// An XML-related error
#[derive(Debug)]
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

impl From<xml_builder::XMLError> for XMLError {
    fn from(error: xml_builder::XMLError) -> Self {
        Self::XMLBuilderError(error)
    }
}

impl From<roxmltree::Error> for XMLError {
    fn from(error: roxmltree::Error) -> Self {
        Self::ParseError(error)
    }
}

impl std::fmt::Display for XMLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(source) => write!(f, "error parsing XML: {}", source),
            Self::ElementNotFound(details) => write!(f, "element not found {}", details),
            Self::UnexpectedValue(details) => write!(f, "unexpected value: {}", details),
            Self::XMLBuilderError(source) => write!(f, "error building XML: {}", source),
        }
    }
}

impl std::error::Error for XMLError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ElementNotFound(_) => None,
            Self::ParseError(source) => Some(source),
            Self::UnexpectedValue(_) => None,
            Self::XMLBuilderError(source) => Some(source),
        }
    }
}

/// Errors that may be returned from speaker methods
#[derive(Debug)]
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

impl From<XMLError> for SpeakerError {
    fn from(error: XMLError) -> Self {
        Self::XMLError(error)
    }
}

impl From<SonosError> for SpeakerError {
    fn from(error: SonosError) -> Self {
        Self::SonosError(error)
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
            Self::RequestError(error)
        } else {
            Self::ResponseError(error)
        }
    }
}

impl std::fmt::Display for SpeakerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(details) => write!(f, "invalid input: {}", details),
            Self::RequestError(source) => write!(f, "request error: {}", source),
            Self::ResponseError(source) => write!(f, "response error: {}", source),
            Self::SonosError(source) => write!(f, "Sonos speaker error: {}", source),
            Self::XMLError(source) => write!(f, "XML error: {}", source),
        }
    }
}

impl std::error::Error for SpeakerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidInput(_) => None,
            Self::RequestError(source) => Some(source),
            Self::ResponseError(source) => Some(source),
            Self::SonosError(source) => Some(source),
            Self::XMLError(source) => Some(source),
        }
    }
}

/// Speaker-specific errors
#[derive(Debug)]
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
            "701" => Self::TransitionUnavailable,
            "711" => Self::InvalidSeekTarget,
            _ => Self::Unknown(String::from(additional_details)),
        }
    }
}

impl std::fmt::Display for SonosError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TransitionUnavailable => write!(f, "transition unavailable"),
            Self::InvalidSeekTarget => write!(f, "invalid seek target"),
            Self::Unknown(s) => write!(f, "other Sonos error: {}", s),
        }
    }
}

impl std::error::Error for SonosError {}
