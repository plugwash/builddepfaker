use std::fs::File;
use std::fs;
use std::str;
use std::io::BufReader;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::BufRead;
//use deb822_lossless::Deb822;
use std::borrow::Cow;
use std::borrow::Cow::Borrowed;
use std::borrow::Cow::Owned;
use std::time::Instant;
use deb822_fast::Deb822Fast;
use unicase::Ascii;
use std::ops::Deref;
use std::io::BufWriter;
use std::io::Write;

fn main() {
    println!("Hello, world!");
    let starttime = Instant::now();
    //f = open("/home/repo/private/private/conf/fakebuilddeps")
    let args : Vec<String> = std::env::args().collect();
    let f = File::open(&args[1]).unwrap();
    let b = BufReader::new(f);
    let fakelines : Vec<String> = b.lines().map(|x| x.unwrap()).collect();
    let fakelines : Vec<&str> = fakelines.iter().map(|s| s.as_ref()).collect();

    let mut fakedepdict : HashMap<&str,(HashSet<_>,HashSet<_>)> = HashMap::new();

    for fakeline in fakelines {
	if fakeline.contains(':') {
		let (package, fakedeps) = fakeline.split_once(':').unwrap();
		let fakedeps : Vec<&str> = fakedeps.split(',').collect();
		let empty = (HashSet::new(),HashSet::new());
		let (negative,positive) = fakedepdict.get(package).unwrap_or(&empty);
		let mut negative = negative.clone();
		let mut positive = positive.clone();
		for mut fakedep in fakedeps {
			fakedep = fakedep.trim();
			if fakedep.chars().next().unwrap() == '!' {
				negative.insert(&fakedep[1..]);
			} else {
				positive.insert(fakedep);
			}
		}
		fakedepdict.insert(package,(negative,positive));
	}
    }
    eprintln!("fakedepdict built {}",starttime.elapsed().as_secs_f64());
    //print(repr(fakedepdict))

    //exit()

    //f = File::open("/home/repo/private/private/dists/buster-staging/main/source/Sources")
    let f = &args[2];

    //fakelist = ()
    let data = fs::read(f).unwrap();
    let mut d822 = Deb822Fast::new(&data);
    eprintln!("d822 built {}",starttime.elapsed().as_secs_f64());


    for mut entry in &mut d822.paragraphs {
	let package = entry.get(&Ascii::new("Package")).unwrap();
	//println!("looking for {:?} in fakedepdict",str::from_utf8(package).unwrap());	
	if let Some((negative,positive)) = fakedepdict.get(str::from_utf8(package).unwrap()) {
		println!("found enty in fakedepdict for package {}",str::from_utf8(package).unwrap());
		//if package == 'gcc-6':
		//	sys.stderr.write(repr((negative,positive)))
		let mut positive = positive.clone();
		let mut processedbuilddeps : Vec<Cow<str>> = Vec::new();
		
		let builddepsstr = entry.get(&Ascii::new("Build-Depends")).unwrap_or(&Cow::Borrowed(b""));
		let builddeps : Vec<&[u8]> = if builddepsstr.deref() != b"" {
			builddepsstr.split(|c| *c == b',').collect()
		} else {
			Vec::new()
		};
		for depstr in builddeps {
			let depname = str::from_utf8(depstr).unwrap().trim().split(' ').next().unwrap().split(':').next().unwrap();
			//if package == 'gcc-6':
			//	sys.stderr.write(depname+'\n')
			if ! negative.contains(depname) {
				processedbuilddeps.push(Borrowed(str::from_utf8(depstr).unwrap()));
			}
			//if package == 'gcc-6':
			//	sys.stderr.write(depname+' output\n')
			positive.remove(depname);
		}
		for depstr in positive {
			processedbuilddeps.push(Owned(" ".to_owned()+depstr+" (!= 0.fake.0)"));
		}
		entry.insert(Ascii::new("Build-Depends"),Cow::Owned(processedbuilddeps.join(",").into_bytes()));
		if entry.contains_key(&Ascii::new("Build-Depends-Indep")) {
			let builddepsstr = entry.get(&Ascii::new("Build-Depends-Indep")).unwrap();
			let builddeps = builddepsstr.split(|c| *c == b',');
			processedbuilddeps = Vec::new();
			for depstr in builddeps {
				let depname = str::from_utf8(depstr).unwrap().trim().split(' ').next().unwrap();
				//print(depname)
				if ! negative.contains(depname) {
					processedbuilddeps.push(Borrowed(str::from_utf8(depstr).unwrap()));
				}
			}
			entry.insert(Ascii::new("Build-Depends-Indep"),Cow::Owned(processedbuilddeps.join(",").into_bytes()));
		}
	}
	//println!("{}",entry);
    }
    let outputfile = fs::File::create(&args[3]).unwrap();
    let mut outputfile = BufWriter::new(outputfile);
    d822.write(&mut outputfile);
    outputfile.flush().unwrap();

    eprintln!("faking done {}",starttime.elapsed().as_secs_f64());
    eprintln!("output written {}",starttime.elapsed().as_secs_f64());
    

}
