use byteorder::{ReadBytesExt, LittleEndian};
use rigid_body::RigidBody;
use semver::Version;
use std::io::BufRead;
use super::{Result, Unpack};

/// A `Skeleton` is a collection of `RigidBody`
#[derive(Clone, Debug, PartialEq)]
pub struct Skeleton {
    /// ID of skeleton
    pub id: i32,
    /// Vector of rigid bodies comprising this skeleton
    pub bones: Vec<RigidBody>,
}

impl Unpack<Skeleton> for Skeleton {
    fn unpack<B: BufRead>(ver: &Version, bytes: &mut B) -> Result<Skeleton> {
        let id = try!(bytes.read_i32::<LittleEndian>());
        let num_bodies = try!(bytes.read_i32::<LittleEndian>());
        let mut bodies = Vec::with_capacity(num_bodies as usize);
        for _ in 0..num_bodies {
            bodies.push(try!(RigidBody::unpack(ver, bytes)));
        }
        Ok(Skeleton {
            id: id,
            bones: bodies,
        })
    }
}
