extern crate env_logger;
extern crate natnet_decode;
extern crate semver;

use natnet_decode::{NatNet, NatNetResponse};
use semver::Version;
use std::fs::File;
use std::io::BufReader;

#[test]
fn verions() {
    let files = vec![
        "tests/data/frame-motive-1.5.0-000.bin",
        "tests/data/frame-motive-1.7.2-000.bin",
        "tests/data/frame-motive-1.9.0-000.bin"];
    let versions = vec![
        Version::parse("2.5.0").unwrap(),
        Version::parse("2.7.0").unwrap(),
        Version::parse("2.9.0").unwrap()];
    for (f_name, version) in files.iter().zip(versions.iter()) {
        let f = File::open(f_name).unwrap();
        let mut buf = BufReader::new(f);
        if let NatNetResponse::Ping(sender)
            = NatNet::unpack_with(&version, &mut buf).unwrap() {
                assert_eq!("NatNetLib", sender.name);
                assert_eq!(*version, sender.natnet_version);
        } else {
            assert!(false);
        }
    }
}

fn test_parse(p: &NatNet, file_name: String) -> NatNetResponse {
    let f = File::open(file_name).unwrap();
    let mut buf = BufReader::new(f);
    p.unpack(&mut buf).unwrap()
}

#[test]
fn parse_2_5() {
    let parser = NatNet::new(Version::parse("2.5.0").unwrap());
    for i in 1..3 {
        let f_name = format!("tests/data/frame-motive-1.5.0-00{}.bin", i);
        if let NatNetResponse::FrameOfData(frame) = test_parse(&parser, f_name) {
            assert!(frame.marker_sets.contains_key("all"));
            assert!(frame.marker_sets.contains_key("Rigid Body 1"));
            assert_eq!(frame.other_markers.len(), 2);
            assert_eq!(frame.rigid_bodies.len(), 1);
            assert_eq!(frame.skeletons.len(), 0);
            assert_eq!(frame.labeled_markers.len(), 3);
            assert!(frame.force_plates.is_none());
            assert!(frame.timestamp.is_none());
            assert!(frame.is_recording.is_none());
            assert!(frame.tracked_models_changed.is_none());
        }
    }
}

#[test]
fn parse_2_7() {
    let parser = NatNet::new(Version::parse("2.7.0").unwrap());
    for i in 1..3 {
        let f_name = format!("tests/data/frame-motive-1.7.2-00{}.bin", i);
        if let NatNetResponse::FrameOfData(frame) = test_parse(&parser, f_name) {
            assert!(frame.marker_sets.contains_key("all"));
            assert!(frame.marker_sets.contains_key("Rigid Body 1"));
            assert_eq!(frame.other_markers.len(), 2);
            assert_eq!(frame.rigid_bodies.len(), 1);
            assert_eq!(frame.skeletons.len(), 0);
            assert_eq!(frame.labeled_markers.len(), 3);
            assert!(frame.force_plates.is_none());
            assert!(frame.timestamp.is_some());
            assert!(frame.is_recording.is_some());
            assert!(frame.tracked_models_changed.is_some());
        }
    }
}

#[test]
fn parse_2_9() {
    let parser = NatNet::new(Version::parse("2.9.0").unwrap());
    for i in 1..3 {
        let f_name = format!("tests/data/frame-motive-1.9.0-00{}.bin", i);
        if let NatNetResponse::FrameOfData(frame) = test_parse(&parser, f_name) {
            assert!(frame.marker_sets.contains_key("all"));
            assert!(frame.marker_sets.contains_key("Triangle"));
            assert_eq!(frame.other_markers.len(), 0);
            assert_eq!(frame.rigid_bodies.len(), 1);
            assert_eq!(frame.skeletons.len(), 0);
            assert_eq!(frame.labeled_markers.len(), 3);
            assert!(frame.force_plates.is_some());
            assert!(frame.timestamp.is_some());
            assert!(frame.is_recording.is_some());
            assert!(frame.tracked_models_changed.is_some());
        }
    }
}
