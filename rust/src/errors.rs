use std::{error, fmt, io, string};
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};

pub enum Error {
    /// Errors encountered while operating on I/O channels.
    ///
    /// These include *connection closed* and *bind failure*.
    Transport(TransportError),
    /// Errors encountered during runtime-library processing.
    ///
    /// These include *message too large* and *unsupported protocol version*.
    Protocol(ProtocolError),
    /// Errors encountered within auto-generated code, or when incoming
    /// or outgoing messages violate the Thrift spec.
    ///
    /// These include *out-of-order messages* and *missing required struct
    /// fields*.
    ///
    /// This variant also functions as a catch-all: errors from handler
    /// functions are automatically returned as an `ApplicationError`.
    Application(ApplicationError),
}

impl error::Error for Error {}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Error::Transport(ref e) => Debug::fmt(e, f),
            Error::Protocol(ref e) => Debug::fmt(e, f),
            Error::Application(ref e) => Debug::fmt(e, f),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Error::Transport(ref e) => Display::fmt(e, f),
            Error::Protocol(ref e) => Display::fmt(e, f),
            Error::Application(ref e) => Display::fmt(e, f),
        }
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Application(ApplicationError {
            kind: ApplicationErrorKind::Unknown,
            message: s,
        })
    }
}

impl<'a> From<&'a str> for Error {
    fn from(s: &'a str) -> Self {
        Error::Application(ApplicationError {
            kind: ApplicationErrorKind::Unknown,
            message: String::from(s),
        })
    }
}

impl From<TransportError> for Error {
    fn from(e: TransportError) -> Self {
        Error::Transport(e)
    }
}

impl From<ProtocolError> for Error {
    fn from(e: ProtocolError) -> Self {
        Error::Protocol(e)
    }
}

impl From<ApplicationError> for Error {
    fn from(e: ApplicationError) -> Self {
        Error::Application(e)
    }
}

/// Create a new `Error` instance of type `Transport` that wraps a
/// `TransportError`.
pub fn new_transport_error<S: Into<String>>(kind: TransportErrorKind, message: S) -> Error {
    Error::Transport(TransportError::new(kind, message))
}

/// Information about I/O errors.
#[derive(Debug, Eq, PartialEq)]
pub struct TransportError {
    /// I/O error variant.
    ///
    /// If a specific `TransportErrorKind` does not apply use
    /// `TransportErrorKind::Unknown`.
    pub kind: TransportErrorKind,
    /// Human-readable error message.
    pub message: String,
}

impl TransportError {
    /// Create a new `TransportError`.
    pub fn new<S: Into<String>>(kind: TransportErrorKind, message: S) -> TransportError {
        TransportError {
            kind: kind,
            message: message.into(),
        }
    }
}

/// I/O error categories.
///
/// This list may grow, and it is not recommended to match against it.
#[derive(Clone, Copy, Eq, Debug, PartialEq)]
pub enum TransportErrorKind {
    /// Catch-all I/O error.
    Unknown = 0,
    /// An I/O operation was attempted when the transport channel was not open.
    NotOpen = 1,
    /// The transport channel cannot be opened because it was opened previously.
    AlreadyOpen = 2,
    /// An I/O operation timed out.
    TimedOut = 3,
    /// A read could not complete because no bytes were available.
    EndOfFile = 4,
    /// An invalid (buffer/message) size was requested or received.
    NegativeSize = 5,
    /// Too large a buffer or message size was requested or received.
    SizeLimit = 6,
}

impl Display for TransportError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", match self.kind {
            TransportErrorKind::Unknown => "transport error",
            TransportErrorKind::NotOpen => "not open",
            TransportErrorKind::AlreadyOpen => "already open",
            TransportErrorKind::TimedOut => "timed out",
            TransportErrorKind::EndOfFile => "end of file",
            TransportErrorKind::NegativeSize => "negative size message",
            TransportErrorKind::SizeLimit => "message too long",
        })
    }
}

impl TryFrom<i32> for TransportErrorKind {
    type Error = Error;
    fn try_from(from: i32) -> Result<Self, Self::Error> {
        match from {
            0 => Ok(TransportErrorKind::Unknown),
            1 => Ok(TransportErrorKind::NotOpen),
            2 => Ok(TransportErrorKind::AlreadyOpen),
            3 => Ok(TransportErrorKind::TimedOut),
            4 => Ok(TransportErrorKind::EndOfFile),
            5 => Ok(TransportErrorKind::NegativeSize),
            6 => Ok(TransportErrorKind::SizeLimit),
            _ => Err(Error::Protocol(ProtocolError {
                kind: ProtocolErrorKind::Unknown,
                message: format!("cannot convert {} to TransportErrorKind", from),
            })),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionRefused
            | io::ErrorKind::NotConnected => Error::Transport(TransportError {
                kind: TransportErrorKind::NotOpen,
                message: err.to_string().to_owned(),
            }),
            io::ErrorKind::AlreadyExists => Error::Transport(TransportError {
                kind: TransportErrorKind::AlreadyOpen,
                message: err.to_string().to_owned(),
            }),
            io::ErrorKind::TimedOut => Error::Transport(TransportError {
                kind: TransportErrorKind::TimedOut,
                message: err.to_string().to_owned(),
            }),
            io::ErrorKind::UnexpectedEof => Error::Transport(TransportError {
                kind: TransportErrorKind::EndOfFile,
                message: err.to_string().to_owned(),
            }),
            _ => {
                Error::Transport(TransportError {
                    kind: TransportErrorKind::Unknown,
                    message: err.to_string().to_owned(),
                })
            }
        }
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Self {
        Error::Protocol(ProtocolError {
            kind: ProtocolErrorKind::InvalidData,
            message: err.to_string().to_owned(),
        })
    }
}

/// Create a new `Error` instance of type `Protocol` that wraps a
/// `ProtocolError`.
pub fn new_protocol_error<S: Into<String>>(kind: ProtocolErrorKind, message: S) -> Error {
    Error::Protocol(ProtocolError::new(kind, message))
}

/// Information about errors that occur in the runtime library.
#[derive(Debug, Eq, PartialEq)]
pub struct ProtocolError {
    /// Protocol error variant.
    ///
    /// If a specific `ProtocolErrorKind` does not apply use
    /// `ProtocolErrorKind::Unknown`.
    pub kind: ProtocolErrorKind,
    /// Human-readable error message.
    pub message: String,
}

impl ProtocolError {
    /// Create a new `ProtocolError`.
    pub fn new<S: Into<String>>(kind: ProtocolErrorKind, message: S) -> ProtocolError {
        ProtocolError {
            kind: kind,
            message: message.into(),
        }
    }
}

/// Runtime library error categories.
///
/// This list may grow, and it is not recommended to match against it.
#[derive(Clone, Copy, Eq, Debug, PartialEq)]
pub enum ProtocolErrorKind {
    /// Catch-all runtime-library error.
    Unknown = 0,
    /// An invalid argument was supplied to a library function, or invalid data
    /// was received from a Thrift endpoint.
    InvalidData = 1,
    /// An invalid size was received in an encoded field.
    NegativeSize = 2,
    /// Thrift message or field was too long.
    SizeLimit = 3,
    /// Unsupported or unknown Thrift protocol version.
    BadVersion = 4,
    /// Unsupported Thrift protocol, server or field type.
    NotImplemented = 5,
    /// Reached the maximum nested depth to which an encoded Thrift field could
    /// be skipped.
    DepthLimit = 6,
}

impl Display for ProtocolError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", match self.kind {
            ProtocolErrorKind::Unknown => "protocol error",
            ProtocolErrorKind::InvalidData => "bad data",
            ProtocolErrorKind::NegativeSize => "negative message size",
            ProtocolErrorKind::SizeLimit => "message too long",
            ProtocolErrorKind::BadVersion => "invalid thrift version",
            ProtocolErrorKind::NotImplemented => "not implemented",
            ProtocolErrorKind::DepthLimit => "maximum skip depth reached",
        })
    }
}

impl TryFrom<i32> for ProtocolErrorKind {
    type Error = Error;
    fn try_from(from: i32) -> Result<Self, Self::Error> {
        match from {
            0 => Ok(ProtocolErrorKind::Unknown),
            1 => Ok(ProtocolErrorKind::InvalidData),
            2 => Ok(ProtocolErrorKind::NegativeSize),
            3 => Ok(ProtocolErrorKind::SizeLimit),
            4 => Ok(ProtocolErrorKind::BadVersion),
            5 => Ok(ProtocolErrorKind::NotImplemented),
            6 => Ok(ProtocolErrorKind::DepthLimit),
            _ => Err(Error::Protocol(ProtocolError {
                kind: ProtocolErrorKind::Unknown,
                message: format!("cannot convert {} to ProtocolErrorKind", from),
            })),
        }
    }
}

/// Create a new `Error` instance of type `Application` that wraps an
/// `ApplicationError`.
pub fn new_application_error<S: Into<String>>(kind: ApplicationErrorKind, message: S) -> Error {
    Error::Application(ApplicationError::new(kind, message))
}

/// Information about errors in auto-generated code or in user-implemented
/// service handlers.
#[derive(Debug, Eq, PartialEq)]
pub struct ApplicationError {
    /// Application error variant.
    ///
    /// If a specific `ApplicationErrorKind` does not apply use
    /// `ApplicationErrorKind::Unknown`.
    pub kind: ApplicationErrorKind,
    /// Human-readable error message.
    pub message: String,
}

impl ApplicationError {
    /// Create a new `ApplicationError`.
    pub fn new<S: Into<String>>(kind: ApplicationErrorKind, message: S) -> ApplicationError {
        ApplicationError {
            kind: kind,
            message: message.into(),
        }
    }
}

/// Auto-generated or user-implemented code error categories.
///
/// This list may grow, and it is not recommended to match against it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationErrorKind {
    /// Catch-all application error.
    Unknown = 0,
    /// Made service call to an unknown service method.
    UnknownMethod = 1,
    /// Received an unknown Thrift message type. That is, not one of the
    /// `thrift::protocol::TMessageType` variants.
    InvalidMessageType = 2,
    /// Method name in a service reply does not match the name of the
    /// receiving service method.
    WrongMethodName = 3,
    /// Received an out-of-order Thrift message.
    BadSequenceId = 4,
    /// Service reply is missing required fields.
    MissingResult = 5,
    /// Auto-generated code failed unexpectedly.
    InternalError = 6,
    /// Thrift protocol error. When possible use `Error::ProtocolError` with a
    /// specific `ProtocolErrorKind` instead.
    ProtocolError = 7,
    /// *Unknown*. Included only for compatibility with existing Thrift implementations.
    InvalidTransform = 8,
    // ??
    /// Thrift endpoint requested, or is using, an unsupported encoding.
    InvalidProtocol = 9,
    // ??
    /// Thrift endpoint requested, or is using, an unsupported auto-generated client type.
    UnsupportedClientType = 10, // ??
}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", match self.kind {
            ApplicationErrorKind::Unknown => "service error",
            ApplicationErrorKind::UnknownMethod => "unknown service method",
            ApplicationErrorKind::InvalidMessageType => "wrong message type received",
            ApplicationErrorKind::WrongMethodName => "unknown method reply received",
            ApplicationErrorKind::BadSequenceId => "out of order sequence id",
            ApplicationErrorKind::MissingResult => "missing method result",
            ApplicationErrorKind::InternalError => "remote service threw exception",
            ApplicationErrorKind::ProtocolError => "protocol error",
            ApplicationErrorKind::InvalidTransform => "invalid transform",
            ApplicationErrorKind::InvalidProtocol => "invalid protocol requested",
            ApplicationErrorKind::UnsupportedClientType => "unsupported protocol client",
        })
    }
}

impl TryFrom<i32> for ApplicationErrorKind {
    type Error = Error;
    fn try_from(from: i32) -> Result<Self, Self::Error> {
        match from {
            0 => Ok(ApplicationErrorKind::Unknown),
            1 => Ok(ApplicationErrorKind::UnknownMethod),
            2 => Ok(ApplicationErrorKind::InvalidMessageType),
            3 => Ok(ApplicationErrorKind::WrongMethodName),
            4 => Ok(ApplicationErrorKind::BadSequenceId),
            5 => Ok(ApplicationErrorKind::MissingResult),
            6 => Ok(ApplicationErrorKind::InternalError),
            7 => Ok(ApplicationErrorKind::ProtocolError),
            8 => Ok(ApplicationErrorKind::InvalidTransform),
            9 => Ok(ApplicationErrorKind::InvalidProtocol),
            10 => Ok(ApplicationErrorKind::UnsupportedClientType),
            _ => Err(Error::Application(ApplicationError {
                kind: ApplicationErrorKind::Unknown,
                message: format!("cannot convert {} to ApplicationErrorKind", from),
            })),
        }
    }
}
