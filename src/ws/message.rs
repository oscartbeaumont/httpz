//! The implementation in this file have been taken from the tungstenite crate. Thanks to it's original authors!
//! The tungstenite crate doesn't support Cloudflare Workers so we have to have a local version.

use std::{fmt, str};

use crate::Error;

/// An enum representing the various forms of a WebSocket message.
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Message {
    /// A text WebSocket message
    Text(String),
    /// A binary WebSocket message
    Binary(Vec<u8>),
    /// A ping message with the specified payload
    ///
    /// The payload here must have a length less than 125 bytes
    Ping(Vec<u8>),
    /// A pong message with the specified payload
    ///
    /// The payload here must have a length less than 125 bytes
    Pong(Vec<u8>),
    /// A close message with the optional close frame.
    #[cfg(feature = "tokio-ws")]
    Close(Option<async_tungstenite::tungstenite::protocol::CloseFrame<'static>>),
    /// Raw frame. Note, that you're not going to get this value while reading the message.
    #[cfg(feature = "tokio-ws")]
    Frame(async_tungstenite::tungstenite::protocol::frame::Frame),
}

impl Message {
    /// Create a new text WebSocket message from a stringable.
    pub fn text<S>(string: S) -> Message
    where
        S: Into<String>,
    {
        Message::Text(string.into())
    }

    /// Create a new binary WebSocket message by converting to Vec<u8>.
    pub fn binary<B>(bin: B) -> Message
    where
        B: Into<Vec<u8>>,
    {
        Message::Binary(bin.into())
    }

    /// Indicates whether a message is a text message.
    pub fn is_text(&self) -> bool {
        matches!(*self, Message::Text(_))
    }

    /// Indicates whether a message is a binary message.
    pub fn is_binary(&self) -> bool {
        matches!(*self, Message::Binary(_))
    }

    /// Indicates whether a message is a ping message.
    pub fn is_ping(&self) -> bool {
        matches!(*self, Message::Ping(_))
    }

    /// Indicates whether a message is a pong message.
    pub fn is_pong(&self) -> bool {
        matches!(*self, Message::Pong(_))
    }

    /// Indicates whether a message ia s close message.
    #[cfg(feature = "tokio-ws")]
    pub fn is_close(&self) -> bool {
        matches!(*self, Message::Close(_))
    }

    /// Get the length of the WebSocket message.
    pub fn len(&self) -> usize {
        match *self {
            Message::Text(ref string) => string.len(),
            Message::Binary(ref data) | Message::Ping(ref data) | Message::Pong(ref data) => {
                data.len()
            }
            #[cfg(feature = "tokio-ws")]
            Message::Close(ref data) => data.as_ref().map(|d| d.reason.len()).unwrap_or(0),
            #[cfg(feature = "tokio-ws")]
            Message::Frame(ref frame) => frame.len(),
        }
    }

    /// Returns true if the WebSocket message has no content.
    /// For example, if the other side of the connection sent an empty string.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Consume the WebSocket and return it as binary data.
    pub fn into_data(self) -> Vec<u8> {
        match self {
            Message::Text(string) => string.into_bytes(),
            Message::Binary(data) | Message::Ping(data) | Message::Pong(data) => data,
            #[cfg(feature = "tokio-ws")]
            Message::Close(None) => Vec::new(),
            #[cfg(feature = "tokio-ws")]
            Message::Close(Some(frame)) => frame.reason.into_owned().into_bytes(),
            #[cfg(feature = "tokio-ws")]
            Message::Frame(frame) => frame.into_data(),
        }
    }

    /// Attempt to consume the WebSocket message and convert it to a String.
    pub fn into_text(self) -> Result<String, Error> {
        match self {
            Message::Text(string) => Ok(string),
            Message::Binary(data) | Message::Ping(data) | Message::Pong(data) => {
                Ok(String::from_utf8(data).map_err(|_| Error::Utf8)?)
            }
            #[cfg(feature = "tokio-ws")]
            Message::Close(None) => Ok(String::new()),
            #[cfg(feature = "tokio-ws")]
            Message::Close(Some(frame)) => Ok(frame.reason.into_owned()),
            #[cfg(feature = "tokio-ws")]
            Message::Frame(frame) => Ok(frame.into_string().map_err(|_| Error::Utf8)?),
        }
    }

    /// Attempt to get a &str from the WebSocket message,
    /// this will try to convert binary data to utf8.
    pub fn to_text(&self) -> Result<&str, Error> {
        match *self {
            Message::Text(ref string) => Ok(string),
            Message::Binary(ref data) | Message::Ping(ref data) | Message::Pong(ref data) => {
                Ok(str::from_utf8(data).map_err(|_| Error::Utf8)?)
            }
            #[cfg(feature = "tokio-ws")]
            Message::Close(None) => Ok(""),
            #[cfg(feature = "tokio-ws")]
            Message::Close(Some(ref frame)) => Ok(&frame.reason),
            #[cfg(feature = "tokio-ws")]
            Message::Frame(ref frame) => Ok(frame.to_text().map_err(|_| Error::Utf8)?),
        }
    }
}

impl From<String> for Message {
    fn from(string: String) -> Self {
        Message::text(string)
    }
}

impl<'s> From<&'s str> for Message {
    fn from(string: &'s str) -> Self {
        Message::text(string)
    }
}

impl<'b> From<&'b [u8]> for Message {
    fn from(data: &'b [u8]) -> Self {
        Message::binary(data)
    }
}

impl From<Vec<u8>> for Message {
    fn from(data: Vec<u8>) -> Self {
        Message::binary(data)
    }
}

impl From<Message> for Vec<u8> {
    fn from(message: Message) -> Self {
        message.into_data()
    }
}

impl TryFrom<Message> for String {
    type Error = Error;

    fn try_from(value: Message) -> Result<Self, Error> {
        value.into_text()
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if let Ok(string) = self.to_text() {
            write!(f, "{}", string)
        } else {
            write!(f, "Binary Data<length={}>", self.len())
        }
    }
}

#[cfg(feature = "tokio-ws")]
impl From<async_tungstenite::tungstenite::Message> for Message {
    fn from(msg: async_tungstenite::tungstenite::Message) -> Self {
        match msg {
            async_tungstenite::tungstenite::Message::Text(string) => Message::Text(string),
            async_tungstenite::tungstenite::Message::Binary(data) => Message::Binary(data),
            async_tungstenite::tungstenite::Message::Ping(data) => Message::Ping(data),
            async_tungstenite::tungstenite::Message::Pong(data) => Message::Pong(data),
            #[cfg(feature = "tokio-ws")]
            async_tungstenite::tungstenite::Message::Close(frame) => Message::Close(frame),
            #[cfg(not(feature = "tokio-ws"))]
            async_tungstenite::tungstenite::Message::Close(frame) => unreachable!(),
            #[cfg(feature = "tokio-ws")]
            async_tungstenite::tungstenite::Message::Frame(frame) => Message::Frame(frame),
            #[cfg(not(feature = "tokio-ws"))]
            async_tungstenite::tungstenite::Message::Frame(frame) => unreachable!(),
        }
    }
}

#[cfg(feature = "tokio-ws")]
impl From<Message> for async_tungstenite::tungstenite::Message {
    fn from(v: Message) -> Self {
        match v {
            Message::Text(string) => Self::Text(string),
            Message::Binary(data) => Self::Binary(data),
            Message::Ping(data) => Self::Ping(data),
            Message::Pong(data) => Self::Pong(data),
            #[cfg(feature = "tokio-ws")]
            Message::Close(frame) => Self::Close(frame),
            #[cfg(feature = "tokio-ws")]
            Message::Frame(frame) => Self::Frame(frame),
        }
    }
}
