use byteorder::{WriteBytesExt, LittleEndian};
use frame::FrameOfData;
use model;
use sender::Sender;
use std::ffi::CString;
use super::NatNetMsgType;

/// Enumeration of possible responses from `NatNet`
#[derive(Clone, Debug, PartialEq)]
pub enum NatNetResponse {
    /// Response to ping request
    ///
    /// The ping response contains data about the sender application
    Ping(Sender),
    /// Response to command
    Response(i32),
    /// Response to command in String form
    ResponseString(String),
    /// Model definitions
    ///
    /// This type contains a list of `DataSet`s that describe data in a
    /// `FrameOfData`
    ModelDef(Vec<model::DataSet>),
    /// Data about tracked content
    FrameOfData(FrameOfData),
    /// Message from the sender application
    MessageString(String),
    /// The sender application did not understand the request
    UnrecognizedRequest,
}

/// Enumeration of possible requests sent to `NatNet`
#[derive(Clone, Debug, PartialEq)]
pub enum NatNetRequest {
    /// Send ping to other application
    ///
    /// This should result in a `NatNetResponse::Ping`
    Ping(CString),
    /// Request model definitions
    ModelDefinitions,
    /// Request a frame of data
    FrameOfData,
}

impl Into<Vec<u8>> for NatNetRequest {
    fn into(self) -> Vec<u8> {
        // Pre-allocate some bytes for the message
        // most messages are smaller than this
        let mut bytes = Vec::with_capacity(32);
        match self {
            NatNetRequest::ModelDefinitions => {
                bytes.write_u16::<LittleEndian>(NatNetMsgType::RequestModelDef as u16).unwrap();
                bytes.write_u16::<LittleEndian>(0).unwrap();
            }
            NatNetRequest::FrameOfData => {
                bytes.write_u16::<LittleEndian>(NatNetMsgType::RequestFrameOfData as u16).unwrap();
                bytes.write_u16::<LittleEndian>(0).unwrap();
            }
            NatNetRequest::Ping(data) => {
                let str_data = data.to_bytes_with_nul();
                bytes.write_u16::<LittleEndian>(NatNetMsgType::Ping as u16).unwrap();
                // NatNet does not support more than 100_000 bytes in messages,
                // to support this restriction in an Into we simply truncate
                // FIXME: Use `TryInto` instead
                if str_data.len() > u16::max_value() as usize {
                    bytes.write_u16::<LittleEndian>(u16::max_value()).unwrap();
                    // The message might still be valid so we append as much as
                    // possible, NOTE: We need to append C-String null to the
                    // end and so we must take `max_value() - 1`
                    bytes.extend_from_slice(&str_data[..u16::max_value() as usize - 1]);
                    bytes.push(b'\0');
                } else {
                    bytes.write_u16::<LittleEndian>(str_data.len() as u16).unwrap();
                    bytes.extend_from_slice(str_data);
                }
            }
        }
        bytes
    }
}
