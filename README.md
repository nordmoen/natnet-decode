# `NatNet`-decode
[![Build
Status](https://travis-ci.org/nordmoen/natnet-decode.svg?branch=master)](https://travis-ci.org/nordmoen/natnet-decode) [![Crates.io](https://img.shields.io/crates/v/natnet-decode.svg)](https://crates.io/crates/natnet-decode)[![Docs.rs](https://docs.rs/natnet-decode/badge.svg)](https://docs.rs/natnet-decode/)

Decode `NatNet` messages in Rust.

[`NatNet`](http://www.optitrack.com/downloads/developer-tools.html#natnet-sdk)
is the software solution supporting [OptiTrack](http://www.optitrack.com/)
many offerings. Since motion tracking data is multicast it can be decoded
in any language and this crate is a Rust (unofficial!) implementation.

# Example
```rust,ignore
use natnet_decode::NatNet;
use semver::Version;

let mut data = Vec::new();
// Fill data here
// ...

// We can then decode this:
let parsed = NatNet::unpack_with(&Version::parse("2.5.0").unpack(), &mut data.as_ref());
// Do stuff with parsed
println!("{:?}", parsed);
```

# Acknowledgement
This crate is heavily inspired by
[python-optirx](https://bitbucket.org/astanin/python-optirx/overview) and
test data is borrowed with permission.
