use byteorder::{ReadBytesExt, LittleEndian};
use force_plate::ForcePlate;
use marker::{Marker, LabeledMarker};
use rigid_body::RigidBody;
use semver::Version;
use skeleton::Skeleton;
use std::collections::BTreeMap;
use std::io::BufRead;
use super::{Result, Unpack, ParseError, read_cstring};

/// Frame of Data
///
/// This struct represents the main data coming from Motive
#[derive(Clone, Debug, PartialEq)]
pub struct FrameOfData {
    /// Current frame number
    pub frame_number: i32,
    /// Named marker sets
    pub marker_sets: BTreeMap<String, Vec<Marker>>,
    /// List of unnamed markers
    pub other_markers: Vec<Marker>,
    /// List of rigid bodies
    pub rigid_bodies: Vec<RigidBody>,
    /// List of skeletons
    pub skeletons: Vec<Skeleton>,
    /// List of labeled markers
    pub labeled_markers: Vec<LabeledMarker>,
    /// List of Force plate data (NatNet >= 2.9)
    pub force_plates: Option<Vec<ForcePlate>>,
    pub latency: f32,
    pub timecode: (u32, u32),
    /// Time stamp of data (NatNet >= 2.6)
    pub timestamp: Option<f64>,
    /// Is Motive recording data? (NatNet >= 2.6)
    pub is_recording: Option<bool>,
    /// Has the list of actively tracked models changed? (NatNet >= 2.6)
    pub tracked_models_changed: Option<bool>,
}

fn unpack_vec<R, T: Unpack<R>, B: BufRead>(ver: &Version, bytes: &mut B) -> Result<Vec<R>> {
    let num = try!(bytes.read_i32::<LittleEndian>());
    trace!("Unpacking vector of length {}", num);
    let mut result = Vec::with_capacity(num as usize);
    for _ in 0..num {
        result.push(try!(T::unpack(ver, bytes)));
    }
    Ok(result)
}

impl Unpack<FrameOfData> for FrameOfData {
    fn unpack<B: BufRead>(ver: &Version, bytes: &mut B) -> Result<FrameOfData> {
        debug!("Unpacking frame of data");
        // Unpack Frame of Data, ref: line 618
        let frame_num = try!(bytes.read_i32::<LittleEndian>());
        trace!("Frame number: {}", frame_num);
        // Read marker sets, line 625:648
        let num_marker_sets = try!(bytes.read_i32::<LittleEndian>());
        trace!("Number of marker sets: {}", num_marker_sets);
        let mut sets = BTreeMap::new();
        for _ in 0..num_marker_sets {
            let name = try!(read_cstring(bytes));
            let num_markers = try!(bytes.read_i32::<LittleEndian>());
            let mut markers = Vec::with_capacity(num_markers as usize);
            for _ in 0..num_markers {
                markers.push(try!(Marker::unpack(ver, bytes)));
            }
            sets.insert(name, markers);
        }
        let others = try!(unpack_vec::<Marker, Marker, _>(ver, bytes));
        let bodies = try!(unpack_vec::<RigidBody, RigidBody, _>(ver, bytes));
        let skels = try!(unpack_vec::<Skeleton, Skeleton, _>(ver, bytes));
        let labeled = try!(unpack_vec::<LabeledMarker, LabeledMarker, _>(ver, bytes));
        // Force plates added in version 2.9
        let plates = if *ver >= Version::parse("2.9.0").unwrap() {
            Some(try!(unpack_vec::<ForcePlate, ForcePlate, _>(ver, bytes)))
        } else {
            None
        };
        let latency = try!(bytes.read_f32::<LittleEndian>());
        trace!("Latency: {}", latency);
        let tc = try!(bytes.read_u32::<LittleEndian>());
        let tcs = try!(bytes.read_u32::<LittleEndian>());
        trace!("Time code: ({}, {})", tc, tcs);
        // Timestamp changed from f32 to f64 in version >= 2.7
        let ts = if *ver >= Version::parse("2.7.0").unwrap() {
            Some(try!(bytes.read_f64::<LittleEndian>()))
        } else if *ver >= Version::parse("2.6.0").unwrap() {
            Some(try!(bytes.read_f32::<LittleEndian>()) as f64)
        } else {
            None
        };
        // In the `PacketClient.cpp` code, line 913 these parameter
        // are simply extracted, however they seem to have been added
        // between 2.5 and 2.6 and so must be checked
        let (is_rec, tmc) = if *ver >= Version::parse("2.6.0").unwrap() {
            let params = try!(bytes.read_i16::<LittleEndian>());
            (Some(params & 0x01 > 0), Some(params & 0x02 > 0))
        } else {
            (None, None)
        };
        // End of data tag, must be `0` for valid message
        let eod = try!(bytes.read_i32::<LittleEndian>());
        if eod == 0 {
            trace!("Parsed complete frame of data");
            Ok(FrameOfData {
                frame_number: frame_num,
                marker_sets: sets,
                other_markers: others,
                rigid_bodies: bodies,
                skeletons: skels,
                labeled_markers: labeled,
                force_plates: plates,
                latency: latency,
                timecode: (tc, tcs),
                timestamp: ts,
                is_recording: is_rec,
                tracked_models_changed: tmc,
            })
        } else {
            debug!("End of data tag, 0 != {}", eod);
            Err(ParseError::UnknownError)
        }
    }
}
