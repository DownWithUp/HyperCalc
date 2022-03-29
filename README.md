# HyperCalc
An Intel HAXM powered, protected mode, 32 bit, hypervisor addition calculator, written in Rust.

## Purpose
None 😏.
Mostly just to learn Rust better. Especially looking at the [winapi](https://crates.io/crates/winapi) crate.

## Requirements 
* [HAXM for Windows](https://github.com/intel/haxm/releases)

## Notes
Lots of unsafe Rust used.
Some things I found interesting related to working with HAXM:

* Real mode is expected to be handled by the user mode component which is usually listed as QEMU. Make sure CR0.PE is set!
* It is not required to register a vCPU tunnel to actually run a VM. However if you do you can get extra information such as the VM-exit code.
* Some registers have default values set by HAXM for a newly created vCPU. These include the DR6, and DR7 registers. This happens if after creating a new vCPU you first call `HAX_VCPU_GET_REGS`.
* The `vcpu_state_t` is essentially just a limited version of a VMCS.
* Remember that VMCS fields may not be sized accurately to what they reflect in an actual OS. I'm looking at you segment descriptors limits!🧐


## Useful resources:
* [HAXM Device API](https://github.com/intel/haxm/blob/master/docs/api.md)
* [Semi-Working Example](https://github.com/Nukem9/Haxm)