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
- ✔️ basic test framework in `tests/`
- ✔️ Add support for endian-ness
- ✔️ Tested + working on multiple x86 systems
- ✔️ Basic support for virtualization information

#Todo
- Continue expanding support for virtualization vedor/type info from pci devices
 - Implement inline asm for `cpuid`
 - Look into features for dmi info
 - Basically lots of work to do
- Add some of the compatibility features from `lscpu`
- Full command-line compatibility with `lscpu`
- Integer overflow when logical CPUs > 64.

#On Compatibility with `lscpu`
The end goal is to have a drop-in replacement for `lscpu`. Variables/stats will be in the same order, so shell scripts expecting thing `x` at line `y` won't break.
However, we're not  interested in making the output 100% identical. Eventually CPU flags will print a little cleaner, and `,` characters may become `-`, etc.

#Extra Reading
- https://www.kernel.org/doc/Documentation/cputopology.txt
- https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-devices-system-cpu

#Known compatibility issues:
- ❌ IBM Power 7 / PPC
- ❌ sparc64
