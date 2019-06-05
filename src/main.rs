extern crate osv2osv;
extern crate structopt;

use osv2osv::*;
use std::io::Read;
use std::io;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "osv2osv")]
struct Opt {
    /// sha256sum to crack creation time and version for
    #[structopt(short = "h", long = "hashcode")]
    hashcode: Option<String>,

    /// Version numbers to try
    #[structopt(short = "v", long = "versionlist")]
    versions: Vec<String>,

    /// Date strings to try
    #[structopt(short = "d", long = "datelist")]
    dates: Option<String>,

    // /// Minute to start
    // #[structopt(short = "m", long = "minute")]
    // dates: Option<String>,
}


fn main() {
	let opt = Opt::from_args();
    let contest = csv2contest(Box::new(io::stdin()) as Box<Read>);
    let eml = match opt.hashcode {
    	Some(ref hashcode) => contest2eml_find_sha(&contest, hashcode.to_string(), opt.versions, opt.dates),
		None => contest2eml(&contest)
	};
    print!("{}", eml);
}