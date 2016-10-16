##RSCPU

`rscpu` is a re-write of the classic `lscpu` utility, written entirely in rust.

This is an early alpha, and under development. If your kernel/CPU driverges too much from my 5 year old Xeon desktop, it may do something odd.

#Building
`cargo {run, build}`


#Todo
- Finish support for `-s` so I can run actual tests
- Add `uname` capability so we can get arch
- Add support for endian-ness
- Add some of the compatibility features features from `lscpu`
- Add colored terminal output. For fun.
- support for machine readable output (`-p`)
