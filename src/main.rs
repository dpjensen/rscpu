extern crate getopts;

mod file_handler;
use file_handler::*;

mod printers;
use printers::*;

use std::collections::BTreeMap;
use getopts::Options;
use std::env;
/*
    we extrapolate threads per core based on data from
    /sys/devices/sy6stem/cpu/cpu/cpu0/topology/thread_siblings_list
    `lscpu` is doing something similar, but with bitmasks in thread_siblings
*/
fn get_threads_per_core(info_list:&mut BTreeMap<String, String>){

    let check_path = "/sys/devices/system/cpu/cpu0/topology/thread_siblings_list";

    if check_file(check_path){
        let thread_siblings:String = open_file_as_string(check_path).unwrap();
        info_list.insert("threads_per_core".to_string(), thread_siblings
                                                            .split(",")
                                                            .collect::<Vec<&str>>()
                                                            .len().to_string());
    } else {
        println!("Threads Per Core currently not supported on platforms without {}", check_path);
        return;
    }

}


fn read_basic_info() -> BTreeMap<String, String> {

    let mut data:BTreeMap<String, String> = BTreeMap::new();

    let info_file = "/proc/cpuinfo";

    let info_str = open_file_as_string(info_file).unwrap();
    let mut processor:String = String::new();


    for line in info_str.lines(){
        let line_parts:Vec<&str> = line.split(":").collect();
        if line_parts.len() == 1{
            continue;
        }
        //we're using this to grab total number of logical CPUs
        //No, we can't just check the map later because of rust's non-lexical borrows
        //see github issue 6393
        if line_parts[0].trim() == "processor"{
            processor = line_parts[1].trim().to_string();
        }
        data.insert(line_parts[0].trim().to_string(), line_parts[1].trim().to_string());

    }

    //How we get the total logical CPUs
    if processor != ""  {
        data.insert("CPUs".to_string(), ((processor.parse::<i32>().unwrap()) + 1).to_string());
    }
    //now the threads per core
    get_threads_per_core(&mut data);
    //and now cores per socket
    if  processor != "" && data.get("threads_per_core").is_some(){

        let result = ((processor.parse::<i32>().unwrap()) + 1) /
                    (data.get("threads_per_core").unwrap().parse::<i32>().unwrap());

        data.insert("cores_per_socket".to_string(), result.to_string());
    }

    data

}




fn main() {

    let mut opts = Options::new();
    let args: Vec<String> = env::args().collect();

    opts.optopt("s", "", "Set a custom file root, as opposed to `/` ", "SYSROOT");
    opts.optflag("h", "help", "Print this help menu");

    let matches = match opts.parse(&args[1..]){
        Ok(m)  => { m }
        Err(t) => { panic!(t.to_string()) }
    };

    if matches.opt_present("h"){
        println!("{}", opts.usage("Usage: rscpu [Options]"));
        return;
    }

    //manage the root sysdir
    let sysroot = match manage_sysroot(matches){
        Ok(m)  => { m }
        Err(e) => { panic!(e.to_string()) }
    };

    //This vector is the primary source of truth.
    // Format: ([Printable Name], [Name in dict])
    //because the formats of /proc/cpuinfo and /sys/ can vary, make no assumptions
    let known_datapoints = vec![("CPUs:","CPUs"),
                                ("Threads Per Core:", "threads_per_core"),
                                ("Core(s) Per Socket:", "cores_per_socket"),
                                ("Vendor ID:", "vendor_id"),
                                ("CPU Family:", "cpu family"),
                                ("Model:", "model"),
                                ("Model Name:", "model name"),
                                ("Stepping:", "stepping"),
                                ("CPU MHz:", "cpu MHz"),
                                ("BogoMIPS:", "bogomips"),
                                ("Flags:", "flags")
                                ];


    let mut basic = read_basic_info();


    normal_print(&basic, known_datapoints);

    println!("{:?}", basic);

}
