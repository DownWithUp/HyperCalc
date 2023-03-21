use std::ptr;

use winapi::um::memoryapi::VirtualAlloc;
use winapi::um::winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE};
use winapi::um::errhandlingapi::GetLastError;
use winapi::shared::basetsd::UINT8;


const RAM_SIZE: u32 = 0x4000;

mod haxm_interface_windows;

mod haxm {
    
    use std::mem;
    use std::ptr;
    use winapi::um::winnt::*;
    use winapi::um::fileapi::*;
    use winapi::shared::minwindef::*;
    use winapi::um::errhandlingapi::*;
    use winapi::um::ioapiset::*;
    use winapi::ctypes::*;
    use winapi::shared::basetsd::*;
    use crate::haxm_interface_windows::*;

    /// Helper function because winapi booleans are rust i32s.
    pub fn win_bool_eval(input: BOOL) -> bool {
        if input == 0 {
            false
        }
        else {
            true
        }
    }

    pub struct HaxmVCPU {
        pub vcpu_handle: HANDLE,
        pub id: UINT32,
        pub cpu_state: vcpu_state_t,
        pub tunnel: hax_tunnel_info
    }
    
    impl HaxmVCPU {
    
        /// Associated function constructor. On success constructs a new HaxmVCPU, else returns the value of GetLastError()
        /// 
        /// # Arguments
        /// 
        /// * `id` -  The ID assigned to this VCPU when it is created. Normally this is done by HAX_VM_IOCTL_VCPU_CREATE in HaxmVM.new_vcpu().
        /// * `vm_id` - The ID of the parent VM creating this vcpu.`
        pub fn new(id: UINT32, vm_id: UINT32) -> Result<Self, DWORD> {
            unsafe {
                let formatted_name = format!("\\\\.\\hax_vm{:02}_vcpu{:02}\0", id, vm_id);
                let vm_name: &str = &formatted_name;
                let vcpu_handle = CreateFileA(vm_name as *const str as *const i8, GENERIC_READ | GENERIC_WRITE, 0, ptr::null_mut(),
                                                            OPEN_EXISTING, 0, ptr::null_mut());
                let new_vcpu = HaxmVCPU {
                    vcpu_handle: vcpu_handle,
                    id: id,
                    cpu_state: mem::zeroed::<vcpu_state_t>(),
                    tunnel: mem::zeroed::<hax_tunnel_info>()
                };
    
                let last_error = GetLastError();
    
                if last_error == 0 {
                    return Ok(new_vcpu);
                }
                else {
                    return Err(last_error);
                }
            }
        }
    
        /// Creates a tunnel from the HAXM driver to the user (designed for QEMU) modules for dealing specific actions that the 
        /// guest performs which are not supported by the driver. On success returns None and this vCPU's tunnel member is valid.
        /// On failure returns the value of GetLastError.
        pub fn setup_vcpu_tunnel(&mut self) -> Option<DWORD> {
            unsafe {
    
                let tunnel_info = mem::zeroed::<hax_tunnel_info>();
    
                let was_successful = DeviceIoControl(self.vcpu_handle, HAX_VCPU_IOCTL_SETUP_TUNNEL, ptr::null_mut(), 0, 
                    &tunnel_info as *const hax_tunnel_info as *mut c_void, 
                mem::size_of_val(&tunnel_info) as u32, ptr::null_mut(), ptr::null_mut()); 
    
                if win_bool_eval(was_successful) {
                    self.tunnel = tunnel_info;
                    return None;
                }
                else  {
                    return Some(GetLastError());
                }
            }
        }
    
        /// Gets the VCPUs registers from the Haxm created vCPU. On success returns None, else returns the value of GetLastError(). 
        pub fn get_regs(&mut self) -> Option<DWORD> {
            unsafe {
                let was_successful = DeviceIoControl(self.vcpu_handle, HAX_VCPU_GET_REGS, ptr::null_mut(), 0, 
                    &mut self.cpu_state as *const vcpu_state_t as *mut c_void, 
                    mem::size_of_val(&self.cpu_state) as u32, ptr::null_mut(), ptr::null_mut());
    
                if win_bool_eval(was_successful) {
                    None
                }
                else {
                    Some(GetLastError())
                }
            }
        }
    
        /// Sets the VCPUs registers for the vCPU. On success returns None, else returns the value of GetLastError(). 
        pub fn set_regs(&mut self) -> Option<DWORD> {
            unsafe {
                let was_successful = DeviceIoControl(self.vcpu_handle, HAX_VCPU_SET_REGS, 
                    &mut self.cpu_state as *const vcpu_state_t as *mut c_void, 
                    mem::size_of_val(&self.cpu_state) as u32, ptr::null_mut(), 0, ptr::null_mut(), ptr::null_mut());
    
                if win_bool_eval(was_successful) {
                    None
                }
                else {
                    Some(GetLastError())
                }
            }
        }
    
        /// Runs the VCPU until a VM-Exit occurs. On success returns None, else returns the value of GetLastError().
        pub fn run(&self) -> Option<DWORD> {
            unsafe {
                let was_successful = DeviceIoControl(self.vcpu_handle, HAX_VCPU_IOCTL_RUN, ptr::null_mut(), 0, 
                    ptr::null_mut(), 0, ptr::null_mut(), ptr::null_mut());
    
                if win_bool_eval(was_successful) {
                    
                    return None;
                }
                else {
                    return Some(GetLastError());
                }
            }
        }
    
    }
    
    
    pub struct HaxmVM {
        pub vm_handle: HANDLE,
        pub id: UINT32,
        pub vcpus: Vec<HaxmVCPU>
    }
    
    impl HaxmVM {
    
        /// Associated function constructor. On success constructs a new HaxmVM, else returns the value of GetLastError()
        /// 
        /// # Arguments
        /// 
        /// * `id` - The ID assigned to this VM when it is created. Normally this is done by HAX_IOCTL_CREATE_VM in HaxmDevice.create_vm().
        pub fn new(id: UINT) -> Result<Self, DWORD> {
    
            unsafe {
                let formatted_name = format!("\\\\.\\hax_vm{:02}\0", id);
                let vm_name: &str = &formatted_name;
                let vm_handle = CreateFileA(vm_name as *const str as *const i8, GENERIC_READ | GENERIC_WRITE, 0, ptr::null_mut(),
                                                            OPEN_EXISTING, 0, ptr::null_mut());
        
                let last_error = GetLastError();
        
                if last_error == 0 {
                    let new_vm = HaxmVM {
                        vm_handle: vm_handle,
                        id: id,
                        vcpus: vec!()
                    };
                    
                    return Ok(new_vm);
                }
                else {
                    return Err(last_error);
                }
            }
        }
    
        /// Allocates RAM for the VM. If If successful, returns None, else returns Some with the result of GetLastError()
        /// 
        /// # Arguments
        /// 
        /// * `hva` - The start address of the user buffer. Must be page-aligned (i.e. a multiple of 4KB), and must not be 0. 
        /// The HVA range specified by va and size must not overlap with that of any previously registered user buffer for the same VM.
        /// Registers with HAXM a user space buffer to be used as memory for this VM. Currently, 
        /// HAXM does not allow mapping a guest physical address (GPA) range to a host virtual address (HVA) range that does not 
        /// belong to any previously registered buffers.
        /// * `size` - The size of the user buffer to register, in bytes. 
        /// Must be in whole pages (i.e. a multiple of 4KB), and must not be 0. Note that this IOCTL can only handle buffers smaller than 4GB.
        pub fn alloc_ram(&self, hva: UINT64, size: UINT32) -> Option<DWORD> {
            unsafe {
    
                let ram_info = hax_alloc_ram_info {
                    size: size,
                    pad: 0,
                    va: hva
                };
    
                let was_successful = DeviceIoControl(self.vm_handle, HAX_VM_IOCTL_ALLOC_RAM, 
                    &ram_info as *const hax_alloc_ram_info as *mut c_void,  
                    mem::size_of_val(&ram_info) as DWORD, ptr::null_mut(), 0, ptr::null_mut(), ptr::null_mut());
                if win_bool_eval(was_successful) {
                    return None
                }
                else {
                    return Some(GetLastError())
                }
    
            }
        }
    
        /// Sets the RAM size of the VM. If successful, returns None, else returns Some with the result of GetLastError()
        ///
        /// # Arguments
        ///
        /// * `gpa_start` - The start address of the GPA (Guet Physical Address) range to map. Must be page- aligned (i.e. a multiple of 4KB).
        /// * `size` - Size of the mapping. The size of the GPA range, in bytes. Must be in whole pages (i.e. a multiple of 4KB), and must not be 0. 
        /// If the GPA range covers any guest physical pages that are already mapped, those pages will be remapped.
        /// * `hva_start` The start address of the HVA range to map to. Must be page- aligned (i.e. a multiple of 4KB), and must not be 0 (except when flags == HAX_RAM_INFO_INVALID). 
        /// The size of the HVA range is specified by size. The entire HVA range must fall within a previously registered user buffer.
        pub fn set_ram(&self, gpa_start: UINT64, size: UINT32, hva_start: UINT64) -> Option<DWORD> {
            unsafe {
                let set_info = hax_set_ram_info {
                    pa_start: gpa_start,
                    size: size,
                    flags: 0,
                    pad: [0,0,0],
                    va: hva_start
                };
    
                let was_successful = DeviceIoControl(self.vm_handle, HAX_VM_IOCTL_SET_RAM, 
                    &set_info as *const hax_set_ram_info as *mut c_void, 
                    mem::size_of_val(&set_info) as DWORD, ptr::null_mut(), 0, ptr::null_mut(), ptr::null_mut());
    
                if win_bool_eval(was_successful) {
                    return None
                }
                else {
                    Some(GetLastError())
                }
            }
        }
    
        /// Creates a new cpu associated with a VM. If successful returns None, else returns the value of GetLastError().
        /// 
        /// # Arguments
        /// 
        /// * `vcpu_id` - The VCPU ID that uniquely identifies the new VCPU among the VCPUs in the same VM. Must be less than 16. 
        /// Before API v3, only one VCPU was allowed per VM, and this parameter was ignored.
        pub fn new_cpu(&mut self, vcpu_id: UINT32) -> Option<DWORD> {
            unsafe {
                let was_successful = DeviceIoControl(self.vm_handle, HAX_VM_IOCTL_VCPU_CREATE, 
                    &vcpu_id as *const UINT32 as *mut c_void, mem::size_of_val(&vcpu_id) as u32, ptr::null_mut(), 0, ptr::null_mut(), ptr::null_mut());
    
                if win_bool_eval(was_successful) {
                    let new_vcpu = HaxmVCPU::new(vcpu_id, self.id);
                    if let Ok(was_successful) = new_vcpu {
                        self.vcpus.push(was_successful);
                        return None;
                    }
                }
                Some(GetLastError())
            }
        }
    }
    
    pub struct HaxmDevice
    {
        pub device_handle: HANDLE,
        pub vms: Vec<HaxmVM>
    }
    
    impl HaxmDevice
    {
        /// Associated function constructor. Constructs a new HaxmDevice
        pub fn new() -> Self {
    
            let new_haxm_device = HaxmDevice {
                device_handle: ptr::null_mut(),
                vms: vec!()
            };
            return new_haxm_device;    
        }
    
        /// Initializes (opens using CreateFile the) the Haxm device. 
        /// If successful returns a handle to the device, else returns the value of GetLastError().
        pub fn initialize(&mut self) -> Result<HANDLE, DWORD>
        {
            unsafe {
                // HAXM Device string on Windows
                let haxm_main_device = "\\\\.\\HAX\0" as *const str as *const i8;
                let haxm_device: HANDLE = CreateFileA(haxm_main_device, GENERIC_READ | GENERIC_WRITE, 0, ptr::null_mut(),
                                                        OPEN_EXISTING, 0, ptr::null_mut()); 
                let last_error = GetLastError();
        
                if last_error == 0 {
                    self.device_handle = haxm_device;
                    return Ok(haxm_device);
                }
                else {
                    return Err(last_error);
                }
            }
        }
        
        /// Creates a new VM. On success returns the ID of the new VM and adds the VM to the vms vector of the HaxmDevice. 
        /// On failure returns the value of GetLastError(). 
        pub fn new_vm(&mut self) -> Result<UINT, DWORD> {
            unsafe {
                let vm_id: UINT = 0;
            
                let was_successful = DeviceIoControl(self.device_handle, HAX_IOCTL_CREATE_VM, ptr::null_mut(), 0, 
                    &vm_id as *const UINT as *mut c_void, 4, ptr::null_mut(), ptr::null_mut());
        
                if win_bool_eval(was_successful) {
                    let new_vm = HaxmVM::new(vm_id);
                    match new_vm
                    {
                        Ok(new_vm) => {
                            self.vms.push(new_vm);
                            return Ok(vm_id)
                        }
                        Err(last_error) => {
                            return Err(last_error);
                        }
                    }
                }
                else {
                    return Err(GetLastError())
                }
        
            }
    
        }
    }
}

fn get_integer_input(prompt: &str) -> Result<u32, String> {
    println!("{}", prompt);
    let mut buffer = String::new();
    if let Ok(_str_len) = std::io::stdin().read_line(&mut buffer) {
        if let Ok(int) = buffer.trim().parse::<u32>() {
            Ok(int)
        }
        else {
            Err(String::from("Unable to parse to u32"))
        }
    }
    else {
        Err(String::from("Unable to read input"))
    }
}

fn main() {

    unsafe {
        let mut haxm_device = haxm::HaxmDevice::new();

        if let Err(last_error) = &haxm_device.initialize() {
            panic!("Unable to initialize HAXM device. GetLastError: {}", last_error);
        }

        let new_vm_result = haxm_device.new_vm(); 
        if let Err(last_error) = new_vm_result {
            panic!("Unable to create a new VM. GetLastError: {}", last_error);
        }
        
        let hva = VirtualAlloc(ptr::null_mut(), RAM_SIZE as usize, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
        if hva == ptr::null_mut() {
            panic!("Unable to allocate memory. Call to VirtualAlloc failed. GetLastError: {}", GetLastError());
        }

        let calc_vm = &mut haxm_device.vms[0]; 

        let mut mem: [UINT8; RAM_SIZE as usize] = [0x90; RAM_SIZE as usize];
        // add eax, ecx
        // hlt
        mem[0x2000] = 0x66;
        mem[0x2001] = 0x01;
        mem[0x2002] = 0xC8;
        mem[0x2003] = 0xf4;

        // Move our custom assembly opcodes into the memory which will become the memory of our guest
        ptr::copy(mem.as_mut_ptr(), hva as *mut u8, mem.len());


        if let Some(last_error) = calc_vm.alloc_ram(hva as u64, RAM_SIZE) {
            panic!("Unable to allocate memory for the VM. GetLastError: {}", last_error);
        }


        if let Some(last_error) = calc_vm.set_ram(0, RAM_SIZE, hva as u64) {
            panic!("Unable to set memory for the VM. GetLastError: {}", last_error);
        }

        if let Some(last_error) = calc_vm.new_cpu(0) {
            panic!("Unable to create a vCPU for the VM. GetLastError: {}", last_error);
        }

        if let Some(last_error) = calc_vm.vcpus[0].setup_vcpu_tunnel() {
            panic!("Unable to setup vCPU channel for vCPU: {}. GetLastError: {}", calc_vm.vcpus[0].id, last_error);
        }


        /*
            Physical Memory (processor linear address space) layout for a pseudo flat model:
            [0x0000 - 0x1fff] [Data segment]
            [0x2000 - 0x3fff] [Code segment]
        */

        // Set the register state
        calc_vm.vcpus[0].cpu_state.cs.selector = 0;
        calc_vm.vcpus[0].cpu_state.cs.limit = 0x3FFF;
        calc_vm.vcpus[0].cpu_state.cs.anon_union.ar = 0x9B;
        calc_vm.vcpus[0].cpu_state.cs.base = 0x2000;

        calc_vm.vcpus[0].cpu_state.ds.selector = 0;
        calc_vm.vcpus[0].cpu_state.ds.limit = 0x1FFF;
        calc_vm.vcpus[0].cpu_state.ds.anon_union.ar = 0x93;
        calc_vm.vcpus[0].cpu_state.ds.base = 0;

        calc_vm.vcpus[0].cpu_state.tr.selector = 0;
        calc_vm.vcpus[0].cpu_state.tr.limit = 0;
        calc_vm.vcpus[0].cpu_state.tr.anon_union.ar = 0x83;
        calc_vm.vcpus[0].cpu_state.tr.base = 0;

        calc_vm.vcpus[0].cpu_state.ldt.selector = 0;
        calc_vm.vcpus[0].cpu_state.ldt.limit = 0;
        calc_vm.vcpus[0].cpu_state.ldt.anon_union.ar = 0x10000;
        calc_vm.vcpus[0].cpu_state.ldt.base = 0;

        calc_vm.vcpus[0].cpu_state.gdt.limit = 0;
        calc_vm.vcpus[0].cpu_state.gdt.base = 0;
        calc_vm.vcpus[0].cpu_state.gdt.anon_union.ar = 0x10000; // Set here, but also automatically by the Haxm driver

        calc_vm.vcpus[0].cpu_state.idt.limit = 0;
        calc_vm.vcpus[0].cpu_state.idt.base = 0;
        calc_vm.vcpus[0].cpu_state.idt.anon_union.ar = 0x10000; // Set here, but also automatically by the Haxm driver

        calc_vm.vcpus[0].cpu_state.cr0 = 0x21; // 0x21
        calc_vm.vcpus[0].cpu_state.cr3 = 0;
        calc_vm.vcpus[0].cpu_state.cr4 = 0x2000;

        calc_vm.vcpus[0].cpu_state.dr6 = 0xFFFF0FF0; // Set here, but also automatically by the Haxm driver
        calc_vm.vcpus[0].cpu_state.dr7 = 0x400; // Set here, but also automatically by the Haxm driver

        calc_vm.vcpus[0].cpu_state.anon_union_1.anon_struct.rsp.b32 = 0x100;
        calc_vm.vcpus[0].cpu_state.anon_union_2.rip = 0;

        calc_vm.vcpus[0].cpu_state.anon_union_3.eflags = 0x202;


        // Collect first number
        let mut int1 = 0;
        let first_input = get_integer_input("Enter first number: ");
        if let Err(error_message) = first_input {
            panic!("{}", error_message);
        }
        else if let Ok(result1) = first_input {
            calc_vm.vcpus[0].cpu_state.anon_union_1.anon_struct.rax.b32 = result1;
            int1 = result1;
        }
        
        // Collect second number
        let mut int2 = 0;
        let second_input = get_integer_input("Enter second number: ");
        if let Err(error_message) = second_input {
            panic!("{}", error_message);
        }
        else if let Ok(result2) = second_input {
            calc_vm.vcpus[0].cpu_state.anon_union_1.anon_struct.rcx.b32 = result2;
            int2 = result2;
        }
          
        if let Some(last_error) = calc_vm.vcpus[0].set_regs() {
            panic!("Unable to set vCPU {} registers. GetLastError: {}", calc_vm.vcpus[0].id, last_error);
        }

        calc_vm.vcpus[0].run();

        if let Some(last_error) = calc_vm.vcpus[0].get_regs() {
            panic!("Unable to get vCPU {} registers. GetLastError: {}", calc_vm.vcpus[0].id, last_error);
        }

        println!("{} + {} = {}", int1, int2, calc_vm.vcpus[0].cpu_state.anon_union_1.anon_struct.rax.b32);
        
    }
}
