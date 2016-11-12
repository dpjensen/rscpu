extern crate getopts;
extern crate libc;

mod file_handler;
use file_handler::*;

mod printers;
use printers::*;

use std::fs;
use std::collections::BTreeMap;
use getopts::Options;
use std::env;
use libc::{uname, c_int};
use std::process::exit;

/*
    This part of the virtualization detection looks for vendor info from
    /proc/bus/pci/devices
*/
fn has_pci_device(vendor:u32, device:u32, sysroot:&str) -> bool {

    let base_path = "/proc/bus/pci/devices";
    let node_path =  format!("{}{}", sysroot, base_path);

    let dev_str = open_file_as_string(&node_path);
    if dev_str.is_err(){
        return false
    }

    for line in dev_str.unwrap().lines(){
        let mask2 = 0x0000ffff;

        let ven_dev:Vec<&str> = line.split("\t").collect();
        let ven_dev_int = u32::from_str_radix(ven_dev[1], 16);

        if ven_dev_int.is_err(){
            continue;
        }
        let to_mask = ven_dev_int.unwrap();
        let dev = to_mask & mask2;
        let ven = to_mask >> 0x10;

        if dev == device && ven == vendor{
            return true;
        }
    }

    return false;


}

/*
    Master method for detecting a hypervisor
*/
fn handle_hypervisor(sysroot:&str) -> Option<BTreeMap<String, String>> {
    //Hex vendor IDs
    let hyper_vbox = 0x80ee;
    let hyper_xen = 0x5853;
    //let hyper_kvm = 0x0000;
    //let hyper_mshv = 0x1414;
    let hyper_vmware = 0x15ad;

    //hex graphics device IDs
    let graphics_xen = 0x0001;
    //let graphics_kvm = 0x0000;
    //let graphics_mshv = 0x5353;
    let graphics_vmware = 0x0710;
    let graphics_vbox = 0xbeef;

    let mut visor_map:BTreeMap<String, String> = BTreeMap::new();

    if has_pci_device(hyper_vbox, graphics_vbox, sysroot){
        visor_map.insert("hypervisor_vendor".to_string(), "Oracle".to_string());
        visor_map.insert("hypervisor_type".to_string(), "full".to_string());
    } else if has_pci_device(hyper_vmware, graphics_vmware, sysroot){
        visor_map.insert("hypervisor_vendor".to_string(), "VMWare".to_string());
        visor_map.insert("hypervisor_type".to_string(), "full".to_string());

    } else if has_pci_device(hyper_xen, graphics_xen, sysroot){
        visor_map.insert("hypervisor_vendor".to_string(), "Xen".to_string());
        visor_map.insert("hypervisor_type".to_string(), "full".to_string());

    }

    if visor_map.len() != 0{
        return Some(visor_map);
    } else {
        return None;
    }


}

fn handle_byte_order() -> BTreeMap<String, String> {

    let mut end_map:BTreeMap<String, String> = BTreeMap::new();
    if cfg!(target_endian = "little"){
        end_map.insert("endian".to_string(), "Little Endian".to_string());
    } else {
        end_map.insert("endian".to_string(), "Big Endian".to_string());
    }

    end_map
}

/*
    This uses the uname system call to get system info.
    As of now, don't know a way to do this other than the libc crate
*/
fn handle_uname() -> Option<BTreeMap<String, String>> {

    let mut machine_map:BTreeMap<String, String> = BTreeMap::new();
    unsafe{
        let mut name_struct:libc::utsname = libc::utsname{
            sysname:[0i8; 65],
            nodename:[0i8; 65],
            release:[0i8; 65],
            version:[0i8; 65],
            machine:[0i8; 65],
            domainname:[0i8; 65]
        };

        let result:c_int = uname(&mut name_struct);

        if result != 0{
            return None;
        }

        //I honestly don't know if there's a better way to do this...
        let mvec = std::mem::transmute::<[i8; 65], [u8; 65]>(name_struct.machine);
        let machine =  String::from_utf8(mvec.to_vec()).unwrap();
        //strip out null bytes
        machine_map.insert("arch".to_string(), machine.replace(0x00 as char, ""));


    }
    Some(machine_map)

}

fn handle_numa(sysroot:&str) -> Option<BTreeMap<String, String>> {

    //first we get the numa nodes.
    //This is mostly similar to how lscpu does it
    let base_path = "/sys/devices/system/node";
    let node_path =  format!("{}{}", sysroot, base_path);
    let mut numa_map = BTreeMap::new();

    let node_dir = match fs::read_dir(&node_path){
        Ok(n) => { n }
        Err(_) => return None

    };

    let mut node_count:i32 = 0;
    for dir_item in node_dir{
        let ending = match dir_item {
            Ok(e) =>  e.path(),
            Err(_) =>  continue
        };

        let end_str = match ending.iter().last(){
            Some(end) => { end.to_str() }
            None => { continue }
        };

        if end_str.unwrap_or("").contains("node"){
            node_count = node_count + 1;
        }

    }

    numa_map.insert("numa_nodes".to_string(), node_count.to_string());

    //now we're gonna try for numa config
    for numa_node in 0..(node_count + 1) {

        let n_node_path = format!("{}/node{}/cpumap", node_path, numa_node);

        let node_string = match open_file_as_string(&n_node_path){
            Ok(s) => { s }
            Err(_) => { continue }
        };
        let mut node_map = match u64::from_str_radix(&node_string.replace(",", ""), 16){
            Ok(nm) => { nm }
            Err(_) => { continue }
        };
        //go through and list marked nodes
        let mut numa_membership:Vec<i32> = Vec::new();
        for cpu_mem in 0..2048{
            if node_map < 1{
                break;
            }

            if node_map & 1 == 1 {
                numa_membership.push(cpu_mem);
            }
             node_map = node_map >> 1;
        }


        //if there are no 0s in bitmasks, then we don't need to print entire list
        if (numa_membership.last().unwrap_or(&0) + 1) == numa_membership.len() as i32 {
            let list_string = format!("0-{}", numa_membership.last().unwrap_or(&0));
            numa_map.insert(format!("node{}", numa_node), list_string);
        } else {
            //otherwise, make and format a list
             let list_string = format!("{:?}", numa_membership)
                            .trim_matches(|c| c == '[' || c == ']')
                            .replace(" ", "")
                            .to_string();

            //println!("{}", list_string);
            numa_map.insert(format!("node{}", numa_node), list_string);
        }
    } //end loop over node map


    Some(numa_map)


}


/*
Not all CPUs have this, obviously.
*/
fn cpu_range(sysroot:&str) -> Option<BTreeMap<String, String>>{

    let mut range_map = BTreeMap::new();

    let base_min = "/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_min_freq";
    let min_path = format!("{}{}", sysroot, base_min);
    let min = open_file_as_string(&min_path);

    let base_max = "/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq";
    let max_path = format!("{}{}", sysroot, base_max);
    let max = open_file_as_string(&max_path);

    if min.is_ok(){
        let min_float = min.unwrap().parse::<f32>();
        if min_float.is_ok(){
            let min_final:f32 = min_float.unwrap() / 1000.0;
            range_map.insert("cpu_min".to_string(), min_final.to_string());
        } else {
            return None;
        }
    } else {
        return None;
    }

    if max.is_ok(){
        let max_float = max.unwrap().parse::<f32>();
        if max_float.is_ok(){
            let max_final:f32 = max_float.unwrap() / 1000.0;
            range_map.insert("cpu_max".to_string(), max_final.to_string());
        } else {
            return None;
        }
    } else {
        return None;
    }

    Some(range_map)

}

fn get_online(sysroot:&str) -> Option<String>{
    let base_online = "/sys/devices/system/cpu/online";
    let cache_online = format!("{}{}", sysroot, base_online);

    let online = open_file_as_string(&cache_online);

    match online{
        Ok(o) => Some(o),
        Err(_) => None
    }
}

/*
    This will pull our cache info from /sys/
*/
fn handle_cache(sysroot:&str) -> Option<Vec<BTreeMap<String, String>>>{

    let base_cache = "/sys/devices/system/cpu/cpu0/cache/";
    let cache_root = format!("{}{}", sysroot, base_cache);

    let mut cache_num = 0;
    let mut cache_vec:Vec<BTreeMap<String, String>> = Vec::new();

    loop{
        let cache_state = check_path(&format!("{}index{}/",cache_root, cache_num));
        //println!("{}",format!("{}index{}",cache_root, cache_num));
        if cache_state {
            //get all of our cache stats
            let mut cache_map = BTreeMap::new();
            let cache_type = open_file_as_string(&format!("{}index{}/type", &cache_root, cache_num));
            if cache_type.is_ok(){
                cache_map.insert("type".to_string(), cache_type.unwrap());
            }
            let cache_level = open_file_as_string(&format!("{}index{}/level", &cache_root, cache_num));
            if cache_level.is_ok(){
                cache_map.insert("level".to_string(), cache_level.unwrap());
            }
            let cache_size = open_file_as_string(&format!("{}index{}/size", &cache_root, cache_num));
            if cache_size.is_ok(){
                cache_map.insert("size".to_string(), cache_size.unwrap());
            }

            cache_vec.push(cache_map);
            cache_num = cache_num + 1;

        } else {
            if cache_num == 0{
                return None;
            } else {
                return Some(cache_vec);
            }
        }
    }

}

/*
    We can use the core_siblings_list attribute
    to extrapolate sockets, hardware cores, etc.
    The value in this field should be the logical cores per physical package
*/
fn get_core_siblings(sysroot:&str) -> Option<i32>{

    let base_path = "/sys/devices/system/cpu/cpu0/topology/core_siblings_list";
    let check_path = format!("{}{}",sysroot, base_path);

    if !check_file(&check_path){
        return None;
    }

    let core_siblings:String = open_file_as_string(&check_path).unwrap();
    //println!("{:?}", core_siblings);
    //two formats I've found for the core_siblings_list file: start-end and a,b,c...
    //TODO: Error handling
    if core_siblings.contains("-"){
        let end_val = core_siblings.split("-").collect::<Vec<&str>>()[1].trim();
        let val_int = end_val.parse::<i32>().unwrap() + 1;
        return Some(val_int);
    } else if core_siblings.contains(","){
        let sib_count = core_siblings.split(",").collect::<Vec<&str>>().len();
        return Some(sib_count as i32);
    } else {
        //¯\_(ツ)_/¯
        return Some(1);
    }

}

/*
    we extrapolate threads per core based on data from
    /sys/devices/sy6stem/cpu/cpu/cpu0/topology/thread_siblings_list
    `lscpu` is doing something similar, but with bitmasks in thread_siblings
*/
fn get_threads_per_core(sysroot:&str) -> Option<i32>{

    let base_path = "/sys/devices/system/cpu/cpu0/topology/thread_siblings_list";
    let check_path = format!("{}{}", sysroot, base_path);

    if !check_file(&check_path){
        return None;
    }

    let thread_siblings:String = open_file_as_string(&check_path).unwrap();
    let thread_count:usize = thread_siblings.split(",").collect::<Vec<&str>>().len();

    Some(thread_count as i32)

}

fn read_basic_info(sysroot:&str) -> BTreeMap<String, String> {

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
        //lscpu grabs the values from the first processor it finds in cpuinfo
        //we do the same, but then keep iterating to get total num of CPUs
        if line_parts[0].trim() == "processor"{
            processor = line_parts[1].trim().to_string();
            data.insert(line_parts[0].trim().to_string(), line_parts[1].trim().to_string());
        }
        //we've reached the end of a CPU, now we just need CPU count
        if data.contains_key(&line_parts[0].trim().to_string()){
            continue;
        }

        data.insert(line_parts[0].trim().to_string(), line_parts[1].trim().to_string());

    }

    //we get these as ints, and will use them to extrapolate sockets, hw cores, etc.
    //If we can't get this data, extrapolated data won't be printed as well.
    let core_siblings = match get_core_siblings(&sysroot){
        Some(c)  => { c }
        None => { -1 }
    };
    let threads_per_core = match get_threads_per_core(&sysroot){
        Some(t)  => { t }
        None => { -1 }
    };

    if core_siblings > 0 && threads_per_core > 0 && processor != "" {
        let total_cpus = processor.parse::<i32>().unwrap() + 1;
        let cores_per_socket = core_siblings / threads_per_core;
        let hw_sockets = (total_cpus / threads_per_core) / cores_per_socket;

        data.insert("cores_per_socket".to_string(), cores_per_socket.to_string());
        data.insert("sockets".to_string(), hw_sockets.to_string());
        data.insert("threads_per_core".to_string(), threads_per_core.to_string());

    }

    //How we get the total logical CPUs
    if processor != ""  {
        data.insert("CPUs".to_string(), ((processor.parse::<i32>().unwrap()) + 1).to_string());
    }

    //virt flag data, op-mode data
    if data.contains_key("flags"){
        let flist = data.get("flags").unwrap().clone();
        if flist.contains("svm"){
            data.insert("virtualization".to_string(), "AMD-V".to_string());
        } else if flist.contains("vmx") {
            data.insert("virtualization".to_string(), "VT-x".to_string());
        }

        if flist.contains("lm") || flist.contains("sun4v") || flist.contains("ppc64"){
            data.insert("op_mode".to_string(), "32-bit, 64-bit".to_string());
        }

        if flist.contains("ppc"){
            data.insert("op_mode".to_string(), "32-bit".to_string());
        }

    }

    data

}

/*
This is a wrapper around the entire set of data-gathering operations
*/
fn generate_info(sysroot:&str) -> Option<BTreeMap<String, String>>{
    //the basics
    let mut basic = read_basic_info(&sysroot);

    //online cpu count
    let online = get_online(&sysroot);
    if online.is_some(){
        basic.insert("online".to_string(), online.unwrap());
    }

    let mhz_range = cpu_range(&sysroot);
    if mhz_range.is_some(){
        //This may look strange, but... We want to operation on a no-assumption basis
        basic.append(&mut mhz_range.unwrap());
    }

    let numa_info = handle_numa(&sysroot);
    if numa_info.is_some(){
        basic.append(&mut numa_info.unwrap());
    }

    let arch = handle_uname();
    if arch.is_some(){
        basic.append(&mut arch.unwrap());
    }

    let mut end = handle_byte_order();
    basic.append(&mut end);

    let vendor = handle_hypervisor(sysroot);
    if vendor.is_some(){
        basic.append(&mut vendor.unwrap());
    }

    if basic.len() != 0{
        return Some(basic);
    }else{
        return None;
    }

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
        Err(_) => { println!("Invalid path supplied to -s."); exit(1) }
    };

    //This vector is the primary source of truth.
    // Format: ([Printable Name], [Name in dict])
    //because the formats of /proc/cpuinfo and /sys/ can vary, make no assumptions
    let known_datapoints = vec![("Architecture:", "arch"),
                                ("CPU op-mode(s):", "op_mode"),
                                ("Byte Order:", "endian"),
                                ("CPUs:","CPUs"),
                                ("On-line CPU(s):", "online"),
                                ("Threads Per Core:", "threads_per_core"),
                                ("Core(s) Per Socket:", "cores_per_socket"),
                                ("Socket(s):", "sockets"),
                                ("NUMA Node(s):", "numa_nodes"),
                                ("Vendor ID:", "vendor_id"),
                                ("CPU Family:", "cpu family"),
                                ("Model:", "model"),
                                ("Model Name:", "model name"),
                                ("Stepping:", "stepping"),
                                ("CPU MHz:", "cpu MHz"),
                                ("CPU Max MHz:", "cpu_max"),
                                ("CPU Min MHz:", "cpu_min"),
                                ("BogoMIPS:", "bogomips"),
                                ("Hypervisor vendor:", "hypervisor_vendor"),
                                ("Virtualization type:", "hypervisor_type"),
                                ("Virtualization:", "virtualization"),
                                ("CACHEDATA", "null"),
                                ("NUMADATA", "null"),
                                ("Flags:", "flags")
                                ];


    let basic = generate_info(&sysroot);
    //cache is handled differently, as number, config is unknown.
    let cache_info = handle_cache(&sysroot);
    if basic.is_some(){
        normal_print(&basic.unwrap(), cache_info, known_datapoints);
    } else {
        //on the off chance that something has gone _really_ wrong
        println!("No CPU info found");
    }
    //println!("\n{:?}", basic);

}
