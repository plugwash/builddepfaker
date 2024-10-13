use std::borrow::Cow;
use indexmap::IndexMap;
use unicase::Ascii;

pub struct Deb822Fast<'a> {
    pub paragraphs : Vec<IndexMap<unicase::Ascii<&'a str>,Cow<'a,[u8]>>>
}

impl <'a> Deb822Fast<'a> {
    pub fn new(contents: &'a[u8]) -> Self {
        let mut result = Self { paragraphs: Vec::new() };
        let len = contents.len();
        let mut fieldstart = 0;
        let mut paragraph = IndexMap::with_capacity(16);
        for i in 0..len {
            if (contents[i] != 0x0A) && i != len - 1 {
                continue
            }
            let endofpara = (i == len-1) || (contents[i+1] == 0x0A);
            let endoffield = endofpara || ((contents[i+1] != 0x20) && (contents[i+1] != 0x09));
            if endoffield {
                let mut fieldcontent = &contents[fieldstart..i+1];
                if fieldcontent.last().copied() == Some(b'\n') {
                    fieldcontent = &fieldcontent[0..fieldcontent.len() - 1];
                }
                let colonpos = fieldcontent.iter().position(|c| *c == b':');
                if let Some(colonpos) = colonpos {
                    let fieldname = Ascii::new(std::str::from_utf8(&fieldcontent[0..colonpos]).unwrap());
                    let mut fieldcontent = &fieldcontent[colonpos+1..];
                    if (fieldcontent.len() >= 2) && (fieldcontent[0] == b' ') && (fieldcontent[1] != b'\n') {
                        fieldcontent = &fieldcontent[1..fieldcontent.len()];
                    }
                    paragraph.insert(fieldname,Cow::Borrowed(fieldcontent));
                } else {
                    if fieldcontent.len() > 0 {
                        panic!("failed to find : in line {:?}",String::from_utf8_lossy(fieldcontent));
                    }
                }
                fieldstart = i + 1;
            }
            if endofpara {
                //if result.paragraphs.len() == 0 {
                //    let countestimate = contents.len() / (i/(result.paragraphs.len()+1));
                //    //println!("reserving space for {} entries",countestimate);
                //    result.paragraphs.reserve(countestimate);
                //}
                if paragraph.len() != 0 {
                    result.paragraphs.push(paragraph);
                    paragraph = IndexMap::with_capacity(16);
                }
            }
        }
        result
    }

    pub fn write(&self, target: &mut impl std::io::Write) -> Result<(),std::io::Error>{
        for paragraph in &self.paragraphs {
            for (fieldname, fieldcontents) in paragraph {
                target.write(fieldname.as_bytes())?;
                if (fieldcontents.first().copied() == Some(b'\n')) || (fieldcontents.first().copied() == Some(b' ')) {
                    target.write(b":")?;
                } else {
                    target.write(b": ")?;
                }
                target.write(fieldcontents)?;
                target.write(b"\n")?;
            }
            target.write(b"\n")?;

        }
        Ok(())

    }


}