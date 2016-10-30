##RSCPU

`rscpu` is a re-write of the classic `lscpu` utility, written entirely in rust.

This is an early alpha, and under development. If your kernel/CPU driverges too much from my 5 year old Xeon desktop, it may do something odd.

#Building
`cargo {run, build}`

#Done
- ✔️ Add `uname` capability so we can get arch
- ✔️ Preliminary support for `-s`
- ✔️ Read in Data from /proc/cpuinfo
- ✔️ Get cache data
- ✔️ Extract op-mode from CPU flags
- ✔️ NUMA info

#Todo
- Start automating tests
- Add support for endian-ness
- Add some of the compatibility features features from `lscpu`
- Add colored terminal output. For fun.
- Full command-line compatibility with `lscpu`
- Pretty-print CPU flags
- Integer overflow when logical CPUs > 64.
- Unwrap()s on NUMA parsing that I should deal with
- Possibly refactor how we print cache info, get rid of extra args

#Extra Reading
- https://www.kernel.org/doc/Documentation/cputopology.txt
- https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-devices-system-cpu

#Known compatibility issues:
- ❌ IBM Power 7
- ❌ sparc64
