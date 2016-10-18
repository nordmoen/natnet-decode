use byteorder::{ReadBytesExt, LittleEndian};
use nalgebra::Vector3;
use semver::Version;
use std::io::BufRead;
use super::{Result, Unpack, read_cstring};

/// Description of `MarkerSet`
#[derive(Clone, Debug, PartialEq)]
pub struct MarkerSet {
    /// Name of set
    pub name: String,
    /// Description of markers in set
    pub markers: Vec<String>,
}

/// Description of `RigidBody`
#[derive(Clone, Debug, PartialEq)]
pub struct RigidBody {
    /// Name of body
    pub name: String,
    /// ID of body
    pub id: i32,
    /// Parent ID of this body
    pub parent_id: i32,
    /// Offset from parent
    pub offset: Vector3<f32>,
}

/// Description of `Skeleton`
#[derive(Clone, Debug, PartialEq)]
pub struct Skeleton {
    /// Name of skeleton
    pub name: String,
    /// ID
    pub id: i32,
    /// List of `RigidBody` descriptions
    pub bones: Vec<RigidBody>,
}

/// Description of dataset
#[derive(Clone, Debug, PartialEq)]
pub enum DataSet {
    /// Description of a `MarkerSet`
    MarkerSet(MarkerSet),
    /// Description of a `RigidBody`
    RigidBody(RigidBody),
    /// Description of a `Skeleton`
    Skeleton(Skeleton),
}

/// Private type to match against
enum DataSetType {
    MarkerSet = 0,
    RigidBody = 1,
    Skeleton = 2,
}

impl Unpack<DataSet> for DataSet {
    fn unpack<B: BufRead>(ver: &Version, bytes: &mut B) -> Result<DataSet> {
        let d_type = try!(bytes.read_i32::<LittleEndian>());
        match d_type {
            _ if d_type == DataSetType::MarkerSet as i32 => {
                Ok(DataSet::MarkerSet(try!(MarkerSet::unpack(ver, bytes))))
            }
            _ if d_type == DataSetType::RigidBody as i32 => {
                Ok(DataSet::RigidBody(try!(RigidBody::unpack(ver, bytes))))
            }
            _ if d_type == DataSetType::Skeleton as i32 => {
                Ok(DataSet::Skeleton(try!(Skeleton::unpack(ver, bytes))))
            }
            _ => unreachable!(),
        }
    }
}

impl Unpack<MarkerSet> for MarkerSet {
    fn unpack<B: BufRead>(_: &Version, bytes: &mut B) -> Result<MarkerSet> {
        let name = try!(read_cstring(bytes));
        let num_markers = try!(bytes.read_i32::<LittleEndian>());
        let mut markers = Vec::with_capacity(num_markers as usize);
        for _ in 0..num_markers {
            markers.push(try!(read_cstring(bytes)));
        }
        Ok(MarkerSet {
            name: name,
            markers: markers,
        })
    }
}

impl Unpack<RigidBody> for RigidBody {
    fn unpack<B: BufRead>(_: &Version, bytes: &mut B) -> Result<RigidBody> {
        let name = try!(read_cstring(bytes));
        let id = try!(bytes.read_i32::<LittleEndian>());
        let p_id = try!(bytes.read_i32::<LittleEndian>());
        let x = try!(bytes.read_f32::<LittleEndian>());
        let y = try!(bytes.read_f32::<LittleEndian>());
        let z = try!(bytes.read_f32::<LittleEndian>());
        Ok(RigidBody {
            name: name,
            id: id,
            parent_id: p_id,
            offset: Vector3::new(x, y, z),
        })
    }
}

impl Unpack<Skeleton> for Skeleton {
    fn unpack<B: BufRead>(ver: &Version, bytes: &mut B) -> Result<Skeleton> {
        let name = try!(read_cstring(bytes));
        let id = try!(bytes.read_i32::<LittleEndian>());
        let num_rb = try!(bytes.read_i32::<LittleEndian>());
        let mut bodies = Vec::with_capacity(num_rb as usize);
        for _ in 0..num_rb {
            bodies.push(try!(RigidBody::unpack(ver, bytes)));
        }
        Ok(Skeleton {
            name: name,
            id: id,
            bones: bodies,
        })
    }
}
