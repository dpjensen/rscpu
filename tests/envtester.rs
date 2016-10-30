/*
Integration tests.
This runs against the same test suite used for the lscpu regression tests.
There's a couple different tests on here, for different platforms.

If you have the lscpu source, you can find the tarballs under util-linux/tests/ts/lscpu

Some stats (arch, Byte order) don't read from files, so the tests will technically not "valid"
(for example, the arm7 test will say x86, as it reads from the uname syscall) but they will be the same
across the local machine they are being tested on.
*/
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::ffi::OsString;

/*
This is the struct used to run the lscpu tests
you pass the -s flag to create a virtual
*/
#[allow(non_camel_case_types)]
pub struct envTester{
    our_map:Vec<String>,
    ref_map:Vec<String>
}

impl envTester {

    pub fn start_test(&self) {
        //now we actually compare the two
        if self.ref_map.len() != self.our_map.len() {
            println!("{:?}", self.ref_map);
            println!("{:?}", self.our_map);
            panic!("Sizes of the two outputs are not equal.")
        }
        //iterate time...
        for datapoint in 0..self.ref_map.len(){
            println!("STARTING TEST");
            normalize_and_compare(&self.ref_map[datapoint], &self.our_map[datapoint]);
        }
    }

    pub fn new(tname:&str) -> envTester {
        let test_path = get_test_path().join(tname).into_os_string();
        let exe_path = get_exe_path().into_os_string();
        let ref_test = OsString::from("lscpu");

        let ref_map =  get_map_from_exe(&test_path, &ref_test);
        let our_map =  get_map_from_exe(&test_path, &exe_path);

        envTester{
            ref_map: ref_map,
            our_map: our_map,
        }
    }
} // end of impl

fn get_exe_path() -> PathBuf {
    let root = env::current_exe().unwrap().parent().expect("Root debug path").to_path_buf().join("rscpu");

    root
}

fn get_test_path() -> PathBuf {
    let root = env::current_exe().unwrap()
                                .parent().expect("exe path")
                                .parent().expect("debug path")
                                .parent().expect("target path")
                                .join("tests");

    root
}

fn get_map_from_exe(s_path:&OsString, exe_path:&OsString) -> Vec<String> {

    let ref_output = match Command::new(exe_path).arg("-s").arg(s_path).output() {
        Ok(o) => { o }
        Err(e) => { panic!("Could not start test: {}", e) }
    };
    println!("{:?}", String::from_utf8_lossy(&ref_output.stderr));
    assert!(ref_output.status.success());

    let ref_str = String::from_utf8_lossy(&ref_output.stdout);
    let mut test_map:Vec<String> = Vec::new();
    for line in ref_str.split_terminator("\n"){
        let split_line:Vec<&str> = line.split(":").collect();
        assert_eq!(split_line.len(), 2);
        test_map.push(split_line[1].trim().to_string());
    }

    test_map
}

/*
Fields are not going to be 100% identical, so...we gotta do this.
*/
fn normalize_and_compare(comp1:&str, comp2:&str) {

    //check to see if we have numbers.
    let int_check1 = comp1.parse::<f32>();
    let int_check2 = comp2.parse::<f32>();
    if int_check2.is_ok() && int_check1.is_ok(){
        let i1 = int_check1.unwrap();
        let i2 = int_check2.unwrap();
        assert_eq!(i1, i2);
        println!("Finished compare of {} :: {}", comp1, comp2);
        return;
    } //end of num checks

    //normalize any delimiters, just to be source
    let tests1 = comp1.replace(|c| c == '-' || c == ',', "");
    let tests2 = comp2.replace(|c| c == '-' || c == ',', "");
    assert_eq!(tests1, tests2);


    println!("Finished compare of {} :: {}", comp1, comp2);

}
