extern crate osv2osv;

use std::io;
use std::io::Read;
use osv2osv::*;

fn main() {
    print!("{}", data2eml(csv2data(Box::new(io::stdin()) as Box<Read>)));
}
