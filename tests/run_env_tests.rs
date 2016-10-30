mod envtester;

use envtester::envTester;

/*
These our are actual test cases to run.
Little to do, just point it at the directory name.
*/

#[test]
fn run_a_test_armv7(){

    let path = "armv7";

    let arm_test = envTester::new(path);
    arm_test.start_test();

}

#[test]
fn run_a_test_x86_64_dell_e4310(){

    let path = "x86_64-dell_e4310";

    let arm_test = envTester::new(path);
    arm_test.start_test();

}

#[test]
fn run_a_test_x86_64_64cpu(){

    let path = "x86_64-64cpu";

    let arm_test = envTester::new(path);
    arm_test.start_test();

}
