use std::fs;
use std::env;
use std::time::Instant;
use std::io::{BufWriter, Write};

use deb822_fast::Deb822Fast;
fn main () {
    let args : Vec<String> = env::args().collect();
    let start = Instant::now();
    let data = fs::read(&args[1]).unwrap();
    println!("data read {:?} elapsed",start.elapsed());
    let parsed = Deb822Fast::new(&data);
    println!("data parsed {:?} elapsed",start.elapsed());
    /*for paragraph in parsed.paragraphs {
        println!("{:?}", paragraph)
    }*/
    let outputfile = fs::File::create("filtered.txt").unwrap();
    let mut outputfile = BufWriter::new(outputfile);
    parsed.write(&mut outputfile);
    outputfile.flush().unwrap();
    println!("output written {:?} elapsed",start.elapsed());
    
}
