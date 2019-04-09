extern crate osv2osv;

use osv2osv::*;
use std::fs::File;
use std::fs;
use std::io::Read;

fn test_osv(csv_file: &str, eml_file: &str, correct_ts: &str) {
    let default_ts = "20T12:34:56.789";
    let csv_rdr = Box::new(File::open(csv_file).expect("Could not read csv file")) as Box<Read>;
    assert_eq!(
        data2eml(csv2data(csv_rdr)).replace(default_ts, correct_ts),
        fs::read_to_string(eml_file).expect("Could not read eml file"));
}

#[test]
fn ps2019() {
    test_osv(
        "tests/data/osv4-3_Telling_PS2019_Fryslan_gemeente_Tytsjerksteradiel.csv",
        "tests/data/Telling_PS2019_Fryslan_gemeente_Tytsjerksteradiel.eml.xml",
        "21T06:57:22.834"
    );
}

#[test]
fn ab2019() {
    test_osv(
        "tests/data/osv4-3_Telling_PS2019_Fryslan_gemeente_Tytsjerksteradiel.csv",
        "tests/data/Telling_PS2019_Fryslan_gemeente_Tytsjerksteradiel.eml.xml",
        "21T06:57:22.834"
    );
}

