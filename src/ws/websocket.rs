use futures::{Sink, Stream};

use super::Message;

/// TODO
pub trait Websocket:
    Sink<Message, Error = crate::Error> + Stream<Item = Result<Message, crate::Error>> + Send + Unpin
{
}

impl<T> Websocket for T where
    T: Sink<Message, Error = crate::Error>
        + Stream<Item = Result<Message, crate::Error>>
        + Send
        + Unpin
{
}
