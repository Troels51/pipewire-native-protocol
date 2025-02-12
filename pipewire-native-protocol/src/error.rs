use core_proxy::CoreEvent;
use spa::deserialize::DeserializeError;

use crate::core_proxy;

#[derive(thiserror::Error, Debug)]
pub(crate) enum PipewireConnectionError {
    #[error("Could not send message to proxy")]
    ChannelConnectionError(#[from] tokio::sync::mpsc::error::SendError<CoreEvent>),
    #[error("Message for proxy not preset")]
    ProxyNotPresentError(i32),
    #[error("Could not deserialize message")]
    DeserializeError(DeserializeError<Vec<u8>>),
    #[error("Unknow error, likely a bug in the library")]
    Unknown,
}

impl From<DeserializeError<&[u8]>> for PipewireConnectionError  {
    fn from(value: DeserializeError<&[u8]>) -> Self {
        match value {
            DeserializeError::Nom(e) => PipewireConnectionError::DeserializeError(DeserializeError::Nom(e.to_owned())),
            DeserializeError::UnsupportedType => PipewireConnectionError::DeserializeError(DeserializeError::UnsupportedType),
            DeserializeError::InvalidType => PipewireConnectionError::DeserializeError(DeserializeError::InvalidType),
            DeserializeError::PropertyMissing => PipewireConnectionError::DeserializeError(DeserializeError::PropertyMissing),
            DeserializeError::PropertyWrongKey(a) => PipewireConnectionError::DeserializeError(DeserializeError::PropertyWrongKey(a)),
            DeserializeError::InvalidChoiceType => PipewireConnectionError::DeserializeError(DeserializeError::InvalidChoiceType),
            DeserializeError::MissingChoiceValues => PipewireConnectionError::DeserializeError(DeserializeError::MissingChoiceValues),
        }
    }
}
