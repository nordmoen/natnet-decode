use byteorder::{ReadBytesExt, LittleEndian};
use semver::Version;
use std::io::BufRead;
use super::{Result, Unpack};

/// Force plate
///
/// # `NatNet` version
/// This structure is new in 2.9
#[derive(Clone, Debug, PartialEq)]
pub struct ForcePlate {
    /// ID of plate
    pub id: i32,
    /// Channels from plate
    pub channels: Vec<Vec<f32>>,
}

impl Unpack<ForcePlate> for ForcePlate {
    fn unpack<B: BufRead>(_: &Version, bytes: &mut B) -> Result<ForcePlate> {
        let id = try!(bytes.read_i32::<LittleEndian>());
        let num_channels = try!(bytes.read_i32::<LittleEndian>());
        let mut chans = Vec::with_capacity(num_channels as usize);
        for _ in 0..num_channels {
            let num_frames = try!(bytes.read_i32::<LittleEndian>());
            let mut frame = Vec::with_capacity(num_frames as usize);
            for _ in 0..num_frames {
                frame.push(try!(bytes.read_f32::<LittleEndian>()));
            }
            chans.push(frame);
        }
        Ok(ForcePlate {
            id: id,
            channels: chans,
        })
    }
}
