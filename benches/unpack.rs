#![feature(test)]

extern crate natnet_decode;
extern crate semver;
extern crate test;

use natnet_decode::NatNet;
use semver::Version;
use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use test::Bencher;

fn help_open(file_name: String) -> Cursor<Vec<u8>> {
    let mut f = File::open(file_name).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    Cursor::new(buf)
}

#[bench]
fn parse_2_5(b: &mut Bencher) {
    let parser = NatNet::new(Version::parse("2.5.0").unwrap());
    let mut buf = help_open(format!("tests/data/frame-motive-1.5.0-001.bin"));
    b.iter(|| {
        parser.unpack(&mut buf).unwrap();
        buf.set_position(0);
    });
}

#[bench]
fn parse_2_7(b: &mut Bencher) {
    let parser = NatNet::new(Version::parse("2.7.0").unwrap());
    let mut buf = help_open(format!("tests/data/frame-motive-1.7.2-001.bin"));
    b.iter(|| {
        parser.unpack(&mut buf).unwrap();
        buf.set_position(0);
    });
}

#[bench]
fn parse_2_9(b: &mut Bencher) {
    let parser = NatNet::new(Version::parse("2.9.0").unwrap());
    let mut buf = help_open(format!("tests/data/frame-motive-1.9.0-001.bin"));
    b.iter(|| {
        parser.unpack(&mut buf).unwrap();
        buf.set_position(0);
    });
}
