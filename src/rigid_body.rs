use byteorder::{ReadBytesExt, LittleEndian};
use marker::Marker;
use nalgebra::Quaternion;
use semver::Version;
use std::io::BufRead;
use super::{Result, Unpack};

/// A set of `Marker`s creating a rigid body
#[derive(Clone, Debug, PartialEq)]
pub struct RigidBody {
    /// ID of body
    pub id: i32,
    /// Position in 3D
    pub position: Marker,
    /// Orientation represented as a quaternion
    pub orientation: Quaternion<f32>,
    /// List of markers comprising this body
    pub markers: Vec<Marker>,
    /// List of marker IDs
    pub marker_ids: Vec<i32>,
    /// List of marker sizes
    pub marker_sizes: Vec<f32>,
    /// Mean error for markers
    pub mean_error: f32,
    /// Was the body successfully tracked this frame (NatNet >= 2.6)
    pub valid_track: Option<bool>,
}

impl Unpack<RigidBody> for RigidBody {
    fn unpack<B: BufRead>(ver: &Version, bytes: &mut B) -> Result<RigidBody> {
        // Unpack Rigid body according to `PacketClient.cpp` lines 667:738
        let id = try!(bytes.read_i32::<LittleEndian>());
        let pos = try!(Marker::unpack(ver, bytes));
        let orient = try!(Quaternion::unpack(ver, bytes));
        let num_markers = try!(bytes.read_i32::<LittleEndian>());
        let mut markers = Vec::with_capacity(num_markers as usize);
        let mut ids = Vec::with_capacity(num_markers as usize);
        let mut sizes = Vec::with_capacity(num_markers as usize);
        // NOTE: All markers are consecutively, then IDs, then sizes
        // See: lines 684:710
        // FIXME: Should data be presented differently to users?
        for _ in 0..num_markers {
            markers.push(try!(Marker::unpack(ver, bytes)));
        }
        for _ in 0..num_markers {
            ids.push(try!(bytes.read_i32::<LittleEndian>()));
        }
        for _ in 0..num_markers {
            sizes.push(try!(bytes.read_f32::<LittleEndian>()));
        }
        let err = try!(bytes.read_f32::<LittleEndian>());
        let track = if *ver >= Version::parse("2.6.0").unwrap() {
            let params = try!(bytes.read_i16::<LittleEndian>());
            Some(params & 0x01 > 0)
        } else {
            None
        };
        Ok(RigidBody {
            id: id,
            position: pos,
            orientation: orient,
            markers: markers,
            marker_ids: ids,
            marker_sizes: sizes,
            mean_error: err,
            valid_track: track,
        })
    }
}

impl Unpack<Quaternion<f32>> for Quaternion<f32> {
    fn unpack<B: BufRead>(_: &Version, bytes: &mut B) -> Result<Quaternion<f32>> {
        let x = try!(bytes.read_f32::<LittleEndian>());
        let y = try!(bytes.read_f32::<LittleEndian>());
        let z = try!(bytes.read_f32::<LittleEndian>());
        let w = try!(bytes.read_f32::<LittleEndian>());
        Ok(Quaternion::new(x, y, z, w))
    }
}
