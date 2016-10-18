use byteorder::{ReadBytesExt, LittleEndian};
use nalgebra::Point3;
use semver::Version;
use std::io::BufRead;
use super::{Result, Unpack};

/// Visible marker as a point
pub type Marker = Point3<f32>;

/// Identifiable `Marker`
#[derive(Clone, Debug, PartialEq)]
pub struct LabeledMarker {
    /// ID of this marker
    pub id: i32,
    /// Position in 3D space
    pub position: Marker,
    /// Size of marker
    pub size: f32,
    /// Was marker occluded in this frame (NatNet >= 2.6)
    pub occluded: Option<bool>,
    /// Was the position solved using point cloud? (NatNet >= 2.6)
    pub point_cloud_solved: Option<bool>,
    /// Was the position solved using a model solver? (NatNet >= 2.6)
    pub model_solved: Option<bool>,
}

impl Unpack<Marker> for Marker {
    fn unpack<B: BufRead>(_: &Version, bytes: &mut B) -> Result<Marker> {
        // From `PacketClient.cpp` line 643:645
        let x = try!(bytes.read_f32::<LittleEndian>());
        let y = try!(bytes.read_f32::<LittleEndian>());
        let z = try!(bytes.read_f32::<LittleEndian>());
        Ok(Marker::new(x, y, z))
    }
}

impl Unpack<LabeledMarker> for LabeledMarker {
    fn unpack<B: BufRead>(ver: &Version, bytes: &mut B) -> Result<LabeledMarker> {
        // From `PacketClient.cpp` line 825:857
        let id = try!(bytes.read_i32::<LittleEndian>());
        let pos = try!(Marker::unpack(ver, bytes));
        let size = try!(bytes.read_f32::<LittleEndian>());
        let (oc, pcs, ms) = if *ver >= Version::parse("2.6.0").unwrap() {
            let params = try!(bytes.read_i16::<LittleEndian>());
            (Some(params & 0x01 > 0), Some(params & 0x02 > 0), Some(params & 0x04 > 0))
        } else {
            (None, None, None)
        };
        Ok(LabeledMarker {
            id: id,
            position: pos,
            size: size,
            occluded: oc,
            point_cloud_solved: pcs,
            model_solved: ms,
        })
    }
}
