#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

//! Decode `NatNet` messages from raw bytes.
//!
//! [`NatNet`](http://www.optitrack.com/downloads/developer-tools.html#natnet-sdk)
//! is the software solution supporting [OptiTrack](http://www.optitrack.com/)
//! many offerings. Since motion tracking data is multicast it can be decoded
//! in any language and this crate is a Rust (unofficial!) implementation.
//!
//! # Note on `NatNet` versions
//! The raw byte interface in `NatNet` has been through many revisions. Many
//! data fields are optional going backwards. Because of this it is assumed
//! that users have at least version `>=2.5.0`, fields added after this is
//! marked optional so that this crate can support several versions. It is
//! currently up to the user to decide which version to use.
//!
//! # Example
//! ```rust,ignore
//! use natnet_decode::NatNet;
//! use semver::Version;
//!
//! let mut data = Vec::new();
//! // Fill data here
//! // ...
//!
//! // We can then decode this:
//! let parsed = NatNet::unpack_with(&Version::parse("2.5.0").unpack(), &mut data.as_ref());
//! // Do stuff with parsed
//! println!("{:?}", parsed);
//! ```
//!
//! # Acknowledgement
//! This crate is heavily inspired by
//! [python-optirx](https://bitbucket.org/astanin/python-optirx/overview) and
//! test data is borrowed with permission.

extern crate byteorder;
#[macro_use]
extern crate log;
extern crate nalgebra;
extern crate semver;

mod force_plate;
mod frame;
mod marker;
pub mod model;
mod rigid_body;
mod sender;
mod skeleton;
mod messages;

// External imports
use byteorder::{ReadBytesExt, LittleEndian};
use semver::Version;

// Imports from standard library
use std::error::Error as StdError;
use std::fmt;
use std::io::BufRead;
use std::result;

// Local imports
pub use force_plate::ForcePlate;
pub use frame::FrameOfData;
pub use marker::{Marker, LabeledMarker};
pub use messages::{NatNetResponse, NatNetRequest};
pub use rigid_body::RigidBody;
pub use sender::Sender;
pub use skeleton::Skeleton;

/// A result type for errors
pub type Result<T> = result::Result<T, ParseError>;

/// Errors caused during message parsing
#[derive(Debug)]
pub enum ParseError {
    /// Something unexpected happened
    ///
    /// This error is returned when End-Of-Data marker is not
    /// as expected, this most likely mean that all data parsed
    /// is garbage and that there is a version mismatch.
    UnknownError,
    /// Unknown message received from Motive
    ///
    /// The number included is the message ID given by `NatNet`.
    UnknownResponse(u16),
    /// Problem reading bytes from input source
    ///
    /// This error is caused by an IO error on the given input source
    /// the cause of the IO error is returned so that users can
    /// inspect the cause.
    IO(std::io::Error),
    /// Problem converting C-String in Motive
    ///
    /// There was a problem converting the bytes that Motive considers
    /// a string into what Rust considers a String.
    StringError,
    /// There were not enough bytes in the source to parse a complete message
    ///
    /// This is most likely caused by a mismatch in versions.
    NotEnoughBytes,
}

/// C-like Enum representing the different possible messages coming from `NatNet`
/// Updated for `2.10.0`
#[derive(Clone, PartialEq, Debug, PartialOrd)]
pub enum NatNetMsgType {
    Ping = 0,
    PingResponse = 1,
    Request = 2,
    Response = 3,
    RequestModelDef = 4,
    ModelDef = 5,
    RequestFrameOfData = 6,
    FrameOfData = 7,
    MessageString = 8,
    UnrecognizedRequest = 100,
}

/// Parser for `NatNet` data
///
/// This is the main entry point to unpack/parse `NatNet` data.
#[derive(Clone, Debug)]
pub struct NatNet {
    ver: Version,
}

impl NatNet {
    /// Create a new `NatNet` parser with the given version
    ///
    /// This will create a new parser that utilizes the given version
    /// for subsequent `unpack` calls
    pub fn new<V: Into<Version>>(ver: V) -> NatNet {
        NatNet { ver: ver.into() }
    }

    /// Unpack a message from `NatNet` using a specified version
    ///
    /// This will try to unpack a message coming from a NatNet application
    /// assuming the message uses the given version
    pub fn unpack_with<B: BufRead>(ver: &Version, bytes: &mut B) -> Result<NatNetResponse> {
        // First 4 bytes contains `msg_id` and number of bytes in message
        // according to `PacketClient.cpp` line 609:615
        let msg_id = try!(bytes.read_u16::<LittleEndian>());
        let num_bytes = try!(bytes.read_u16::<LittleEndian>());
        NatNet::unpack_rest(msg_id, num_bytes, ver, bytes)
    }

    fn unpack_rest<B: BufRead>(msg_id: u16,
                               num_bytes: u16,
                               ver: &Version,
                               bytes: &mut B)
                               -> Result<NatNetResponse> {
        debug!("Unpacking `NatNet` message with type: {}, size: {}",
               msg_id,
               num_bytes);
        match msg_id {
            _ if msg_id == NatNetMsgType::FrameOfData as u16 => {
                Ok(NatNetResponse::FrameOfData(try!(FrameOfData::unpack(ver, bytes))))
            }
            _ if msg_id == NatNetMsgType::ModelDef as u16 => {
                let num_models = try!(bytes.read_i32::<LittleEndian>());
                let mut models = Vec::with_capacity(num_models as usize);
                for _ in 0..num_models {
                    models.push(try!(model::DataSet::unpack(ver, bytes)));
                }
                Ok(NatNetResponse::ModelDef(models))
            }
            _ if msg_id == NatNetMsgType::PingResponse as u16 => {
                Ok(NatNetResponse::Ping(try!(Sender::unpack(ver, bytes))))
            }
            _ if msg_id == NatNetMsgType::MessageString as u16 => {
                Ok(NatNetResponse::MessageString(try!(read_cstring(bytes))))
            }
            // If the message is a command response it can either be a
            // i32 response code or a response string, line: 147
            _ if msg_id == NatNetMsgType::Response as u16 && num_bytes == 4 => {
                Ok(NatNetResponse::Response(try!(bytes.read_i32::<LittleEndian>())))
            }
            _ if msg_id == NatNetMsgType::Response as u16 => {
                Ok(NatNetResponse::ResponseString(try!(read_cstring(bytes))))
            }
            _ if msg_id == NatNetMsgType::UnrecognizedRequest as u16 => {
                Ok(NatNetResponse::UnrecognizedRequest)
            }
            _ => Err(ParseError::UnknownResponse(msg_id)),
        }
    }

    /// Unpack only `NatNetMsgType` messages
    ///
    /// This method unpacks only messages of the requested type. The function
    /// will consume the header of any `NatNet` message to check if it is the
    /// correct message and unpack only if it is. This method can be useful when
    /// needing to unpack only sender messages if `NatNet` version is unknown.
    pub fn unpack_type_with<B: BufRead>(t: NatNetMsgType,
                                        ver: &Version,
                                        bytes: &mut B)
                                        -> Option<Result<NatNetResponse>> {
        let msg_id = bytes.read_u16::<LittleEndian>();
        let num_bytes = bytes.read_u16::<LittleEndian>();
        trace!("Trying to unpack {:?}", t);
        if let Ok(msg_id) = msg_id {
            if let Ok(num_bytes) = num_bytes {
                if msg_id == t as u16 {
                    trace!("Correct message found");
                    return Some(NatNet::unpack_rest(msg_id, num_bytes, ver, bytes));
                }
            }
        }
        None
    }

    /// Unpack a message from `NatNet`
    pub fn unpack<B: BufRead>(&self, bytes: &mut B) -> Result<NatNetResponse> {
        NatNet::unpack_with(&self.ver, bytes)
    }

    /// Unpack only `NatNetMsgType` messages
    ///
    /// This method unpacks only messages of the requested type. The function
    /// will consume the header of any `NatNet` message to check if it is the
    /// correct message and unpack only if it is. This method can be useful when
    /// needing to unpack only sender messages if `NatNet` version is unknown.
    pub fn unpack_type<B: BufRead>(&self,
                                   t: NatNetMsgType,
                                   bytes: &mut B)
                                   -> Option<Result<NatNetResponse>> {
        NatNet::unpack_type_with(t, &self.ver, bytes)
    }
}

// Private trait used to unpack underlying data
trait Unpack<T> {
    /// Unpack the type `T` from the `BufRead` source
    fn unpack<B: BufRead>(ver: &Version, bytes: &mut B) -> Result<T>;
}

// From io error for ParseError
impl From<std::io::Error> for ParseError {
    /// Convert an IO error into a `ParseError`
    fn from(err: std::io::Error) -> ParseError {
        match err.kind() {
            std::io::ErrorKind::UnexpectedEof => ParseError::NotEnoughBytes,
            _ => ParseError::IO(err),
        }
    }
}

impl From<std::ffi::NulError> for ParseError {
    /// Convert a `std::ffi::NulError` into a `ParseError::StringError`
    fn from(_: std::ffi::NulError) -> ParseError {
        // FIXME: `StringError` should contain the cause
        ParseError::StringError
    }
}

impl fmt::Display for ParseError {
    /// Format `ParseError` in human readable fashion
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::UnknownError => {
                write!(f,
                       "An unknown error occurred (most likely caused by version mismatch)")
            }
            ParseError::UnknownResponse(ref resp) => {
                write!(f, "Got an unknown message from NatNet with ID: {}", resp)
            }
            ParseError::IO(ref err) => write!(f, "IO error: {}", err),
            ParseError::StringError => write!(f, "Error parsing C-String from NatNet"),
            ParseError::NotEnoughBytes => {
                write!(f, "Not enough bytes in source to parse complete message")
            }
        }
    }
}

impl StdError for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::UnknownError => "Unknown error occurred",
            ParseError::UnknownResponse(_) => "Unknown message ID",
            ParseError::IO(ref err) => err.description(),
            ParseError::StringError => "Problem parsing C-String from NatNet",
            ParseError::NotEnoughBytes => "Not enough bytes in source",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            ParseError::IO(ref err) => Some(err),
            _ => None,
        }
    }
}

/// Helper function to read a C-String from raw bytes
fn read_cstring<B: BufRead>(bytes: &mut B) -> Result<String> {
    // Maximum size for a String is 256, ref: line 631
    let mut str_buf = Vec::with_capacity(256);
    try!(bytes.read_until(b'\0', &mut str_buf));
    // Remove null byte from end of `str_buf`
    str_buf.pop();
    match try!(std::ffi::CString::new(str_buf)).into_string() {
        Ok(s) => Ok(s),
        Err(err) => {
            let reason = err.utf8_error();
            error!("Could not convert C-String '{:?}' into String, reason: {:?}",
                   err.into_cstring(),
                   reason);
            // FIXME: Return more descriptive error here
            Err(ParseError::StringError)
        }
    }
}
