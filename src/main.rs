extern crate getopts;

mod file_handler;
use file_handler::*;

mod printers;
use printers::*;

use std::collections::BTreeMap;
use getopts::Options;
use std::env;

/*
    We can use the core_siblings_list attribute
    to extrapolate sockets, hardware cores, etc.
    The value in this field should be the logical cores per physical package
*/
fn get_core_siblings(sysroot:&str) -> Result<i32, &'static str>{

    let base_path = "/sys/devices/system/cpu/cpu0/topology/core_siblings_list";
    let check_path = format!("{}{}",sysroot, base_path);

    if !check_file(&check_path){
        return Err("No core_siblings list found");
    }

    let core_siblings:String = open_file_as_string(&check_path).unwrap();
    println!("{:?}", core_siblings);
    //two formats I've found for the core_siblings_list file: start-end and a,b,c...
    //TODO: Error handling
    if core_siblings.contains("-"){
        let end_val = core_siblings.split("-").collect::<Vec<&str>>()[1].trim();
        let val_int = end_val.parse::<i32>().unwrap() + 1;
        return Ok(val_int);
    } else if core_siblings.contains(","){
        let sib_count = core_siblings.split(",").collect::<Vec<&str>>().len();
        return Ok(sib_count as i32);
    } else {
        //¯\_(ツ)_/¯
        return Ok(1);
    }

}


/*
    we extrapolate threads per core based on data from
    /sys/devices/sy6stem/cpu/cpu/cpu0/topology/thread_siblings_list
    `lscpu` is doing something similar, but with bitmasks in thread_siblings
*/
fn get_threads_per_core(sysroot:&str) -> Result<i32, &'static str>{

    let base_path = "/sys/devices/system/cpu/cpu0/topology/thread_siblings_list";
    let check_path = format!("{}{}", sysroot, base_path);

    if !check_file(&check_path){
        return Err("thread_siblings_list not found");
    }

    let thread_siblings:String = open_file_as_string(&check_path).unwrap();
    let thread_count:usize = thread_siblings.split(",").collect::<Vec<&str>>().len();

    Ok(thread_count as i32)

}


fn read_basic_info(sysroot:String) -> BTreeMap<String, String> {

    let mut data:BTreeMap<String, String> = BTreeMap::new();
    let base_info = "/proc/cpuinfo";

    let info_file = format!("{}{}",sysroot, base_info);

    let info_str = open_file_as_string(&info_file).unwrap();
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

    //we get these as ints, and will use them to extrapolate sockets, hw cores, etc.
    //If we can't get this data, extrapolated data won't be printed as well.
    let core_siblings = match get_core_siblings(&sysroot){
        Ok(c)  => { c }
        Err(e) => {println!("{}", e); -1 }
    };
    let threads_per_core = match get_threads_per_core(&sysroot){
        Ok(t)  => { t }
        Err(e) => {println!("{}", e); -1 }
    };

    if core_siblings > 0 && threads_per_core > 0 && processor != "" {
        let total_cpus = processor.parse::<i32>().unwrap() + 1;
        let cores_per_socket = core_siblings / threads_per_core;
        let hw_sockets = total_cpus / threads_per_core / cores_per_socket;

        data.insert("cores_per_socket".to_string(), cores_per_socket.to_string());
        data.insert("sockets".to_string(), hw_sockets.to_string());
        data.insert("threads_per_core".to_string(), threads_per_core.to_string());

    }

    //How we get the total logical CPUs
    if processor != ""  {
        data.insert("CPUs".to_string(), ((processor.parse::<i32>().unwrap()) + 1).to_string());
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
                                ("Socket(s):", "sockets"),
                                ("Vendor ID:", "vendor_id"),
                                ("CPU Family:", "cpu family"),
                                ("Model:", "model"),
                                ("Model Name:", "model name"),
                                ("Stepping:", "stepping"),
                                ("CPU MHz:", "cpu MHz"),
                                ("BogoMIPS:", "bogomips"),
                                ("Flags:", "flags")
                                ];


    let mut basic = read_basic_info(sysroot);


    normal_print(&basic, known_datapoints);

    //println!("\n{:?}", basic);

}
