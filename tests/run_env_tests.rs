mod envtester;

use envtester::envTester;

/*
These our are actual test cases to run.
Little to do, just point it at the directory name.
*/

#[test]
fn run_a_test_armv7(){

    let path = "armv7";
    let ignore = vec![];

    let arm_test = envTester::new(path);
    arm_test.start_test(ignore, false);

}

#[test]
fn run_a_test_x86_64_dell_e4310(){

    let path = "x86_64-dell_e4310";
    let ignore = vec![];


    let arm_test = envTester::new(path);
    arm_test.start_test(ignore, false);

}

#[test]
fn run_a_test_x86_64_64cpu(){

    let path = "x86_64-64cpu";
    let ignore = vec![];


    let arm_test = envTester::new(path);
    arm_test.start_test(ignore, false);

}

#[test]
fn run_a_test_vbox_win(){

    let path = "vbox-win";
    let ignore = vec![];


    let arm_test = envTester::new(path);
    arm_test.start_test(ignore, false);

}

#[test]
fn run_a_test_ppc_qemu(){
    let path = "ppc-qemu";

    //Notable odd behavior in LSCPU:
    //It prints the cpuinfo rev string as the model number.
    //We also ignore missing cache info.
    let ignore = vec![7, 10, 11];

    let tester = envTester::new(path);
    tester.start_test(ignore, false);
}
