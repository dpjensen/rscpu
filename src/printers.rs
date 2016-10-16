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

//this is the default printing style
pub fn normal_print(cpuinfo:&BTreeMap<String, String>, known_datapoints:Vec<(&str, &str)>) {

    for datapoint in known_datapoints{
        pretty_print_line(datapoint.0, cpuinfo.get(datapoint.1));
    }

}
