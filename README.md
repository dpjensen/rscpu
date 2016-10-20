##RSCPU

`rscpu` is a re-write of the classic `lscpu` utility, written entirely in rust.

This is an early alpha, and under development. If your kernel/CPU driverges too much from my 5 year old Xeon desktop, it may do something odd.

#Building
`cargo {run, build}`

#Done
- ✔️ Preliminary support for `-s`
- ✔️ Read in Data from /proc/cpuinfo
- ✔️ Get cache data

#Todo
- Add `uname` capability so we can get arch
- Add support for endian-ness
- Add some of the compatibility features features from `lscpu`
- Add colored terminal output. For fun.
- Support for machine readable output (`-p`)
- Full command-line compatibility with `lscpu`
- Pretty-print CPU flags
- Complete NUMA support

#Extra Reading
- https://www.kernel.org/doc/Documentation/cputopology.txt
- https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-devices-system-cpu

#Known compatibility issues:
- ❌ IBM Power 7
- ❌ sparc64
