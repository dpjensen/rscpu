extern crate getopts;
use std::fs::File;
use std::io::Error;
use std::io::prelude::*;
use getopts::Matches;
use std::path::Path;

pub fn manage_sysroot(matches: Matches) -> Result<String, &'static str> {

    if !matches.opt_present("s"){
        return Ok("".to_string());
    }

        let str_path = matches.opt_str("s").unwrap().to_string();
        let path = Path::new(&str_path);
        if !path.exists() {
            return Err("Path does not exist");
        }
        if path.is_file(){
            return Err("You supplied a file, not a path.");
        }
        let final_path = path.to_str().unwrap().to_string();

        Ok(final_path)

}

pub fn open_file_as_string(fname:&str) -> Result<String, Error> {


    let mut fd = try!(File::open(fname));
    let mut file_string = String::new();

    try!(fd.read_to_string(&mut file_string));

    Ok(file_string)

}

//There's a chance that some of these files won't exist, so...
pub fn check_file(path: &str) -> bool {

    let status = File::open(path);
    match status{
        Ok(_) => return true,
        _     => return false,
    }
}
