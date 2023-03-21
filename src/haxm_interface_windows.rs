#![allow(non_camel_case_types)]
#![allow(dead_code)]

use winapi::shared::minwindef::*;
use winapi::um::winioctl::*;
use winapi::shared::basetsd::*;
use modular_bitfield::prelude::*;

// Almost everything here is from: https://github.com/intel/haxm/blob/master/docs/api.md
// Various #defines are from hax_interface_windows.h

macro_rules! CTL_CODE_MACRO {
    ($device_type: expr, $function: expr, $method: expr, $access: expr)=> {
        {
            ($device_type << 16) | ($access << 14) | ($function << 2) | $method
        }
    }
}

// Original structure has __attribute__ ((__packed__));
#[repr(C, packed(8))]
pub struct hax_alloc_ram_info {
    pub size: UINT32,
    pub pad: UINT32,
    pub va: UINT64
}

// Original structure has __attribute__ ((__packed__));
#[repr(C, packed(8))]
pub struct hax_set_ram_info {
    pub pa_start: UINT64,
    pub size: UINT32,
    pub flags: UINT8,
    pub pad: [UINT8; 3],
    pub va: UINT64
}


// The structs for HAX_VCPU_SET_REGS had to be modified to confirm to Rust syntax
// It gets slightly messy when trying to preserve the original C structure 

#[bitfield]
#[repr(C)]
pub struct interruptibility_state_t_anon_struct {
    pub sti_blocking   : B1,
    pub movss_blocking : B1,
    pub smi_blocking   : B1,
    pub nmi_blocking   : B1,
    pub reserved       : B28
}

#[repr(C)]
pub union interruptibility_state_t {
    pub raw: UINT32,
    pub anon_struct: std::mem::ManuallyDrop<interruptibility_state_t_anon_struct>,
    pub pad: UINT64
}


#[bitfield]
#[repr(C)]
pub struct segment_desc_t_anon_struct {
    pub segment_type: B4,
    pub desc: B1,
    pub dpl: B2,
    pub present: B1,
    pub reserved: B4,
    pub available: B1,
    pub long_mode: B1,
    pub operand_size: B1,
    pub granularity: B1,
    pub null: B1,
    pub reserved2: B15
}

#[repr(C)]
pub union segment_desc_t_anon_union {

    pub anon_struct: std::mem::ManuallyDrop<segment_desc_t_anon_struct>,
    pub ar: UINT32
}

#[repr(C)]
pub struct segment_desc_t {
    pub selector: UINT16,
    pub _dummy: UINT16,
    pub limit: UINT32,
    pub base: UINT64,
    pub anon_union: segment_desc_t_anon_union,
    pub ipad: UINT32
}

// My custom types of a register value
#[repr(C)]
pub struct eight_bit_values {
    pub low: B4,
    pub high: B4
}

#[repr(C)]
pub union gp_reg {
    pub b8: std::mem::ManuallyDrop<eight_bit_values>,
    pub b16: UINT16,
    pub b32: UINT32,
    pub b64: UINT64
} 

#[repr(C)]
pub struct vcpu_state_t_anon_union_1_anon_struct {
    pub rax: gp_reg, 
    pub rcx: gp_reg,
    pub rdx: gp_reg, 
    pub rbx: gp_reg,
    pub rsp: gp_reg,
    pub rbp: gp_reg,
    pub rsi: gp_reg,
    pub rdi: gp_reg,
    pub r8: gp_reg,
    pub r9: gp_reg,
    pub r10: gp_reg,
    pub r11: gp_reg,
    pub r12: gp_reg,
    pub r13: gp_reg,
    pub r14: gp_reg,
    pub r15: gp_reg
}

#[repr(C)]
pub union vcpu_state_t_anon_union_1 {
    pub regs: [UINT64;16],
    pub anon_struct: std::mem::ManuallyDrop<vcpu_state_t_anon_union_1_anon_struct>
}

#[repr(C)]
pub union vcpu_state_t_anon_union_2 {
    pub eip: UINT32,
    pub rip: UINT64
}

#[repr(C)]
pub union vcpu_state_t_anon_union_3 {
    pub eflags: UINT32,
    pub rflags: UINT64
}

#[repr(C)]
pub struct vcpu_state_t {
    
    pub anon_union_1: vcpu_state_t_anon_union_1,

    pub anon_union_2: vcpu_state_t_anon_union_2,

    pub anon_union_3: vcpu_state_t_anon_union_3,

    pub cs: segment_desc_t,
    pub ss: segment_desc_t,
    pub ds: segment_desc_t,
    pub es: segment_desc_t,
    pub fs: segment_desc_t,
    pub gs: segment_desc_t,
    pub ldt: segment_desc_t,
    pub tr: segment_desc_t,

    pub gdt: segment_desc_t ,
    pub idt: segment_desc_t ,

    pub cr0: UINT64,
    pub cr2: UINT64,
    pub cr3: UINT64,
    pub cr4: UINT64,

    pub dr0: UINT64,
    pub dr1: UINT64,
    pub dr2: UINT64,
    pub dr3: UINT64,
    pub dr6: UINT64,
    pub dr7: UINT64,
    pub pde: UINT64,

    pub efer: UINT32,

    pub sysenter_cs: UINT32,
    pub sysenter_eip: UINT64,
    pub sysenter_esp: UINT64,

    pub activity_state: UINT32,
    pub pad: UINT32,
    pub interruptibility_state: interruptibility_state_t
}

#[repr(C, packed(4))]
pub struct hax_qemu_version {
    pub cur_version: UINT32,
    pub least_version: UINT32
}

#[repr(C, packed(4))]
pub struct hax_tunnel_info {
    pub va: UINT64,
    pub io_va: UINT64,
    pub size: UINT16,
    pub pad: [UINT16; 3],
}


pub const HAX_DEVICE_TYPE: DWORD    =  0x4000;
//
pub const HAX_IOCTL_VERSION: DWORD  = CTL_CODE_MACRO!(HAX_DEVICE_TYPE, 0x900, METHOD_BUFFERED, FILE_ANY_ACCESS);
//
//pub const HAX_IOCTL_CREATE_VM: DWORD       = CTL_CODE_MACRO!(HAX_DEVICE_TYPE, 0x901, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const HAX_IOCTL_CREATE_VM: DWORD = (0x4000 << 16) | (FILE_ANY_ACCESS << 14) | (0x901 << 2) | METHOD_BUFFERED;
// 14 and 2 work??? HMMMM


//const HAX_IOCTL_CAPABILITY: DWORD     = CTL_CODE_MACRO(HAX_DEVICE_TYPE, 0x910, METHOD_BUFFERED, FILE_ANY_ACCESS);
//const HAX_IOCTL_SET_MEMLIMIT: DWORD    = CTL_CODE_MACRO(HAX_DEVICE_TYPE, 0x911, METHOD_BUFFERED, FILE_ANY_ACCESS);
//
pub const HAX_VM_IOCTL_VCPU_CREATE: DWORD  = CTL_CODE_MACRO!(HAX_DEVICE_TYPE, 0x902, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const HAX_VM_IOCTL_ALLOC_RAM: DWORD    = CTL_CODE_MACRO!(HAX_DEVICE_TYPE, 0x903, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const HAX_VM_IOCTL_SET_RAM: DWORD      = CTL_CODE_MACRO!(HAX_DEVICE_TYPE, 0x904, METHOD_BUFFERED, FILE_ANY_ACCESS);
//const HAX_VM_IOCTL_VCPU_DESTROY: DWORD = CTL_CODE_MACRO(HAX_DEVICE_TYPE, 0x905, METHOD_BUFFERED, FILE_ANY_ACCESS);
//const HAX_VM_IOCTL_ADD_RAMBLOCK: DWORD = CTL_CODE_MACRO(HAX_DEVICE_TYPE, 0x913, METHOD_BUFFERED, FILE_ANY_ACCESS);
//const HAX_VM_IOCTL_SET_RAM2: DWORD     = CTL_CODE_MACRO(HAX_DEVICE_TYPE, 0x914, METHOD_BUFFERED, FILE_ANY_ACCESS);
//const HAX_VM_IOCTL_PROTECT_RAM: DWORD  = CTL_CODE_MACRO(HAX_DEVICE_TYPE, 0x915, METHOD_BUFFERED, FILE_ANY_ACCESS);
//
pub const HAX_VCPU_IOCTL_RUN: DWORD        = CTL_CODE_MACRO!(HAX_DEVICE_TYPE, 0x906, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const HAX_VCPU_IOCTL_SET_MSRS: DWORD   = CTL_CODE_MACRO!(HAX_DEVICE_TYPE, 0x907, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const HAX_VCPU_IOCTL_GET_MSRS: DWORD   = CTL_CODE_MACRO!(HAX_DEVICE_TYPE, 0x908, METHOD_BUFFERED, FILE_ANY_ACCESS);

pub const HAX_VCPU_IOCTL_SET_FPU: DWORD    = CTL_CODE_MACRO!(HAX_DEVICE_TYPE, 0x909, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const HAX_VCPU_IOCTL_GET_FPU: DWORD    = CTL_CODE_MACRO!(HAX_DEVICE_TYPE, 0x90a, METHOD_BUFFERED, FILE_ANY_ACCESS);

pub const HAX_VCPU_IOCTL_SETUP_TUNNEL: DWORD = CTL_CODE_MACRO!(HAX_DEVICE_TYPE, 0x90b, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const HAX_VCPU_IOCTL_INTERRUPT: DWORD  = CTL_CODE_MACRO!(HAX_DEVICE_TYPE, 0x90c, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const HAX_VCPU_SET_REGS: DWORD         = CTL_CODE_MACRO!(HAX_DEVICE_TYPE, 0x90d, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const HAX_VCPU_GET_REGS: DWORD         = CTL_CODE_MACRO!(HAX_DEVICE_TYPE, 0x90e, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const HAX_VCPU_IOCTL_KICKOFF: DWORD    = CTL_CODE_MACRO!(HAX_DEVICE_TYPE, 0x90f, METHOD_BUFFERED, FILE_ANY_ACCESS);
//
/* API version 2.0 */
//pub const HAX_VM_IOCTL_NOTIFY_QEMU_VERSION: DWORD  = CTL_CODE_MACRO!(HAX_DEVICE_TYPE, 0x910, METHOD_BUFFERED, FILE_ANY_ACCESS);
//
//const HAX_IOCTL_VCPU_DEBUG: DWORD      = CTL_CODE_MACRO(HAX_DEVICE_TYPE, 0x916, METHOD_BUFFERED, FILE_ANY_ACCESS);
//const HAX_VCPU_IOCTL_SET_CPUID: DWORD  = CTL_CODE_MACRO(HAX_DEVICE_TYPE, 0x917, METHOD_BUFFERED, FILE_ANY_ACCESS);
//const HAX_VCPU_IOCTL_GET_CPUID: DWORD  = CTL_CODE_MACRO(HAX_DEVICE_TYPE, 0x918, METHOD_BUFFERED, FILE_ANY_ACCESS);