use byteorder::ReadBytesExt;
use semver::{Version, Identifier};
use std::io::BufRead;
use super::{Result, Unpack, read_cstring};

/// `NatNet` application identifier
///
/// This struct represents a `NatNet` application that is sending data.
///
/// # Note
/// This struct uses `Version` for ease of use, there
/// is no guarantee from `NatNet` that applications must follow semantic
/// versioning.
#[derive(Clone, Debug, PartialEq)]
pub struct Sender {
    /// Name of application sending data
    pub name: String,
    /// Internal version of sender application
    pub version: Version,
    /// `NatNet` version the sender application is using
    pub natnet_version: Version,
}

/// Helper function to unpack `NatNet` version into a `semver::Version`
fn unpack_version<B: BufRead>(bytes: &mut B) -> Result<Version> {
    let v1 = try!(bytes.read_u8()) as u64;
    let v2 = try!(bytes.read_u8()) as u64;
    let v3 = try!(bytes.read_u8()) as u64;
    let v4 = try!(bytes.read_u8()) as u64;
    Ok(Version {
        major: v1,
        minor: v2,
        patch: v3,
        pre: vec![],
        build: vec![Identifier::Numeric(v4)],
    })
}

impl Unpack<Sender> for Sender {
    fn unpack<B: BufRead>(_: &Version, bytes: &mut B) -> Result<Sender> {
        debug!("Unpacking application identifier");
        let name = try!(read_cstring(bytes));
        // NOTE: The application name always contains 256 bytes, so we need to
        // throw away the rest, the `-1` at the end is for the `'\0'` byte
        bytes.consume(256 - name.as_bytes().len() - 1);
        let ver = try!(unpack_version(bytes));
        let nat = try!(unpack_version(bytes));
        trace!("Found application name {:?}, version {} using NatNet version {}",
               name,
               ver,
               nat);
        Ok(Sender {
            name: name,
            version: ver,
            natnet_version: nat,
        })
    }
}
