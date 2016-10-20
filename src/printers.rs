use std::collections::BTreeMap;


//A simple wrapper for dealing with potentially missing data.
pub fn pretty_print_line(fmt:&str, dict_out:Option<&String>){

    //if it doesn't exist, just quit
    if dict_out.is_none(){
        return;
    }

    let dict_string = dict_out.unwrap();
    //Here we pad with whitespace so it looks nice
    let total_space = 30;
    let fmt_len = total_space - fmt.len();
    let whitespace_amt = (1..(fmt_len+1)).map(|_| " ").collect::<String>();
    let pretty_string = format!("{}{}", fmt, whitespace_amt);

    println!("{} {}", pretty_string, dict_string);

}

fn print_cache(cacheinfo:Vec<BTreeMap<String, String>>){
    for cache in cacheinfo{
        let blank = String::new();
        let clevel = cache.get("level").unwrap_or(&blank);
        let csize = cache.get("size");
        let ctype = cache.get("type").unwrap_or(&blank);
        //We need at least two datapoints...
        if csize.is_none(){
            continue;
        }

        let prefix = format!("L{}-{} cache:", clevel, ctype);
        pretty_print_line(&prefix, csize);

    }
}


//this is the default printing style
pub fn normal_print(cpuinfo:&BTreeMap<String, String>,
                    cacheinfo:Option<Vec<BTreeMap<String, String>>>,
                    known_datapoints:Vec<(&str, &str)>) {

    for datapoint in known_datapoints{
        //I just want the cache info printed at the same place
        if datapoint.0 == "CACHEDATA" && !cacheinfo.is_none(){
            let cache = cacheinfo.clone().unwrap();
            print_cache(cache);
            continue;
        }

        pretty_print_line(datapoint.0, cpuinfo.get(datapoint.1));
    }

}
